//! Value chain maps: components placed by how visible they are to the user (up) and how far they
//! have evolved (right) in four stages, genesis, custom, product and commodity, with dependencies
//! as lines and planned movement as dashed arrows. The positions are the specification's, not
//! computed; only the labels move, to the side of the dot where they overlap nothing. A bar
//! across a component's way marks inertia. The description and the data table give every
//! component with its position, its stage, its movement and what it depends on.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{arrowhead, pixels, polyline, warn_growth_advising},
    draw::{circle, line, rect, text},
    error::ChartWarning,
    layout::{fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Scene, TextAnchor},
    spec::{ChartSpec, NumberStyle, ValueFormat, ValuemapSpec},
    text,
};

const SIZE: f64 = 12.0;
/// The radius of a dot, and how far the plot is inset so that a dot on the edge stays whole.
const DOT: f64 = 6.0;
const INSET: f64 = 14.0;

fn valuemap(spec: &ChartSpec) -> &ValuemapSpec {
    spec.valuemap
        .as_ref()
        .expect("validated value chain maps carry a valuemap block")
}

/// A place on the map as a percentage, with no decimals.
fn percent(spec: &ChartSpec, value: f64) -> String {
    format_value(
        value,
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(0),
            ..spec.number_style()
        },
    )
}

/// A box a label or a dot occupies: left, top, right, bottom.
type Area = (f64, f64, f64, f64);

fn overlap(a: Area, b: Area) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let map = valuemap(spec);
    let compact = spec.width < crate::layout::NARROW;
    let margin = if compact { 12.0 } else { 24.0 };
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let words = text::valuemap_words(spec.locale);
    let axis_left = margin + if compact { 44.0 } else { 62.0 };
    let (plot_left, plot_right) = (axis_left, width - margin);
    let (plot_top, plot_bottom) = (top + 14.0, height - 46.0);
    let plot_bottom = plot_bottom.max(plot_top + 120.0);
    let x_of =
        |evolution: f64| plot_left + INSET + evolution * (plot_right - plot_left - 2.0 * INSET);
    let y_of = |visibility: f64| {
        plot_top + INSET + (1.0 - visibility) * (plot_bottom - plot_top - 2.0 * INSET)
    };

    // The frame, the stage boundaries and their names, and the names of the axes.
    elements.push(rect(
        (
            plot_left,
            plot_top,
            plot_right - plot_left,
            plot_bottom - plot_top,
        ),
        "chartlet-vmap-frame",
        None,
    ));
    for (index, (start, stage)) in crate::spec::VALUEMAP_STAGES.iter().enumerate() {
        let end = crate::spec::VALUEMAP_STAGES
            .get(index + 1)
            .map_or(1.0, |(next, _)| *next);
        let (from, to) = (
            if index == 0 { plot_left } else { x_of(*start) },
            if index == 3 { plot_right } else { x_of(end) },
        );
        if index > 0 {
            elements.push(line(
                (from, plot_top),
                (from, plot_bottom),
                "chartlet-vmap-stage-line",
            ));
        }
        elements.push(text(
            (f64::midpoint(from, to), plot_bottom + 16.0),
            TextAnchor::Middle,
            "chartlet-vmap-stage",
            fit_text(
                text::valuemap_stage(spec.locale, *stage),
                to - from - 4.0,
                SIZE,
                metrics,
                warnings,
                "/valuemap",
            ),
        ));
    }
    elements.push(text(
        (f64::midpoint(plot_left, plot_right), plot_bottom + 36.0),
        TextAnchor::Middle,
        "chartlet-vmap-axis",
        words.evolution,
    ));
    elements.push(text(
        (plot_left, plot_top - 4.0),
        TextAnchor::Start,
        "chartlet-vmap-axis",
        words.visibility,
    ));
    for (label, y) in [
        (words.visible, plot_top + INSET + 4.0),
        (words.hidden, plot_bottom - INSET + 4.0),
    ] {
        elements.push(text(
            (plot_left - 6.0, y),
            TextAnchor::End,
            "chartlet-vmap-tick",
            fit_text(
                label,
                plot_left - margin - 6.0,
                SIZE,
                metrics,
                warnings,
                "/valuemap",
            ),
        ));
    }

    let at = |index: usize| {
        let component = &map.components[index];
        (x_of(component.evolution), y_of(component.visibility))
    };
    // Dependencies first, so that the dots lie over them.
    for dependency in &map.dependencies {
        let from = map.index_of(&dependency.from).expect("validated");
        let to = map.index_of(&dependency.to).expect("validated");
        elements.push(line(at(from), at(to), "chartlet-vmap-link"));
    }
    // The movement of a component: a dashed arrow to a hollow dot.
    let mut taken: Vec<Area> = Vec::new();
    for (index, component) in map.components.iter().enumerate() {
        let (x, y) = at(index);
        taken.push((x - DOT - 2.0, y - DOT - 2.0, x + DOT + 2.0, y + DOT + 2.0));
        if let Some(target) = &component.moves_to {
            let (tx, ty) = (
                x_of(target.evolution),
                y_of(target.visibility.unwrap_or(component.visibility)),
            );
            let length = (tx - x).hypot(ty - y).max(1.0);
            let direction = ((tx - x) / length, (ty - y) / length);
            elements.push(circle(
                (tx, ty, DOT),
                "chartlet-vmap-ghost",
                Some(format!(
                    "{}: moves to {}",
                    component.label,
                    percent(spec, target.evolution)
                )),
            ));
            elements.push(Element::Polyline(polyline(
                vec![
                    (x + direction.0 * (DOT + 1.0), y + direction.1 * (DOT + 1.0)),
                    (
                        tx - direction.0 * (DOT + 9.0),
                        ty - direction.1 * (DOT + 9.0),
                    ),
                ],
                "chartlet-vmap-move",
                None,
            )));
            elements.push(Element::Polyline(arrowhead(
                (
                    tx - direction.0 * (DOT + 1.0),
                    ty - direction.1 * (DOT + 1.0),
                ),
                direction,
                true,
                "chartlet-vmap-head",
            )));
            taken.push((
                x.min(tx) - 3.0,
                y.min(ty) - 3.0,
                x.max(tx) + 3.0,
                y.max(ty) + 3.0,
            ));
        }
    }
    for (index, component) in map.components.iter().enumerate() {
        let (x, y) = at(index);
        elements.push(circle(
            (x, y, DOT),
            "chartlet-vmap-dot",
            Some(format!(
                "{}: visibility {}, evolution {}",
                component.label,
                percent(spec, component.visibility),
                percent(spec, component.evolution)
            )),
        ));
        if component.inertia {
            elements.push(line(
                (x + DOT + 4.0, y - 8.0),
                (x + DOT + 4.0, y + 8.0),
                "chartlet-vmap-inertia",
            ));
            taken.push((x + DOT + 2.0, y - 9.0, x + DOT + 7.0, y + 9.0));
        }
    }

    // Labels: to the right of the dot, else to the left, above or below, the first place that
    // overlaps nothing and stays on the plot; where none does, the one that overlaps least.
    let room = if compact { 110.0 } else { 190.0 };
    let mut order: Vec<usize> = (0..map.components.len()).collect();
    order.sort_by(|a, b| {
        map.components[*b]
            .visibility
            .total_cmp(&map.components[*a].visibility)
            .then(a.cmp(b))
    });
    for index in order {
        let component = &map.components[index];
        let (x, y) = at(index);
        let label = fit_text(
            &component.label,
            room,
            SIZE,
            metrics,
            warnings,
            &format!("/valuemap/components/{index}/label"),
        );
        let label_width = metrics.width(&label, SIZE);
        let right_edge = x + DOT + if component.inertia { 12.0 } else { 5.0 };
        let candidates: [(f64, f64, TextAnchor, Area); 4] = [
            (
                right_edge + 3.0,
                y + 4.0,
                TextAnchor::Start,
                (right_edge, y - 9.0, right_edge + 6.0 + label_width, y + 6.0),
            ),
            (
                x - DOT - 8.0,
                y + 4.0,
                TextAnchor::End,
                (
                    x - DOT - 12.0 - label_width,
                    y - 9.0,
                    x - DOT - 4.0,
                    y + 6.0,
                ),
            ),
            (
                x,
                y - DOT - 7.0,
                TextAnchor::Middle,
                (
                    x - label_width / 2.0 - 2.0,
                    y - DOT - 20.0,
                    x + label_width / 2.0 + 2.0,
                    y - DOT - 3.0,
                ),
            ),
            (
                x,
                y + DOT + 16.0,
                TextAnchor::Middle,
                (
                    x - label_width / 2.0 - 2.0,
                    y + DOT + 3.0,
                    x + label_width / 2.0 + 2.0,
                    y + DOT + 20.0,
                ),
            ),
        ];
        let inside = |area: Area| {
            area.0 >= plot_left
                && area.2 <= plot_right
                && area.1 >= plot_top
                && area.3 <= plot_bottom
        };
        let clashes = |area: Area| taken.iter().filter(|other| overlap(area, **other)).count();
        let best = candidates
            .iter()
            .position(|candidate| inside(candidate.3) && clashes(candidate.3) == 0)
            .unwrap_or_else(|| {
                (0..candidates.len())
                    .min_by_key(|position| {
                        (
                            !inside(candidates[*position].3),
                            clashes(candidates[*position].3),
                            *position,
                        )
                    })
                    .expect("there are candidates")
            });
        let (tx, ty, anchor, area) = candidates[best];
        taken.push(area);
        elements.push(text((tx, ty), anchor, "chartlet-vmap-label", label));
    }
    warn_growth_advising(spec, (width, height), "shorten the labels", warnings);
    Scene {
        width: pixels(width),
        height: pixels(height),
        elements,
    }
}

/// The map in words: its components from the most visible down, each with its place, its stage and
/// where it moves, and then the dependencies.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let map = valuemap(spec);
    let mut summary = text::valuemap_opening(spec.locale, map.components.len());
    for index in sorted(map) {
        let component = &map.components[index];
        let mut facts = vec![
            text::valuemap_visibility(spec.locale, &percent(spec, component.visibility)),
            text::valuemap_stage(
                spec.locale,
                crate::spec::ValuemapStage::of(component.evolution),
            )
            .to_owned(),
        ];
        if let Some(target) = &component.moves_to {
            facts.push(text::valuemap_moves(
                spec.locale,
                text::valuemap_stage(
                    spec.locale,
                    crate::spec::ValuemapStage::of(target.evolution),
                ),
            ));
        }
        if component.inertia {
            facts.push(text::valuemap_inertia(spec.locale).to_owned());
        }
        let _ = write!(summary, " {} ({}).", component.label, facts.join(", "));
    }
    if !map.dependencies.is_empty() {
        let pairs: Vec<(&str, &str)> = map
            .dependencies
            .iter()
            .map(|dependency| {
                (
                    map.components[map.index_of(&dependency.from).expect("validated")]
                        .label
                        .as_str(),
                    map.components[map.index_of(&dependency.to).expect("validated")]
                        .label
                        .as_str(),
                )
            })
            .collect();
        summary.push(' ');
        summary.push_str(&text::valuemap_dependencies(spec.locale, &pairs));
    }
    summary
}

/// The components from the most visible down; equally visible ones in the order listed.
fn sorted(map: &ValuemapSpec) -> Vec<usize> {
    let mut order: Vec<usize> = (0..map.components.len()).collect();
    order.sort_by(|a, b| {
        map.components[*b]
            .visibility
            .total_cmp(&map.components[*a].visibility)
            .then(a.cmp(b))
    });
    order
}

/// One row per component, the most visible first.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let map = valuemap(spec);
    let words = text::valuemap_words(spec.locale);
    let rows = sorted(map)
        .into_iter()
        .map(|index| {
            let component = &map.components[index];
            let needs: Vec<&str> = map
                .dependencies
                .iter()
                .filter(|dependency| dependency.from == component.id)
                .map(|dependency| {
                    map.components[map.index_of(&dependency.to).expect("validated")]
                        .label
                        .as_str()
                })
                .collect();
            vec![
                component.label.clone(),
                percent(spec, component.visibility),
                percent(spec, component.evolution),
                text::valuemap_stage(
                    spec.locale,
                    crate::spec::ValuemapStage::of(component.evolution),
                )
                .to_owned(),
                component
                    .moves_to
                    .as_ref()
                    .map_or_else(String::new, |target| percent(spec, target.evolution)),
                if component.inertia {
                    words.yes.to_owned()
                } else {
                    String::new()
                },
                needs.join(", "),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", spec.locale.words().data_for, spec.title),
        columns: text::valuemap_columns(spec.locale)
            .iter()
            .map(|column| (*column).to_owned())
            .collect(),
        rows,
    }
}
