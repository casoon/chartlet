//! The exploded view of a layer stack: the layers as slabs in 2:1 dimetric projection, lifted
//! apart at equal gaps, the first at the bottom, with dotted guides between their corners and the
//! labels at the side on leader lines. Every corner comes from [`crate::axo::project`]. On a
//! compact canvas the stack is drawn flat instead; the description is the same either way.

use crate::{
    axo::{self, project},
    diagram::{self, pixels, warn_growth_advising},
    error::ChartWarning,
    flow::focus_classes,
    layers::wrap_lines,
    layout::{count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    roles,
    scene::{Element, Hotspot, Polyline, Scene, Text, TextAnchor},
    spec::{ChartSpec, LayersSpec},
};

const MARGIN: f64 = 24.0;
const BOTTOM: f64 = 16.0;
/// The side of a slab's footprint in grid units, and how thick a slab is, in pixels.
const GRID: f64 = 4.0;
const THICKNESS: f64 = 14.0;
/// The least distance between the tops of two slabs, so that a layer's lower face and the
/// next one's top keep a gap of at least [`MIN_GAP`] between them.
const MIN_GAP: f64 = 30.0;
/// How far apart the slabs are lifted, as a share of the side of their footprint.
const LIFT_SHARE: f64 = 0.5;
const LABEL_COLUMN: f64 = 230.0;
const LABEL_GAP: f64 = 36.0;
const LINE: f64 = 15.0;
const MIN_SIZE: f64 = 90.0;
const MAX_SIZE: f64 = 170.0;

fn layers(spec: &ChartSpec) -> &LayersSpec {
    spec.layer_stack
        .as_ref()
        .expect("validated layer diagrams carry a layers block")
}

/// What a label block says: the label, then every item, then the note.
struct Block {
    label: String,
    items: Vec<String>,
    note: Vec<String>,
}

impl Block {
    fn height(&self) -> f64 {
        LINE * count(1 + self.items.len() + self.note.len())
    }
}

fn blocks(
    layers: &LayersSpec,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<Block> {
    layers
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let path = format!("/layers/layers/{index}");
            Block {
                label: fit_text(
                    &layer.label,
                    LABEL_COLUMN,
                    13.0,
                    metrics,
                    warnings,
                    &format!("{path}/label"),
                ),
                items: layer
                    .items
                    .iter()
                    .enumerate()
                    .map(|(item, name)| {
                        fit_text(
                            name,
                            LABEL_COLUMN - 14.0,
                            12.0,
                            metrics,
                            warnings,
                            &format!("{path}/items/{item}"),
                        )
                    })
                    .collect(),
                note: layer.note.as_ref().map_or_else(Vec::new, |note| {
                    wrap_lines(
                        note,
                        LABEL_COLUMN,
                        11.0,
                        3,
                        metrics,
                        warnings,
                        &format!("{path}/note"),
                    )
                }),
            }
        })
        .collect()
}

fn polygon(points: Vec<(f64, f64)>, class: &'static str, tooltip: Option<String>) -> Element {
    Element::Polyline(Polyline {
        points,
        class,
        series_index: None,
        style_index: None,
        tooltip,
    })
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let layers = layers(spec);
    let count_layers = layers.layers.len();
    let width = f64::from(spec.width);
    let top = 56.0 + title_extra(spec, width - 2.0 * MARGIN, metrics);
    let blocks = blocks(layers, metrics, warnings);
    let steps = count(count_layers.saturating_sub(1));
    // The footprint is as large as the width and the height let it be; the distance between the
    // tops of two slabs is half of it, so that a slab shows its front half under the one above.
    let room_width = width - 2.0 * MARGIN - LABEL_COLUMN - LABEL_GAP;
    let room_height = f64::from(spec.height) - top - BOTTOM - THICKNESS;
    let size = (room_width / 2.0)
        .min(room_height / (1.0 + LIFT_SHARE * steps))
        .clamp(MIN_SIZE, MAX_SIZE);
    let lift = (LIFT_SHARE * size).max(THICKNESS + MIN_GAP);
    let unit = size / GRID;
    let page_width = width.max(2.0 * MARGIN + 2.0 * size + LABEL_GAP + LABEL_COLUMN);
    // The stack hangs from the top: the back corner of the top slab's top face lies at `top`.
    // The stack and its labels stand centred on the canvas.
    let spare =
        ((page_width - 2.0 * MARGIN - 2.0 * size - LABEL_GAP - LABEL_COLUMN) / 2.0).max(0.0);
    let origin = (MARGIN + spare + size, top + lift * steps + THICKNESS);
    let at = |(x, y): (f64, f64)| (origin.0 + x, origin.1 + y);
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        MARGIN,
        page_width - 2.0 * MARGIN,
        metrics,
        warnings,
    );
    let footprint = (0.0, 0.0, GRID, GRID);
    // Dotted guides between the corners of neighbouring slabs.
    for index in 0..count_layers.saturating_sub(1) {
        let from = THICKNESS + lift * count(index);
        let to = lift * count(index + 1);
        for (x, y) in [(0.0, GRID), (GRID, GRID), (GRID, 0.0)] {
            elements.push(Element::Line(crate::scene::Line {
                x1: at(project(unit, x, y, from)).0,
                y1: at(project(unit, x, y, from)).1,
                x2: at(project(unit, x, y, to)).0,
                y2: at(project(unit, x, y, to)).1,
                class: "chartlet-layers-guide",
            }));
        }
    }
    let label_x = MARGIN + spare + 2.0 * size + LABEL_GAP;
    // The labels stand from the top down, each as high as its slab's right corner lets it, and
    // lower where the one above would be covered.
    let mut label_tops = vec![0.0; count_layers];
    let mut floor = top;
    for index in (0..count_layers).rev() {
        let corner = at(project(
            unit,
            GRID,
            0.0,
            lift * count(index) + THICKNESS / 2.0,
        ))
        .1;
        let wanted = corner - 8.0;
        let placed = wanted.max(floor);
        label_tops[index] = placed;
        floor = placed + blocks[index].height() + 10.0;
    }
    let page_height = (origin.1 + size).max(floor - 10.0) + BOTTOM;
    warn_growth_advising(
        spec,
        (page_width, page_height),
        "shorten labels or use fewer items",
        warnings,
    );
    for index in 0..count_layers {
        let layer = &layers.layers[index];
        let mut classes = focus_classes(Some(index), &[index]);
        if let Some(role) = layer.role {
            classes.push_str(roles::class(role));
        }
        elements.push(Element::NodeGroup(classes, layer.id.clone()));
        let z = lift * count(index);
        let faces = axo::faces(unit, footprint, (z, THICKNESS));
        let moved = |outline: Vec<(f64, f64)>| outline.into_iter().map(at).collect::<Vec<_>>();
        let tooltip = layer.note.as_ref().map_or_else(
            || layer.label.clone(),
            |note| format!("{}: {note}", layer.label),
        );
        let (side_left, side_right, side_top) =
            (moved(faces.left), moved(faces.right), moved(faces.top));
        elements.push(polygon(
            side_left.clone(),
            "chartlet-flow-node chartlet-role-blue",
            Some(tooltip),
        ));
        elements.push(polygon(side_left, "chartlet-layers-shade-left", None));
        elements.push(polygon(
            side_right.clone(),
            "chartlet-flow-node chartlet-role-blue",
            None,
        ));
        elements.push(polygon(side_right, "chartlet-layers-shade-right", None));
        elements.push(polygon(
            side_top.clone(),
            "chartlet-flow-node chartlet-role-blue",
            None,
        ));
        let corner = at(project(unit, GRID, 0.0, z + THICKNESS / 2.0));
        let first = label_tops[index] + 11.0;
        let tip = (label_x - 10.0, first - 4.0);
        let elbow = (corner.0 + 16.0, corner.1);
        elements.push(Element::Polyline(diagram::polyline(
            vec![corner, elbow, (elbow.0 + 8.0, tip.1), tip],
            "chartlet-layers-leader",
            None,
        )));
        let block = &blocks[index];
        let mut baseline = first;
        elements.push(Element::Text(Text {
            x: label_x,
            y: baseline,
            class: "chartlet-flow-label",
            anchor: TextAnchor::Start,
            content: block.label.clone(),
        }));
        for item in &block.items {
            baseline += LINE;
            elements.push(Element::Text(Text {
                x: label_x,
                y: baseline,
                class: "chartlet-layers-note",
                anchor: TextAnchor::Start,
                content: format!("· {item}"),
            }));
        }
        for line in &block.note {
            baseline += LINE;
            elements.push(Element::Text(Text {
                x: label_x,
                y: baseline,
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Start,
                content: line.clone(),
            }));
        }
        if let Some(role) = layer.role {
            roles::push_badge(
                role,
                (label_x - 18.0 - 6.0, first - 4.0 - 14.0),
                &mut elements,
            );
        }
        elements.push(Element::GroupEnd);
        let (low, high) = (
            at(project(unit, GRID, GRID, z)).1,
            at(project(unit, 0.0, 0.0, z + THICKNESS)).1,
        );
        elements.push(Element::Hotspot(Hotspot {
            node: index,
            label: layer.label.clone(),
            x: origin.0 - size,
            y: high,
            width: 2.0 * size,
            height: low - high,
        }));
    }
    let mut scene = Scene {
        width: pixels(page_width),
        height: pixels(page_height),
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
        (MARGIN, roles::push_badge),
        metrics,
    );
    scene
}

#[cfg(test)]
mod tests {
    use crate::{
        BuiltinMetrics, ChartSpec, RenderFormat, RenderOptions, render_json,
        scene::{Element, Hotspot},
        text_alternative,
    };

    const STACK: &str = r#"{
        "schemaVersion": 1,
        "type": "layers",
        "title": "Exploded",
        "width": 800,
        "height": 520,
        "layers": {
            "projection": "exploded",
            "layers": [
                {"id": "hardware", "label": "Hardware", "items": ["CPU", "Memory"], "note": "Rented."},
                {"id": "system", "label": "System"},
                {"id": "apps", "label": "Applications", "items": ["API"], "role": "critical"}
            ]
        }
    }"#;

    fn hotspots(json: &str) -> Vec<Hotspot> {
        let spec = ChartSpec::from_json(json).expect("parses");
        let mut warnings = Vec::new();
        crate::layout::layout(&spec, &mut warnings, &BuiltinMetrics)
            .elements
            .iter()
            .filter_map(|element| match element {
                Element::Hotspot(hotspot) => Some(hotspot.clone()),
                _ => None,
            })
            .collect()
    }

    fn render(json: &str) -> String {
        render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect("renders")
            .content
    }

    #[test]
    fn the_layers_are_lifted_apart_at_equal_gaps() {
        let boxes = hotspots(STACK);
        assert_eq!(boxes.len(), 3);
        // The first layer is the lowest; the tops of the slabs rise by one lift each.
        let first = boxes[0].y - boxes[1].y;
        let second = boxes[1].y - boxes[2].y;
        assert!(first > 0.0, "{boxes:?}");
        assert!((first - second).abs() < 1e-6, "{first} against {second}");
    }

    #[test]
    fn the_exploded_view_draws_slabs_guides_and_leader_lines() {
        let svg = render(STACK);
        for class in [
            "chartlet-layers-shade-left",
            "chartlet-layers-shade-right",
            "chartlet-layers-guide",
            "chartlet-layers-leader",
        ] {
            assert!(svg.contains(&format!("class=\"{class}\"")), "{class}");
        }
        for label in ["Hardware", "System", "Applications"] {
            assert!(svg.contains(&format!(">{label}</text>")), "{label}");
        }
        assert_eq!(svg, render(STACK), "the output is deterministic");
    }

    #[test]
    fn its_description_is_that_of_the_flat_stack() {
        let flat = STACK.replace("\"projection\": \"exploded\",", "");
        let describe = |json: &str| {
            text_alternative(&ChartSpec::from_json(json).expect("parses"))
                .expect("valid")
                .description
        };
        assert_eq!(describe(STACK), describe(&flat));
    }

    #[test]
    fn a_compact_canvas_falls_back_to_the_flat_stack() {
        let narrow = STACK.replace("\"width\": 800", "\"width\": 360");
        let svg = render(&narrow);
        assert!(!svg.contains("class=\"chartlet-layers-guide\""));
        assert!(svg.contains("class=\"chartlet-layers-item\""));
    }

    #[test]
    fn nested_scopes_cannot_be_exploded() {
        let json = STACK.replace("\"projection\"", "\"display\": \"nested\", \"projection\"");
        let error =
            render_json(&json, RenderFormat::Svg, &RenderOptions::default()).expect_err("invalid");
        assert_eq!(
            (error.code, error.path.as_str()),
            ("option_not_supported", "/layers/projection")
        );
    }
}
