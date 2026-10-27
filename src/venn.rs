//! Venn diagrams of two or three sets: circles at fixed places, not sized by their contents, so
//! that every region exists whatever its count. A region carries its label and its count inside
//! where they fit; otherwise they stand in a gutter beside the diagram, joined to the region by a
//! leader line. Each circle is named beside it. The description and the data table name every
//! region, also those the specification says nothing about.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{badge, pixels, warn_growth_advising, wrap},
    draw::{circle, line, text},
    error::ChartWarning,
    layout::{count, fit_text, format_value, push_title, title_extra, two_lines},
    metrics::TextMetrics,
    scene::{Element, Scene, TextAnchor},
    spec::{ChartSpec, VennSpec},
    text,
};

const SIZE: f64 = 12.0;
const SET_SIZE: f64 = 13.0;
const LINE: f64 = 14.0;
/// The height of the line of a count.
const COUNT: f64 = 17.0;
const CIRCLES: [&str; 3] = [
    "chartlet-venn chartlet-venn-1",
    "chartlet-venn chartlet-venn-2",
    "chartlet-venn chartlet-venn-3",
];

fn venn(spec: &ChartSpec) -> &VennSpec {
    spec.venn
        .as_ref()
        .expect("validated Venn diagrams carry a venn block")
}

/// The circles of a diagram of `sets` sets as centres in units of the radius, and how far the
/// whole reaches: half its width, and its top and bottom, from the middle of the first circle's
/// row.
struct Shape {
    centers: &'static [(f64, f64)],
    half_width: f64,
    top: f64,
    bottom: f64,
}

const fn shape(sets: usize) -> Shape {
    if sets == 2 {
        Shape {
            centers: &[(-0.5, 0.0), (0.5, 0.0)],
            half_width: 1.5,
            top: -1.0,
            bottom: 1.0,
        }
    } else {
        Shape {
            centers: &[(-0.625, -0.361), (0.625, -0.361), (0.0, 0.722)],
            half_width: 1.625,
            top: -1.361,
            bottom: 1.722,
        }
    }
}

/// Where the label and the count of a region stand, as a point in units of the radius, with the
/// width and the height they may fill there.
const fn anchor(sets: usize, mask: u8) -> (f64, f64, f64, f64) {
    if sets == 2 {
        match mask {
            1 => (-1.0, 0.0, 0.85, 0.8),
            2 => (1.0, 0.0, 0.85, 0.8),
            _ => (0.0, 0.0, 0.8, 0.8),
        }
    } else {
        match mask {
            1 => (-1.146, -0.662, 0.9, 0.7),
            2 => (1.146, -0.662, 0.9, 0.7),
            4 => (0.0, 1.322, 0.9, 0.7),
            3 => (0.0, -0.62, 0.68, 0.6),
            5 => (-0.607, 0.35, 0.6, 0.6),
            6 => (0.607, 0.35, 0.6, 0.6),
            _ => (0.0, 0.07, 0.5, 0.5),
        }
    }
}

/// A region the specification says something about.
struct Content {
    mask: u8,
    /// The position of its entry in `regions`, for the paths of warnings.
    index: usize,
    label: Option<String>,
    count: Option<String>,
}

fn contents(spec: &ChartSpec) -> Vec<Content> {
    let venn = venn(spec);
    venn.masks()
        .into_iter()
        .filter_map(|mask| {
            let index = venn
                .regions
                .iter()
                .position(|region| venn.mask_of(&region.sets) == mask)?;
            let region = &venn.regions[index];
            let count = region
                .count
                .map(|count| format_value(count, spec.number_style()));
            Some(Content {
                mask,
                index,
                label: region.label.clone(),
                count,
            })
        })
        .collect()
}

/// Whether the label and the count of `content` fit where the region has room for them.
fn fits(content: &Content, (sets, radius): (usize, f64), metrics: &impl TextMetrics) -> bool {
    let (_, _, room_width, room_height) = anchor(sets, content.mask);
    let (width, height) = (room_width * radius - 6.0, room_height * radius);
    let mut lines = 0.0;
    let mut wide_enough = true;
    if let Some(label) = &content.label {
        wide_enough = match two_lines(label, width, SIZE, metrics) {
            None => {
                lines = 1.0;
                metrics.width(label, SIZE) <= width
            }
            Some((_, rest)) => {
                lines = 2.0;
                metrics.width(rest, SIZE) <= width
            }
        };
    }
    let needed = LINE * lines + if content.count.is_some() { COUNT } else { 0.0 };
    wide_enough && needed <= height
}

/// The lines a region's label and count make: the label wrapped to `width`, then the count.
fn lines(
    content: &Content,
    width: f64,
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> (Vec<String>, f64) {
    let label = content.label.as_ref().map_or_else(Vec::new, |label| {
        wrap(
            label,
            width,
            SIZE,
            metrics,
            warnings,
            &format!("/venn/regions/{}/label", content.index),
        )
    });
    let height = LINE * count(label.len()) + if content.count.is_some() { COUNT } else { 0.0 };
    (label, height)
}

/// Draws the label and the count of a region as a block of text whose middle is at `y` and whose
/// anchor is at `x`.
fn push_block(
    elements: &mut Vec<Element>,
    content: &Content,
    (label, height): (&[String], f64),
    (x, y, anchor): (f64, f64, TextAnchor),
) {
    let top = y - height / 2.0;
    for (position, line) in label.iter().enumerate() {
        elements.push(text(
            (x, top + 11.0 + LINE * count(position)),
            anchor,
            "chartlet-venn-label",
            line.clone(),
        ));
    }
    if let Some(number) = &content.count {
        elements.push(text(
            (x, top + LINE * count(label.len()) + 13.5),
            anchor,
            "chartlet-venn-count",
            number.clone(),
        ));
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let venn = venn(spec);
    let sets = venn.sets.len();
    let shape = shape(sets);
    let compact = spec.width < crate::layout::NARROW;
    let margin = if compact { 12.0 } else { 24.0 };
    let gutter = if compact { 108.0 } else { 160.0 };
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
    let contents = contents(spec);
    // The circle names stand above the circles, and below the third.
    let (names_above, names_below) = (22.0, if sets == 3 { 24.0 } else { 8.0 });
    let left_of = |content: &Content| anchor(sets, content.mask).0 < -0.05;

    // The region between the two upper circles of three, where they part above it, has room for
    // its text in the notch over the diagram instead of a gutter.
    let in_notch = |content: &Content| !compact && sets == 3 && content.mask == 3;
    let uses_gutter = |content: &Content, out: bool, left: bool| {
        out && !compact && !in_notch(content) && left_of(content) == left
    };

    // Regions whose text does not fit stand in a gutter; making room for it makes the circles
    // smaller, which can push more regions out, so this runs until it settles.
    let mut outside = vec![false; contents.len()];
    let radius = loop {
        let has = |left: bool| {
            contents
                .iter()
                .zip(&outside)
                .any(|(content, out)| uses_gutter(content, *out, left))
        };
        let room_width = width
            - 2.0 * margin
            - if has(true) { gutter + 12.0 } else { 0.0 }
            - if has(false) { gutter + 12.0 } else { 0.0 };
        let room_height = height - top - names_above - names_below - 16.0;
        let radius = (room_width / (2.0 * shape.half_width))
            .min(room_height / (shape.bottom - shape.top))
            .clamp(30.0, 170.0)
            .floor();
        let mut changed = false;
        for (content, out) in contents.iter().zip(outside.iter_mut()) {
            if !*out && !fits(content, (sets, radius), metrics) {
                *out = true;
                changed = true;
            }
        }
        if !changed {
            break radius;
        }
    };
    let left_room = if contents
        .iter()
        .zip(&outside)
        .any(|(content, out)| uses_gutter(content, *out, true))
    {
        gutter + 12.0
    } else {
        0.0
    };
    let diagram_width = 2.0 * shape.half_width * radius;
    let center_x = margin + left_room + diagram_width / 2.0;
    let center_y = top + names_above - shape.top * radius;

    // The circles first, then what is written in them.
    for (index, (dx, dy)) in shape.centers.iter().enumerate() {
        elements.push(circle(
            (center_x + dx * radius, center_y + dy * radius, radius),
            CIRCLES[index],
            Some(venn.sets[index].clone()),
        ));
    }
    for (index, (dx, dy)) in shape.centers.iter().enumerate() {
        // The name of a circle stands beside its outer edge: above, or below for the third.
        let (x, y) = if sets == 3 && index == 2 {
            (
                center_x + dx * radius,
                center_y + shape.bottom * radius + 18.0,
            )
        } else {
            let shift = if *dx < 0.0 { -0.3 } else { 0.3 };
            (
                center_x + (dx + shift) * radius,
                center_y + (dy - 1.0) * radius - 8.0,
            )
        };
        elements.push(text(
            (x, y),
            TextAnchor::Middle,
            "chartlet-venn-set",
            fit_text(
                &venn.sets[index],
                1.4 * radius,
                SET_SIZE,
                metrics,
                warnings,
                &format!("/venn/sets/{index}"),
            ),
        ));
    }
    // Regions that fit carry their text. On a wide canvas the others get a dot and a leader line
    // to a gutter; on a narrow one a numbered badge and an entry in a key below the diagram.
    let mut gutters: [Vec<(f64, f64, usize)>; 2] = [Vec::new(), Vec::new()];
    let mut keyed: Vec<usize> = Vec::new();
    let mut needed = center_y + shape.bottom * radius + names_below + 16.0;
    for (position, content) in contents.iter().enumerate() {
        let (ax, ay, room_width, _) = anchor(sets, content.mask);
        let (x, y) = (center_x + ax * radius, center_y + ay * radius);
        if !outside[position] {
            let (label, block) = lines(content, room_width * radius - 6.0, (warnings, metrics));
            push_block(
                &mut elements,
                content,
                (&label, block),
                (x, y, TextAnchor::Middle),
            );
        } else if compact {
            keyed.push(position);
            badge(keyed.len(), (x, y), &mut elements);
        } else if in_notch(content) {
            let (label, block) = lines(content, radius.min(120.0), (warnings, metrics));
            let notch = center_y + (shape.centers[0].1 - 0.781) * radius;
            push_block(
                &mut elements,
                content,
                (&label, block),
                (center_x, notch - block / 2.0 - 4.0, TextAnchor::Middle),
            );
        } else {
            gutters[usize::from(!left_of(content))].push((y, x, position));
        }
    }
    for (side, entries) in gutters.iter_mut().enumerate() {
        entries.sort_by(|a, b| a.0.total_cmp(&b.0));
        let (edge, anchor_side, text_x) = if side == 0 {
            (margin + left_room - 12.0, TextAnchor::End, margin + gutter)
        } else {
            (
                center_x + diagram_width / 2.0 + 12.0,
                TextAnchor::Start,
                center_x + diagram_width / 2.0 + 12.0 + 4.0,
            )
        };
        let mut next = top + names_above;
        for (y, x, position) in entries.iter() {
            let content = &contents[*position];
            let (label, block) = lines(content, gutter - 8.0, (warnings, metrics));
            let middle = (*y).max(next + block / 2.0);
            next = middle + block / 2.0 + 10.0;
            elements.push(circle((*x, *y, 3.0), "chartlet-venn-dot", None));
            elements.push(line((*x, *y), (edge, middle), "chartlet-venn-leader"));
            push_block(
                &mut elements,
                content,
                (&label, block),
                (text_x, middle, anchor_side),
            );
        }
        needed = needed.max(next + 6.0);
    }
    // The key: the badge's number, then the region's label and count.
    let mut key_y = needed - 8.0;
    for (number, position) in keyed.iter().enumerate() {
        let content = &contents[*position];
        let (label, block) = lines(content, width - 2.0 * margin - 30.0, (warnings, metrics));
        let middle = key_y + block.max(20.0) / 2.0;
        badge(number + 1, (margin + 8.0, middle), &mut elements);
        push_block(
            &mut elements,
            content,
            (&label, block),
            (margin + 26.0, middle, TextAnchor::Start),
        );
        key_y = middle + block.max(20.0) / 2.0 + 8.0;
        needed = key_y + 8.0;
    }
    warn_growth_advising(
        spec,
        (width, needed),
        "shorten the labels or name fewer regions",
        warnings,
    );
    Scene {
        width: spec.width,
        height: pixels(needed.max(height)),
        elements,
    }
}

/// The diagram in words: its sets, then every region the specification says something about.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let venn = venn(spec);
    let mut summary = text::venn_opening(spec.locale, &venn.sets);
    for content in contents(spec) {
        let name = text::venn_region(spec.locale, &venn.names(content.mask));
        let said = match (&content.label, &content.count) {
            (Some(label), Some(count)) => format!("{label} ({count})"),
            (Some(label), None) => label.clone(),
            (None, Some(count)) => count.clone(),
            (None, None) => continue,
        };
        let _ = write!(summary, " {name}: {said}.");
    }
    summary
}

/// One row per region, also the ones the specification does not mention: its name, its label and
/// its count.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let venn = venn(spec);
    let rows = venn
        .masks()
        .into_iter()
        .map(|mask| {
            let region = venn.region(mask);
            vec![
                text::venn_region(spec.locale, &venn.names(mask)),
                region
                    .and_then(|region| region.label.clone())
                    .unwrap_or_default(),
                region
                    .and_then(|region| region.count)
                    .map(|number| format_value(number, spec.number_style()))
                    .unwrap_or_default(),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", spec.locale.words().data_for, spec.title),
        columns: text::venn_columns(spec.locale)
            .iter()
            .map(|column| (*column).to_owned())
            .collect(),
        rows,
    }
}
