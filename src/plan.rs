//! Plans: rooms, buildings and open ground on a grid, seen at an angle in 2:1 dimetric
//! projection. Walls are cut at one low height, so that every room shows; the walls at the back
//! show their inner face, the ones in front their outer face. Furniture from a small set stands
//! as low boxes. Labels stand in columns at both sides on leader lines; the phases of the areas
//! have patterns of their own, named in a legend. Every corner comes from [`crate::axo::project`].

use crate::{
    DataTable,
    axo::{self, project},
    diagram::{self, pixels, warn_growth_advising},
    error::ChartWarning,
    flow::focus_classes,
    layout::{NARROW, count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Hotspot, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{AreaKind, AreaSpec, ChartSpec, ItemKind, PlanSpec},
    text,
};

const MARGIN: f64 = 24.0;
const COMPACT_MARGIN: f64 = 12.0;
const BOTTOM: f64 = 16.0;
/// How high every wall is cut, in pixels.
const WALL: f64 = 14.0;
const COLUMN: f64 = 170.0;
const COLUMN_GAP: f64 = 22.0;
const LINE: f64 = 15.0;
const LEGEND_ROW: f64 = 22.0;
const MIN_UNIT: f64 = 8.0;
const MAX_UNIT: f64 = 44.0;

fn plan(spec: &ChartSpec) -> &PlanSpec {
    spec.plan
        .as_ref()
        .expect("validated plans carry a plan block")
}

/// The grid the areas cover: how far they reach along `x` and along `y`.
fn extent(plan: &PlanSpec) -> (f64, f64) {
    plan.areas.iter().fold((1.0_f64, 1.0_f64), |(x, y), area| {
        (x.max(area.x + area.w), y.max(area.y + area.h))
    })
}

/// The order the areas are drawn in, back to front: open ground first, then by how far from the
/// back corner each one begins.
fn drawing_order(plan: &PlanSpec) -> Vec<usize> {
    let mut order: Vec<usize> = (0..plan.areas.len()).collect();
    order.sort_by(|&a, &b| {
        let (one, other) = (&plan.areas[a], &plan.areas[b]);
        (one.kind != AreaKind::Outdoor)
            .cmp(&(other.kind != AreaKind::Outdoor))
            .then((one.x + one.y).total_cmp(&(other.x + other.y)))
            .then(one.x.total_cmp(&other.x))
            .then(a.cmp(&b))
    });
    order
}

/// The line under a label: the kind, the size and the phase.
fn detail(spec: &ChartSpec, area: &AreaSpec) -> String {
    text::plan_detail(
        spec.locale,
        area.kind,
        (
            &crate::layout::format_value(area.w, spec.number_style()),
            &crate::layout::format_value(area.h, spec.number_style()),
        ),
        area.phase.as_deref(),
    )
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

const PHASE_CLASSES: [&str; 4] = [
    "",
    "chartlet-plan-phase-1",
    "chartlet-plan-phase-2",
    "chartlet-plan-phase-3",
];

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let plan = plan(spec);
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    // A narrow canvas has no room for columns of labels: the areas carry numbers and a key below
    // the plan names them.
    let column = if compact { 0.0 } else { COLUMN };
    let column_gap = if compact { 0.0 } else { COLUMN_GAP };
    let key_height = if compact {
        (LINE + 5.0) * count(plan.areas.len()) + 8.0
    } else {
        0.0
    };
    let width = f64::from(spec.width);
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let phases = plan.phases();
    let legend_height = if phases.is_empty() {
        0.0
    } else {
        LEGEND_ROW + 6.0
    };
    let (grid_x, grid_y) = extent(plan);
    let tallest = plan
        .areas
        .iter()
        .flat_map(|area| area.items.iter().map(|item| item.kind.size().2))
        .fold(WALL, f64::max);
    // The grid as large as the canvas lets it be, between the two columns of labels.
    let room_width = width - 2.0 * margin - 2.0 * (column + column_gap);
    let room_height = f64::from(spec.height) - top - BOTTOM - legend_height - key_height - tallest;
    let unit = (room_width / (grid_x + grid_y))
        .min(room_height / f64::midpoint(grid_x, grid_y))
        .clamp(MIN_UNIT, MAX_UNIT);
    let drawn_width = (grid_x + grid_y) * unit;
    let drawn_height = (grid_x + grid_y) * unit / 2.0 + tallest;
    let page_width = width.max(2.0 * (margin + column + column_gap) + drawn_width);
    // The plan takes the height it needs, and the columns hold their labels in rows of two lines.
    let page_height = (top + drawn_height + BOTTOM + legend_height + key_height).max(
        top + (2.0 * LINE + 6.0) * count(plan.areas.len()) / 2.0 * f64::from(u8::from(!compact))
            + BOTTOM
            + legend_height,
    );
    let body_height = page_height - top - BOTTOM - legend_height - key_height;
    // The point `(0, 0)` of the grid on the page: the back corner, so that the left corner lies
    // `grid_y` units to the left of it.
    let origin = (
        page_width / 2.0 + (grid_y - grid_x) * unit / 2.0,
        top + tallest + (body_height - drawn_height) / 2.0,
    );
    let at = |(x, y): (f64, f64)| (origin.0 + x, origin.1 + y);
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        page_width - 2.0 * margin,
        metrics,
        warnings,
    );
    warn_growth_advising(
        spec,
        (page_width, page_height),
        "shorten labels or use fewer areas",
        warnings,
    );
    let order = drawing_order(plan);
    // Where each label points to: the middle of the area's floor.
    let mut anchors: Vec<(f64, f64)> = vec![(0.0, 0.0); plan.areas.len()];
    let mut boxes: Vec<(f64, f64, f64, f64)> = vec![(0.0, 0.0, 0.0, 0.0); plan.areas.len()];
    for &index in &order {
        let area = &plan.areas[index];
        let related = [index];
        elements.push(Element::NodeGroup(
            focus_classes(Some(index), &related),
            area.id.clone(),
        ));
        let rectangle = (area.x, area.y, area.w, area.h);
        let floor: Vec<(f64, f64)> = axo::rectangle(unit, rectangle, 0.0)
            .into_iter()
            .map(at)
            .collect();
        let kind_class = match area.kind {
            AreaKind::Room => "chartlet-plan-floor chartlet-plan-room",
            AreaKind::Building => "chartlet-plan-floor chartlet-plan-building",
            AreaKind::Outdoor => "chartlet-plan-floor chartlet-plan-outdoor",
        };
        let tooltip = format!("{}: {}", area.label, detail(spec, area));
        elements.push(polygon(axo::closed(&floor), kind_class, Some(tooltip)));
        if let Some(phase) = plan.phase_of(index).filter(|&phase| phase > 0) {
            elements.push(polygon(axo::closed(&floor), PHASE_CLASSES[phase], None));
        }
        let (x0, y0, x1, y1) = (area.x, area.y, area.x + area.w, area.y + area.h);
        let walled = area.kind != AreaKind::Outdoor;
        let edge = |from: (f64, f64), to: (f64, f64), z: f64| {
            vec![
                at(project(unit, from.0, from.1, 0.0)),
                at(project(unit, to.0, to.1, 0.0)),
                at(project(unit, to.0, to.1, z)),
                at(project(unit, from.0, from.1, z)),
            ]
        };
        if walled {
            // The walls at the back show their inner face.
            elements.push(polygon(
                axo::closed(&edge((x0, y0), (x1, y0), WALL)),
                "chartlet-plan-wall-in",
                None,
            ));
            elements.push(polygon(
                axo::closed(&edge((x0, y0), (x0, y1), WALL)),
                "chartlet-plan-wall-in",
                None,
            ));
        }
        for item in &area.items {
            let (item_width, item_depth, item_height) = item.kind.size();
            let faces = axo::faces(
                unit,
                (area.x + item.x, area.y + item.y, item_width, item_depth),
                (0.0, item_height),
            );
            let moved = |outline: Vec<(f64, f64)>| outline.into_iter().map(at).collect::<Vec<_>>();
            let (left, right) = (moved(faces.left), moved(faces.right));
            elements.push(polygon(left.clone(), "chartlet-plan-item", None));
            elements.push(polygon(left, "chartlet-plan-shade-left", None));
            elements.push(polygon(right.clone(), "chartlet-plan-item", None));
            elements.push(polygon(right, "chartlet-plan-shade-right", None));
            elements.push(polygon(moved(faces.top), "chartlet-plan-item", None));
        }
        if walled {
            // The walls in front show their outer face.
            let front_left = edge((x0, y1), (x1, y1), WALL);
            let front_right = edge((x1, y0), (x1, y1), WALL);
            elements.push(polygon(
                axo::closed(&front_left),
                "chartlet-plan-wall",
                None,
            ));
            elements.push(polygon(
                axo::closed(&front_right),
                "chartlet-plan-wall",
                None,
            ));
            elements.push(polygon(
                axo::closed(&front_left),
                "chartlet-plan-shade-left",
                None,
            ));
            elements.push(polygon(
                axo::closed(&front_right),
                "chartlet-plan-shade-right",
                None,
            ));
            let top_outline: Vec<(f64, f64)> = axo::closed(&axo::rectangle(unit, rectangle, WALL))
                .into_iter()
                .map(at)
                .collect();
            elements.push(polygon(
                top_outline,
                if area.kind == AreaKind::Building {
                    "chartlet-plan-rim chartlet-plan-rim-building"
                } else {
                    "chartlet-plan-rim"
                },
                None,
            ));
        }
        // A label points to the middle of a room, and to the front corner of open ground, which
        // other areas may cover in the middle.
        let (anchor_x, anchor_y) = if walled {
            (area.x + area.w / 2.0, area.y + area.h / 2.0)
        } else {
            (
                area.x + area.w - 0.5_f64.min(area.w / 2.0),
                area.y + area.h - 0.5_f64.min(area.h / 2.0),
            )
        };
        let middle = at(project(
            unit,
            anchor_x,
            anchor_y,
            if walled { WALL } else { 0.0 },
        ));
        anchors[index] = middle;
        let all: Vec<(f64, f64)> = [0.0, WALL]
            .into_iter()
            .flat_map(|z| axo::rectangle(unit, rectangle, z))
            .map(at)
            .collect();
        let left = all.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let right = all.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let upper = all.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let lower = all.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        boxes[index] = (left, upper - tallest + WALL, right, lower);
        elements.push(Element::GroupEnd);
    }
    // The labels: those of areas left of the middle stand in the left column, the others in the
    // right one, each as high as its area lets it and lower where one above would be covered.
    let middle_x = page_width / 2.0;
    let mut sides: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
    for &index in &order {
        sides[usize::from(anchors[index].0 >= middle_x)].push(index);
    }
    if compact {
        let first = page_height - BOTTOM - legend_height - key_height + 18.0;
        for (index, area) in plan.areas.iter().enumerate() {
            let row = first + (LINE + 5.0) * count(index);
            elements.push(Element::Group(focus_classes(None, &[index])));
            diagram::badge(index + 1, anchors[index], &mut elements);
            diagram::badge(index + 1, (margin + 9.0, row - 4.0), &mut elements);
            let label = fit_text(
                &area.label,
                page_width - 2.0 * margin - 24.0,
                13.0,
                metrics,
                warnings,
                &format!("/plan/areas/{index}/label"),
            );
            let after = margin + 24.0 + metrics.width(&label, 13.0) + 8.0;
            let line = fit_text(
                &detail(spec, area),
                (page_width - margin - after).max(0.0),
                11.0,
                metrics,
                warnings,
                &format!("/plan/areas/{index}/kind"),
            );
            elements.push(Element::Text(Text {
                x: margin + 24.0,
                y: row,
                class: "chartlet-flow-label",
                anchor: TextAnchor::Start,
                content: label,
            }));
            elements.push(Element::Text(Text {
                x: after,
                y: row,
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Start,
                content: line,
            }));
            elements.push(Element::GroupEnd);
        }
        sides = [Vec::new(), Vec::new()];
    }
    for (side, members) in sides.iter_mut().enumerate() {
        members.sort_by(|&a, &b| anchors[a].1.total_cmp(&anchors[b].1).then(a.cmp(&b)));
        let (text_x, anchor) = if side == 0 {
            (margin + column, TextAnchor::End)
        } else {
            (page_width - margin - column, TextAnchor::Start)
        };
        let mut floor = top;
        for &index in members.iter() {
            let area = &plan.areas[index];
            let label = fit_text(
                &area.label,
                column,
                13.0,
                metrics,
                warnings,
                &format!("/plan/areas/{index}/label"),
            );
            let line = fit_text(
                &detail(spec, area),
                column,
                11.0,
                metrics,
                warnings,
                &format!("/plan/areas/{index}/kind"),
            );
            let first = (anchors[index].1 + 4.0).max(floor + 11.0);
            floor = first + LINE + 6.0;
            // The leader ends beside the label, on the side of the plan.
            let tip = (
                if side == 0 {
                    text_x + 6.0
                } else {
                    text_x - 6.0
                },
                first - 4.0,
            );
            elements.push(Element::Group(focus_classes(None, &[index])));
            elements.push(Element::Polyline(diagram::polyline(
                vec![anchors[index], (anchors[index].0, first - 4.0), tip],
                "chartlet-plan-leader",
                None,
            )));
            elements.push(Element::Circle(Circle {
                cx: anchors[index].0,
                cy: anchors[index].1,
                radius: 2.5,
                class: "chartlet-plan-dot",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            elements.push(Element::Text(Text {
                x: text_x,
                y: first,
                class: "chartlet-flow-label",
                anchor,
                content: label,
            }));
            elements.push(Element::Text(Text {
                x: text_x,
                y: first + LINE - 2.0,
                class: "chartlet-flow-sublabel",
                anchor,
                content: line,
            }));
            elements.push(Element::GroupEnd);
        }
    }
    for &index in &order {
        let (left, upper, right, lower) = boxes[index];
        elements.push(Element::Hotspot(Hotspot {
            node: index,
            label: plan.areas[index].label.clone(),
            x: left,
            y: upper,
            width: right - left,
            height: lower - upper,
        }));
    }
    // The legend of the phases, each swatch in the pattern of its phase.
    let legend_top = page_height - BOTTOM - legend_height + 6.0;
    let mut x = margin;
    for (index, phase) in phases.iter().enumerate() {
        for class in ["chartlet-plan-swatch", PHASE_CLASSES[index]] {
            if class.is_empty() {
                continue;
            }
            elements.push(Element::Rect(Rect {
                x,
                y: legend_top,
                width: 26.0,
                height: 14.0,
                class: if class == "chartlet-plan-swatch" {
                    "chartlet-plan-swatch"
                } else {
                    PHASE_CLASSES[index]
                },
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
        elements.push(Element::Text(Text {
            x: x + 34.0,
            y: legend_top + 11.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: (*phase).to_owned(),
        }));
        x += 34.0 + metrics.width(phase, 12.0) + 24.0;
    }
    Scene {
        width: pixels(page_width),
        height: pixels(page_height),
        elements,
    }
}

/// The plan in sentences: its size on the grid, every area in the order of the specification
/// with its kind, size, phase and items, and what each phase holds.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let plan = plan(spec);
    let locale = spec.locale;
    let (grid_x, grid_y) = extent(plan);
    let style = spec.number_style();
    let entries: Vec<String> = plan
        .areas
        .iter()
        .map(|area| {
            text::plan_area(
                locale,
                &area.label,
                area.kind,
                (
                    &crate::layout::format_value(area.w, style),
                    &crate::layout::format_value(area.h, style),
                ),
                area.phase.as_deref(),
                &area
                    .items
                    .iter()
                    .map(|item| item.kind)
                    .collect::<Vec<ItemKind>>(),
            )
        })
        .collect();
    let mut description = text::plan_opening(
        locale,
        plan.areas.len(),
        (
            &crate::layout::format_value(grid_x, style),
            &crate::layout::format_value(grid_y, style),
        ),
        &entries,
    );
    let phases = plan.phases();
    if !phases.is_empty() {
        let held: Vec<(String, Vec<String>)> = phases
            .iter()
            .map(|phase| {
                (
                    (*phase).to_owned(),
                    plan.areas
                        .iter()
                        .filter(|area| area.phase.as_deref() == Some(phase))
                        .map(|area| area.label.clone())
                        .collect(),
                )
            })
            .collect();
        description.push(' ');
        description.push_str(&text::plan_phases(locale, &held));
    }
    description
}

/// One row per area in the order of the specification: label, kind, position and size on the
/// grid, phase and items.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let plan = plan(spec);
    let words = spec.locale.words();
    let style = spec.number_style();
    let phases = !plan.phases().is_empty();
    let items = plan.areas.iter().any(|area| !area.items.is_empty());
    let mut columns = vec![
        words.area.to_owned(),
        words.kind.to_owned(),
        words.position.to_owned(),
        words.size.to_owned(),
    ];
    if phases {
        columns.push(words.phase.to_owned());
    }
    if items {
        columns.push(words.items.to_owned());
    }
    let rows = plan
        .areas
        .iter()
        .map(|area| {
            let mut row = vec![
                area.label.clone(),
                text::area_kind(spec.locale, area.kind).to_owned(),
                format!(
                    "{}, {}",
                    crate::layout::format_value(area.x, style),
                    crate::layout::format_value(area.y, style)
                ),
                format!(
                    "{} × {}",
                    crate::layout::format_value(area.w, style),
                    crate::layout::format_value(area.h, style)
                ),
            ];
            if phases {
                row.push(area.phase.clone().unwrap_or_default());
            }
            if items {
                row.push(
                    area.items
                        .iter()
                        .map(|item| text::item_kind(spec.locale, item.kind))
                        .collect::<Vec<_>>()
                        .join(", "),
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
        ChartSpec, RenderFormat, RenderOptions, axo::project, render_json, text_alternative,
    };

    const PLAN: &str = r#"{
        "schemaVersion": 1,
        "type": "plan",
        "title": "A floor",
        "plan": {
            "areas": [
                {"id": "office", "label": "Office", "x": 0, "y": 0, "w": 6, "h": 4, "phase": "Existing",
                 "items": [{"kind": "desk", "x": 0.5, "y": 0.5}, {"kind": "shelf", "x": 3, "y": 0.2}]},
                {"id": "lab", "label": "Lab", "x": 6, "y": 0, "w": 3, "h": 4, "phase": "Phase 1"},
                {"id": "yard", "label": "Yard", "kind": "outdoor", "x": 0, "y": 0, "w": 9.5, "h": 6, "phase": "Phase 2"}
            ]
        }
    }"#;

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    fn svg(json: &str) -> String {
        render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect("renders")
            .content
    }

    #[test]
    fn the_projection_is_two_to_one_dimetric() {
        // One grid unit along x moves 1 unit right and a half unit down; along y left and down; z
        // moves straight up.
        assert_eq!(project(10.0, 1.0, 0.0, 0.0), (10.0, 5.0));
        assert_eq!(project(10.0, 0.0, 1.0, 0.0), (-10.0, 5.0));
        assert_eq!(project(10.0, 0.0, 0.0, 14.0), (0.0, -14.0));
    }

    #[test]
    fn the_description_and_table_list_every_area_with_size_and_phase() {
        let alternative =
            text_alternative(&ChartSpec::from_json(PLAN).expect("parses")).expect("valid");
        assert_eq!(
            alternative.description,
            "Plan with 3 areas on a grid of 9.5 × 6 units, seen at an angle: Office (room, 6 × 4, Existing, with desk and shelf); Lab (room, 3 × 4, Phase 1); Yard (outdoor, 9.5 × 6, Phase 2). Phases: Existing: Office; Phase 1: Lab; Phase 2: Yard."
        );
        assert_eq!(
            alternative.table.columns,
            ["Area", "Kind", "Position", "Size", "Phase", "Items"]
        );
        assert_eq!(
            alternative.table.rows[0],
            ["Office", "room", "0, 0", "6 × 4", "Existing", "desk, shelf"]
        );
        assert_eq!(alternative.table.rows.len(), 3);
    }

    #[test]
    fn phases_have_patterns_and_a_legend_so_color_is_not_the_only_cue() {
        let svg = svg(PLAN);
        for phase in 1..=2 {
            assert!(svg.contains(&format!("-plan-phase-{phase}\"")), "{phase}");
            assert!(svg.contains("fill=\"url(#chartlet-"), "fill by pattern");
        }
        assert!(svg.contains(">Existing</text>") && svg.contains(">Phase 2</text>"));
        // Without phases there is nothing to tell apart: no pattern, no legend.
        assert!(!svg_without_phases().contains("<pattern"));
    }

    fn svg_without_phases() -> String {
        svg(&PLAN
            .replace(", \"phase\": \"Existing\"", "")
            .replace(", \"phase\": \"Phase 1\"", "")
            .replace(", \"phase\": \"Phase 2\"", ""))
    }

    #[test]
    fn labels_stand_on_leader_lines_and_walls_are_cut_low() {
        let svg = svg(PLAN);
        assert!(svg.contains("chartlet-plan-leader"));
        assert!(svg.contains("chartlet-plan-wall-in"));
        assert!(svg.contains("chartlet-plan-item"));
        // Open ground has no walls: the yard's floor is the only outdoor shape.
        assert_eq!(
            svg.matches("class=\"chartlet-plan-floor chartlet-plan-outdoor\"")
                .count(),
            1
        );
    }

    #[test]
    fn invalid_plans_name_the_field() {
        assert_eq!(
            error(r#"{"schemaVersion": 1, "type": "plan", "title": "T"}"#),
            ("missing_plan", "/plan".into())
        );
        assert_eq!(
            error(&PLAN.replace(
                "\"x\": 6, \"y\": 0, \"w\": 3",
                "\"x\": 5, \"y\": 0, \"w\": 3"
            )),
            ("area_overlap", "/plan/areas/1".into())
        );
        assert_eq!(
            error(&PLAN.replace("\"w\": 3, \"h\": 4", "\"w\": 0, \"h\": 4")),
            ("invalid_area", "/plan/areas/1/w".into())
        );
        assert_eq!(
            error(&PLAN.replace("\"x\": 3, \"y\": 0.2", "\"x\": 5.5, \"y\": 0.2")),
            ("item_outside_area", "/plan/areas/0/items/1".into())
        );
        assert_eq!(
            error(&PLAN.replace("\"lab\"", "\"office\"")),
            ("duplicate_id", "/plan/areas/1/id".into())
        );
        let phases = PLAN
            .replace("\"Phase 1\"", "\"B\"")
            .replace("\"Phase 2\"", "\"C\"")
            .replace(
                "{\"id\": \"lab\"",
                "{\"id\": \"extra\", \"label\": \"Extra\", \"x\": 20, \"y\": 0, \"w\": 1, \"h\": 1, \"phase\": \"D\"},\n                {\"id\": \"extra2\", \"label\": \"Extra\", \"x\": 22, \"y\": 0, \"w\": 1, \"h\": 1, \"phase\": \"E\"},\n                {\"id\": \"lab\"",
            );
        assert_eq!(
            error(&phases),
            ("too_many_phases", "/plan/areas/4/phase".into())
        );
    }

    #[test]
    fn the_output_is_deterministic_and_works_at_phone_width() {
        let narrow = PLAN.replace(
            "\"title\": \"A floor\"",
            "\"title\": \"A floor\", \"width\": 360",
        );
        assert_eq!(svg(&narrow), svg(&narrow));
        assert!(svg(&narrow).contains("chartlet-plan-floor"));
    }
}
