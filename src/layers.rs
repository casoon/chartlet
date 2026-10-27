//! Layers: levels of abstraction stacked on each other, the first at the bottom, or scopes drawn
//! inside each other, the first the outermost. Every layer has a label, the items it holds as
//! small boxes under it, an optional note at the side and an optional role. Items wrap onto rows
//! that fit the width of their layer; the notes stand in a column right of the diagram, joined to
//! their layer by a dotted line, or, on a narrow canvas, under the label. The description names
//! every layer with its items and note in order.

use crate::{
    DataTable,
    diagram::{self, pixels, warn_growth_advising},
    error::ChartWarning,
    flow::focus_classes,
    layout::{NARROW, count, fit_text, push_title, title_extra, two_lines},
    metrics::TextMetrics,
    roles,
    scene::{Element, Hotspot, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, DiagramRole, LayersDisplay, LayersSpec},
    text,
};

const MARGIN: f64 = 24.0;
const COMPACT_MARGIN: f64 = 12.0;
const BOTTOM: f64 = 16.0;
const PAD: f64 = 12.0;
const LABEL_LINE: f64 = 18.0;
const NOTE_LINE: f64 = 15.0;
const CHIP_HEIGHT: f64 = 24.0;
const CHIP_GAP: f64 = 6.0;
const CHIP_TEXT: f64 = 12.0;
const LABEL_SIZE: f64 = 13.0;
/// The gap between two bands of a stack, and the inset of every scope inside the one around it.
const BAND_GAP: f64 = 8.0;
const INSET: f64 = 14.0;
/// The column of notes at the side, and the space between it and the diagram.
const NOTE_COLUMN: f64 = 200.0;
const NOTE_GAP: f64 = 28.0;
const NOTE_LINES: usize = 4;

fn layers(spec: &ChartSpec) -> &LayersSpec {
    spec.layer_stack
        .as_ref()
        .expect("validated layer diagrams carry a layers block")
}

/// Text on at most `lines` lines that fit `max_width`, broken at spaces; the last is shortened.
pub(crate) fn wrap_lines(
    text: &str,
    max_width: f64,
    size: f64,
    lines: usize,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> Vec<String> {
    let mut output = Vec::new();
    let mut rest = text;
    while output.len() + 1 < lines {
        match two_lines(rest, max_width, size, metrics) {
            Some((first, more)) => {
                output.push(first.to_owned());
                rest = more;
            }
            None => break,
        }
    }
    output.push(fit_text(rest, max_width, size, metrics, warnings, path));
    output
}

/// What one layer shows, measured.
struct Measured {
    label: String,
    /// Every item with the width of its box.
    chips: Vec<(String, f64)>,
    note: Vec<String>,
}

/// Where a layer stands: its box and where its items begin.
struct Place {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    /// The rows of items, as indices into the chips, and the top of the first row.
    rows: Vec<Vec<usize>>,
    items_top: f64,
}

/// Breaks items into rows that fit `room`.
fn rows(chips: &[(String, f64)], room: f64) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut used = 0.0;
    for (index, (_, width)) in chips.iter().enumerate() {
        match rows.last_mut() {
            Some(row) if used + CHIP_GAP + width <= room => {
                row.push(index);
                used += CHIP_GAP + width;
            }
            _ => {
                rows.push(vec![index]);
                used = *width;
            }
        }
    }
    rows
}

fn measure(
    layers: &LayersSpec,
    room: f64,
    note_width: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<Measured> {
    layers
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let path = format!("/layers/layers/{index}");
            let label = fit_text(
                &layer.label,
                room,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("{path}/label"),
            );
            let chips = layer
                .items
                .iter()
                .enumerate()
                .map(|(item, name)| {
                    let name = fit_text(
                        name,
                        room - 16.0,
                        CHIP_TEXT,
                        metrics,
                        warnings,
                        &format!("{path}/items/{item}"),
                    );
                    let width = metrics.width(&name, CHIP_TEXT) + 16.0;
                    (name, width)
                })
                .collect();
            let note = layer.note.as_ref().map_or_else(Vec::new, |note| {
                wrap_lines(
                    note,
                    note_width,
                    11.0,
                    NOTE_LINES,
                    metrics,
                    warnings,
                    &format!("{path}/note"),
                )
            });
            Measured { label, chips, note }
        })
        .collect()
}

/// The height of a layer's header: padding, label and, where they stand inside, the note.
fn header(measured: &Measured, note_inside: bool) -> f64 {
    let notes = if note_inside {
        NOTE_LINE * count(measured.note.len())
    } else {
        0.0
    };
    PAD + LABEL_LINE + notes + 4.0
}

/// The height of `rows` rows of items.
fn items_height(rows: usize) -> f64 {
    if rows == 0 {
        0.0
    } else {
        (CHIP_HEIGHT + CHIP_GAP) * count(rows) - CHIP_GAP
    }
}

struct Plan {
    places: Vec<Place>,
    width: f64,
    height: f64,
    /// Where the notes stand, `None` when none stands at the side.
    note_x: Option<f64>,
}

fn plan(
    spec: &ChartSpec,
    layers: &LayersSpec,
    measured: &[Measured],
    (margin, top, side): (f64, f64, bool),
) -> Plan {
    let width = f64::from(spec.width);
    let note_x = side.then_some(width - margin - NOTE_COLUMN);
    let body = width - 2.0 * margin - if side { NOTE_COLUMN + NOTE_GAP } else { 0.0 };
    let note_inside = !side;
    let count_layers = layers.layers.len();
    let mut places: Vec<Place> = Vec::new();
    let height = match layers.display {
        LayersDisplay::Stack => {
            let room = body - 2.0 * PAD;
            // The last layer stands on top.
            let mut y = top;
            for index in (0..count_layers).rev() {
                let item_rows = rows(&measured[index].chips, room);
                let head = header(&measured[index], note_inside);
                let mut band = head + items_height(item_rows.len()) + PAD;
                if side {
                    band = band.max(2.0 * PAD + NOTE_LINE * count(measured[index].note.len()));
                }
                places.push(Place {
                    x: margin,
                    y,
                    width: body,
                    height: band,
                    rows: item_rows,
                    items_top: y + head,
                });
                y += band + BAND_GAP;
            }
            places.reverse();
            y - BAND_GAP + BOTTOM
        }
        LayersDisplay::Nested => {
            // The rows of every scope depend on its width, its height on the scope inside it.
            let rows_of: Vec<Vec<Vec<usize>>> = (0..count_layers)
                .map(|index| {
                    let scope_width = body - 2.0 * INSET * count(index);
                    rows(&measured[index].chips, scope_width - 2.0 * PAD)
                })
                .collect();
            let mut heights = vec![0.0; count_layers];
            for index in (0..count_layers).rev() {
                let items = items_height(rows_of[index].len());
                let gap = if items > 0.0 { BAND_GAP } else { 0.0 };
                let below = if index + 1 < count_layers {
                    heights[index + 1] + INSET
                } else {
                    PAD
                };
                heights[index] = header(&measured[index], note_inside) + items + gap + below;
            }
            let mut y = top;
            for (index, item_rows) in rows_of.into_iter().enumerate() {
                let head = header(&measured[index], note_inside);
                let items = items_height(item_rows.len());
                let gap = if items > 0.0 { BAND_GAP } else { 0.0 };
                places.push(Place {
                    x: margin + INSET * count(index),
                    y,
                    width: body - 2.0 * INSET * count(index),
                    height: heights[index],
                    rows: item_rows,
                    items_top: y + head,
                });
                y += head + items + gap;
            }
            top + heights[0] + BOTTOM
        }
    };
    Plan {
        places,
        width,
        height,
        note_x,
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let layers = layers(spec);
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let width = f64::from(spec.width);
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let any_note = layers.layers.iter().any(|layer| layer.note.is_some());
    let side = any_note && !compact;
    let body = width - 2.0 * margin - if side { NOTE_COLUMN + NOTE_GAP } else { 0.0 };
    let note_width = if side { NOTE_COLUMN } else { body - 2.0 * PAD };
    let measured = measure(layers, body - 2.0 * PAD, note_width, metrics, warnings);
    let plan = plan(spec, layers, &measured, (margin, top, side));
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        plan.width - 2.0 * margin,
        metrics,
        warnings,
    );
    warn_growth_advising(
        spec,
        (plan.width, plan.height),
        "shorten labels or use fewer items",
        warnings,
    );
    let nested = layers.display == LayersDisplay::Nested;
    for (index, layer) in layers.layers.iter().enumerate() {
        let place = &plan.places[index];
        let mut classes = focus_classes(Some(index), &[index]);
        if let Some(role) = layer.role {
            classes.push_str(roles::class(role));
        }
        elements.push(Element::NodeGroup(classes, layer.id.clone()));
        let class = if nested && index % 2 == 1 {
            "chartlet-flow-node chartlet-role-blue chartlet-layers-odd"
        } else {
            "chartlet-flow-node chartlet-role-blue"
        };
        let tooltip = layer.note.as_ref().map_or_else(
            || layer.label.clone(),
            |note| format!("{}: {note}", layer.label),
        );
        diagram::with_shadow(
            Element::Rect(Rect {
                x: place.x,
                y: place.y,
                width: place.width,
                height: place.height,
                class,
                series_index: None,
                style_index: None,
                tooltip: Some(tooltip),
            }),
            &mut elements,
        );
        elements.push(Element::Text(Text {
            x: place.x + PAD,
            y: place.y + PAD + 12.0,
            class: "chartlet-flow-label",
            anchor: TextAnchor::Start,
            content: measured[index].label.clone(),
        }));
        let mut note_top = place.y + PAD + LABEL_LINE + 11.0;
        if plan.note_x.is_none() {
            for line in &measured[index].note {
                elements.push(Element::Text(Text {
                    x: place.x + PAD,
                    y: note_top,
                    class: "chartlet-flow-sublabel",
                    anchor: TextAnchor::Start,
                    content: line.clone(),
                }));
                note_top += NOTE_LINE;
            }
        }
        for (row_index, row) in place.rows.iter().enumerate() {
            let y = place.items_top + (CHIP_HEIGHT + CHIP_GAP) * count(row_index);
            let mut x = place.x + PAD;
            for &chip in row {
                let (name, chip_width) = &measured[index].chips[chip];
                elements.push(Element::Rect(Rect {
                    x,
                    y,
                    width: *chip_width,
                    height: CHIP_HEIGHT,
                    class: "chartlet-layers-item",
                    series_index: None,
                    style_index: None,
                    tooltip: None,
                }));
                elements.push(Element::Text(Text {
                    x: x + chip_width / 2.0,
                    y: y + 16.0,
                    class: "chartlet-layers-item-text",
                    anchor: TextAnchor::Middle,
                    content: name.clone(),
                }));
                x += chip_width + CHIP_GAP;
            }
        }
        if let Some(role) = layer.role {
            roles::push_badge(
                role,
                (place.x + place.width - 12.0, place.y + 12.0),
                &mut elements,
            );
        }
        elements.push(Element::GroupEnd);
        if let Some(note_x) = plan.note_x
            && !measured[index].note.is_empty()
        {
            let y = place.y + PAD + 12.0;
            elements.push(Element::Line(Line {
                x1: place.x + place.width,
                y1: y - 4.0,
                x2: note_x - 8.0,
                y2: y - 4.0,
                class: "chartlet-layers-leader",
            }));
            for (line_index, line) in measured[index].note.iter().enumerate() {
                elements.push(Element::Text(Text {
                    x: note_x,
                    y: y + NOTE_LINE * count(line_index),
                    class: "chartlet-layers-note",
                    anchor: TextAnchor::Start,
                    content: line.clone(),
                }));
            }
        }
        elements.push(Element::Hotspot(Hotspot {
            node: index,
            label: layer.label.clone(),
            x: place.x,
            y: place.y,
            width: place.width,
            height: place.height,
        }));
    }
    let mut scene = Scene {
        width: pixels(plan.width),
        height: pixels(plan.height),
        elements,
    };
    let used = roles::used(
        layers.layers.iter().map(|layer| layer.role),
        std::iter::empty(),
    );
    roles::push_legend(
        &mut scene,
        spec.locale,
        &used,
        (margin, roles::push_badge),
        metrics,
    );
    scene
}

/// `number label (items…; note)` for the description.
fn entry(spec: &ChartSpec, index: usize) -> String {
    let layer = &layers(spec).layers[index];
    text::layers_entry(
        spec.locale,
        (index + 1, &layer.label),
        &layer.items,
        layer.note.as_deref(),
    )
}

/// The layers in sentences, in the order of the specification, and the roles.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let layers = layers(spec);
    let entries: Vec<String> = (0..layers.layers.len())
        .map(|index| entry(spec, index))
        .collect();
    let mut description = text::layers_opening(
        spec.locale,
        layers.display == LayersDisplay::Nested,
        &entries,
    );
    let named: Vec<(DiagramRole, String)> = layers
        .layers
        .iter()
        .filter_map(|layer| layer.role.map(|role| (role, layer.label.clone())))
        .collect();
    if let Some(sentence) = roles::sentence(spec.locale, &named) {
        description.push(' ');
        description.push_str(&sentence);
    }
    description
}

/// One row per layer in the order of the specification: its level, label, items, note and role,
/// where the diagram has them.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let layers = layers(spec);
    let words = spec.locale.words();
    let items = layers.layers.iter().any(|layer| !layer.items.is_empty());
    let notes = layers.layers.iter().any(|layer| layer.note.is_some());
    let with_roles = layers.layers.iter().any(|layer| layer.role.is_some());
    let mut columns = vec![
        words.level.to_owned(),
        if layers.display == LayersDisplay::Nested {
            words.scope.to_owned()
        } else {
            words.layer.to_owned()
        },
    ];
    if items {
        columns.push(words.items.to_owned());
    }
    if notes {
        columns.push(words.notes.to_owned());
    }
    if with_roles {
        columns.push(words.role.to_owned());
    }
    let rows = layers
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let mut row = vec![(index + 1).to_string(), layer.label.clone()];
            if items {
                row.push(layer.items.join(", "));
            }
            if notes {
                row.push(layer.note.clone().unwrap_or_default());
            }
            if with_roles {
                row.push(
                    layer
                        .role
                        .map_or_else(String::new, |role| text::role(spec.locale, role).to_owned()),
                );
            }
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BuiltinMetrics, ChartSpec, RenderFormat, RenderOptions, render_json,
        scene::{Element, Hotspot, Scene},
        text_alternative,
    };

    const STACK: &str = r#"{
        "schemaVersion": 1,
        "type": "layers",
        "title": "A stack",
        "layers": {
            "layers": [
                {"id": "hardware", "label": "Hardware", "items": ["CPU", "Memory"], "note": "Rented."},
                {"id": "system", "label": "System"},
                {"id": "apps", "label": "Applications", "items": ["API"], "role": "critical"}
            ]
        }
    }"#;

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    fn scene(json: &str) -> Scene {
        let spec = ChartSpec::from_json(json).expect("parses");
        let mut warnings = Vec::new();
        crate::layout::layout(&spec, &mut warnings, &BuiltinMetrics)
    }

    fn hotspots(scene: &Scene) -> Vec<Hotspot> {
        scene
            .elements
            .iter()
            .filter_map(|element| match element {
                Element::Hotspot(hotspot) => Some(hotspot.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_description_reads_a_stack_from_the_bottom() {
        let alternative =
            text_alternative(&ChartSpec::from_json(STACK).expect("parses")).expect("valid");
        assert_eq!(
            alternative.description,
            "Layer stack of 3 layers, from the bottom: 1 Hardware (items: CPU and Memory; note: Rented.); 2 System; 3 Applications (items: API). Roles: critical: Applications."
        );
        assert_eq!(
            alternative.table.columns,
            ["Level", "Layer", "Items", "Notes", "Role"]
        );
        assert_eq!(
            alternative.table.rows[0],
            ["1", "Hardware", "CPU, Memory", "Rented.", ""]
        );
        assert_eq!(alternative.table.rows[2][4], "critical");
    }

    #[test]
    fn nested_scopes_read_from_the_outermost() {
        let json = STACK.replace("\"layers\": {", "\"layers\": {\"display\": \"nested\",");
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .starts_with("3 nested scopes, from the outermost to the innermost: 1 Hardware"),
            "{}",
            alternative.description
        );
        assert_eq!(alternative.table.columns[1], "Scope");
    }

    #[test]
    fn a_stack_stands_on_its_first_layer_and_its_bands_keep_apart() {
        let boxes = hotspots(&scene(STACK));
        assert_eq!(boxes.len(), 3);
        // The first layer is the lowest on the page; no band reaches into the one above.
        for pair in boxes.windows(2) {
            assert!(pair[0].y >= pair[1].y + pair[1].height, "{pair:?}");
        }
    }

    #[test]
    fn nested_scopes_lie_inside_each_other() {
        let json = STACK.replace("\"layers\": {", "\"layers\": {\"display\": \"nested\",");
        let boxes = hotspots(&scene(&json));
        assert_eq!(boxes.len(), 3);
        for pair in boxes.windows(2) {
            let (outer, inner) = (&pair[0], &pair[1]);
            assert!(inner.x > outer.x && inner.y > outer.y, "{pair:?}");
            assert!(inner.x + inner.width < outer.x + outer.width, "{pair:?}");
            assert!(inner.y + inner.height < outer.y + outer.height, "{pair:?}");
        }
    }

    #[test]
    fn items_wrap_onto_rows_that_fit_their_layer() {
        let items: Vec<String> = (0..12)
            .map(|index| format!("\"Component {index}\""))
            .collect();
        let json = format!(
            r#"{{"schemaVersion": 1, "type": "layers", "title": "T", "width": 400, "layers": {{"layers": [{{"id": "a", "label": "A", "items": [{}]}}]}}}}"#,
            items.join(",")
        );
        let scene = scene(&json);
        let width = f64::from(scene.width);
        for element in &scene.elements {
            if let Element::Rect(rect) = element
                && rect.class == "chartlet-layers-item"
            {
                assert!(rect.x + rect.width <= width, "{rect:?}");
            }
        }
    }

    #[test]
    fn the_output_is_deterministic_and_german_texts_follow_the_locale() {
        let render = || {
            render_json(STACK, RenderFormat::Svg, &RenderOptions::default())
                .expect("renders")
                .content
        };
        assert_eq!(render(), render());
        let json = STACK.replace(
            "\"type\": \"layers\"",
            "\"type\": \"layers\", \"locale\": \"de\"",
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(alternative.description.starts_with("Schichtenstapel aus 3 Schichten, von unten nach oben: 1 Hardware (Elemente: CPU und Memory; Hinweis: Rented.)"));
    }

    #[test]
    fn invalid_layers_name_the_field() {
        assert_eq!(
            error(r#"{"schemaVersion": 1, "type": "layers", "title": "T"}"#),
            ("missing_layers", "/layers".into())
        );
        assert_eq!(
            error(&STACK.replace("\"system\"", "\"hardware\"")),
            ("duplicate_id", "/layers/layers/1/id".into())
        );
        let many: Vec<String> = (0..11)
            .map(|index| format!(r#"{{"id": "l{index}", "label": "L"}}"#))
            .collect();
        let json = format!(
            r#"{{"schemaVersion": 1, "type": "layers", "title": "T", "layers": {{"layers": [{}]}}}}"#,
            many.join(",")
        );
        assert_eq!(error(&json), ("too_many_layers", "/layers/layers".into()));
        let items: Vec<String> = (0..13).map(|index| format!("\"i{index}\"")).collect();
        let json = format!(
            r#"{{"schemaVersion": 1, "type": "layers", "title": "T", "layers": {{"layers": [{{"id": "a", "label": "A", "items": [{}]}}]}}}}"#,
            items.join(",")
        );
        assert_eq!(
            error(&json),
            ("too_many_items", "/layers/layers/0/items".into())
        );
        assert_eq!(
            error(&STACK.replace("\"type\": \"layers\"", "\"type\": \"bar\"")),
            ("option_not_supported", "/layers".into())
        );
    }
}
