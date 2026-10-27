//! Fishbone diagrams: one effect at the head of a spine and, on bones that alternate on either
//! side of it, the categories of its causes. A cause is a horizontal rib that meets its bone, with
//! its label above the line, and a sub-cause is an indented rib below its cause. Labels are never
//! set on a diagonal; only the bones lean, toward the head. On a wide canvas the spine runs from
//! left to right with the bones above and below it; on a narrow one it runs from top to bottom
//! with the bones left and right, where the labels have the width of half the canvas. The
//! description and the data table list the effect, the categories and the causes in the order of
//! the specification.

use crate::{
    DataTable,
    diagram::{arrowhead, pixels, warn_growth_advising, wrap},
    draw::{line, rect, text},
    error::ChartWarning,
    layout::{count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Scene, TextAnchor},
    spec::{ChartSpec, FishboneBoneSpec, FishboneSpec},
    text::{self, FishboneRow},
};

const SIZE: f64 = 12.0;
const LABEL_SIZE: f64 = 13.0;
/// The height of a row of one line and of two lines of text, the distance between the lines of
/// a label, and the height of a category box.
const ROW_ONE: f64 = 26.0;
const ROW_TWO: f64 = 40.0;
const LINE: f64 = 14.0;
const BOX_HEIGHT: f64 = 26.0;

fn fishbone(spec: &ChartSpec) -> &FishboneSpec {
    spec.fishbone
        .as_ref()
        .expect("validated fishbone diagrams carry a fishbone block")
}

/// A row of causes on a bone: its depth (0 for a cause, 1 for a sub-cause) and its lines.
struct Row {
    depth: u8,
    lines: Vec<String>,
}

impl Row {
    fn height(&self) -> f64 {
        if self.lines.len() > 1 {
            ROW_TWO
        } else {
            ROW_ONE
        }
    }
}

/// What one bone needs: its rows and its height, from the category box to the spine.
struct Bone {
    index: usize,
    rows: Vec<Row>,
    height: f64,
}

/// The bones with their rows, and the effect wrapped for the head.
struct Parts {
    bones: Vec<Bone>,
    head_lines: Vec<String>,
    head_height: f64,
}

/// The rows of every bone from the top, each cause followed by its sub-causes, wrapped to
/// `width` (`indent` less for a sub-cause), and the lines of the effect wrapped to `head_width`.
fn parts(
    fishbone: &FishboneSpec,
    (width, indent, head_width): (f64, f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> Parts {
    let bones = fishbone
        .bones
        .iter()
        .enumerate()
        .map(|(bone_index, bone)| {
            let mut rows = Vec::new();
            for (cause_index, cause) in bone.causes.iter().enumerate() {
                let path = format!("/fishbone/bones/{bone_index}/causes/{cause_index}/label");
                rows.push(Row {
                    depth: 0,
                    lines: wrap(&cause.label, width, SIZE, metrics, warnings, &path),
                });
                for (sub_index, sub) in cause.causes.iter().enumerate() {
                    let path = format!(
                        "/fishbone/bones/{bone_index}/causes/{cause_index}/causes/{sub_index}/label"
                    );
                    rows.push(Row {
                        depth: 1,
                        lines: wrap(&sub.label, width - indent, SIZE, metrics, warnings, &path),
                    });
                }
            }
            let height = BOX_HEIGHT + 16.0 + rows.iter().map(Row::height).sum::<f64>();
            Bone {
                index: bone_index,
                rows,
                height,
            }
        })
        .collect();
    let head_lines = wrap(
        &fishbone.effect,
        head_width,
        LABEL_SIZE,
        metrics,
        warnings,
        "/fishbone/effect",
    );
    let head_height = (LINE * count(head_lines.len()) + 20.0).max(40.0);
    Parts {
        bones,
        head_lines,
        head_height,
    }
}

/// Where the rows of a bone stand: the left edge of the text, how far a sub-cause stands in, and
/// for a rib at height `y` and a sub-cause's offset the two ends of its line.
struct Ribs<'a> {
    text_left: f64,
    indent: f64,
    ends: &'a dyn Fn(f64, f64) -> (f64, f64),
}

/// Draws the rows of a bone from `row_top` down: a line for each, and its label above it.
fn push_rows(elements: &mut Vec<Element>, rows: &[Row], mut row_top: f64, ribs: &Ribs) {
    for row in rows {
        let rib_y = row_top + row.height() - 6.0;
        let offset = ribs.indent * f64::from(row.depth);
        let (from, to) = (ribs.ends)(rib_y, offset);
        elements.push(line((from, rib_y), (to, rib_y), "chartlet-fish-rib"));
        let class = if row.depth == 0 {
            "chartlet-fish-cause"
        } else {
            "chartlet-fish-subcause"
        };
        for (position, content) in row.lines.iter().enumerate() {
            let up = count(row.lines.len() - 1 - position);
            elements.push(text(
                (ribs.text_left + offset + 2.0, rib_y - 4.0 - LINE * up),
                TextAnchor::Start,
                class,
                content.clone(),
            ));
        }
        row_top += row.height();
    }
}

/// The head: the effect in a box centred on `x`, whose vertical middle is at `y`.
fn push_head(
    elements: &mut Vec<Element>,
    parts: &Parts,
    effect: &str,
    (x, y, width): (f64, f64, f64),
) {
    elements.push(rect(
        (
            x - width / 2.0,
            y - parts.head_height / 2.0,
            width,
            parts.head_height,
        ),
        "chartlet-fish-head",
        Some(effect.to_owned()),
    ));
    let first = y - LINE * (count(parts.head_lines.len()) - 1.0) / 2.0 + 4.5;
    for (position, content) in parts.head_lines.iter().enumerate() {
        elements.push(text(
            (x, first + LINE * count(position)),
            TextAnchor::Middle,
            "chartlet-fish-head-text",
            content.clone(),
        ));
    }
}

/// The category box of bone `index` with its label, `y` at its top, no wider than `max_width`:
/// centred on `x`, or with its right (`Some(true)`) or its left edge (`Some(false)`) at `x`.
fn push_category(
    elements: &mut Vec<Element>,
    (spec, index): (&ChartSpec, usize),
    (x, y, max_width): (f64, f64, f64),
    edge: Option<bool>,
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) {
    let full = &fishbone(spec).bones[index].label;
    let label = fit_text(
        full,
        max_width - 20.0,
        LABEL_SIZE,
        metrics,
        warnings,
        &format!("/fishbone/bones/{index}/label"),
    );
    let width = (metrics.width(&label, LABEL_SIZE) + 20.0).min(max_width);
    let left = match edge {
        None => x - width / 2.0,
        Some(true) => x - width,
        Some(false) => x,
    };
    elements.push(rect(
        (left, y, width, BOX_HEIGHT),
        "chartlet-fish-category",
        Some(full.clone()),
    ));
    elements.push(text(
        (left + width / 2.0, y + BOX_HEIGHT / 2.0 + 4.5),
        TextAnchor::Middle,
        "chartlet-fish-category-text",
        label,
    ));
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    if spec.width < crate::layout::NARROW {
        vertical(spec, warnings, metrics)
    } else {
        horizontal(spec, warnings, metrics)
    }
}

/// The spine runs from left to right; the bones alternate above and below it.
fn horizontal(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let fishbone = fishbone(spec);
    let (margin, slant, indent, gap) = (24.0, 36.0, 14.0, 20.0);
    let (head_width, min_column) = (140.0, 150.0);
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        f64::from(spec.width) - 2.0 * margin,
        metrics,
        warnings,
    );
    let top = 56.0 + title_extra(spec, f64::from(spec.width) - 2.0 * margin, metrics);

    // Two bones share a column, one above the spine and one below it.
    let columns = fishbone.bones.len().div_ceil(2);
    let room = f64::from(spec.width) - 2.0 * margin - head_width - gap;
    let column = (room / count(columns)).max(min_column).floor();
    let width = 2.0 * margin + head_width + gap + column * count(columns);
    let reach = column - slant - 12.0;
    let parts = parts(
        fishbone,
        (reach - 8.0, indent, head_width - 16.0),
        (warnings, metrics),
    );
    let side = |parity: usize| {
        parts
            .bones
            .iter()
            .filter(|bone| bone.index % 2 == parity)
            .map(|bone| bone.height)
            .fold(parts.head_height / 2.0 + 12.0, f64::max)
    };
    let (above, below) = (side(0), side(1));
    let spine = top + above;
    let height = spine + below + 16.0;
    let head_left = width - margin - head_width;

    // The spine, the head and the bones go down first, so that the ribs and labels sit on them.
    elements.push(line(
        (margin, spine),
        (head_left - 6.0, spine),
        "chartlet-fish-spine",
    ));
    elements.push(Element::Polyline(arrowhead(
        (head_left, spine),
        (1.0, 0.0),
        true,
        "chartlet-fish-arrow",
    )));
    push_head(
        &mut elements,
        &parts,
        &fishbone.effect,
        (head_left + head_width / 2.0, spine, head_width),
    );

    for bone in &parts.bones {
        let is_above = bone.index % 2 == 0;
        let left = margin + column * count(bone.index / 2);
        let outer_x = left + reach;
        // The category box stands at the outer end of the bone.
        let box_top = if is_above {
            spine - above
        } else {
            spine + below - BOX_HEIGHT
        };
        let start_y = if is_above {
            box_top + BOX_HEIGHT
        } else {
            box_top
        };
        elements.push(line(
            (outer_x, start_y),
            (outer_x + slant, spine),
            "chartlet-fish-bone",
        ));
        push_category(
            &mut elements,
            (spec, bone.index),
            (outer_x, box_top, column - 8.0),
            None,
            (warnings, metrics),
        );
        // The ribs run from a common left edge to the bone, which leans toward the head.
        let bone_x = |y: f64| outer_x + slant * (y - start_y) / (spine - start_y);
        let ends = |y: f64, offset: f64| (left + offset, bone_x(y));
        push_rows(
            &mut elements,
            &bone.rows,
            if is_above { start_y + 8.0 } else { spine + 8.0 },
            &Ribs {
                text_left: left,
                indent,
                ends: &ends,
            },
        );
    }
    warn_growth_advising(spec, (width, height), "use fewer bones", warnings);
    Scene {
        width: pixels(width.max(f64::from(spec.width))),
        height: pixels(height.max(f64::from(spec.height))),
        elements,
    }
}

/// The spine runs from top to bottom to the head; the bones alternate left and right of it, two
/// in a band, and lean toward the head.
#[allow(clippy::too_many_lines)]
fn vertical(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let fishbone = fishbone(spec);
    let (margin, slant, indent) = (8.0, 14.0, 8.0);
    let width = f64::from(spec.width);
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

    let half = ((width - 2.0 * margin) / 2.0).floor();
    let reach = half - slant;
    let spine_x = margin + half;
    let head_width = (width - 2.0 * margin).min(200.0);
    let parts = parts(
        fishbone,
        (reach - 8.0, indent, head_width - 16.0),
        (warnings, metrics),
    );
    let bands = fishbone.bones.len().div_ceil(2);
    let band_heights: Vec<f64> = (0..bands)
        .map(|band| {
            parts
                .bones
                .iter()
                .filter(|bone| bone.index / 2 == band)
                .map(|bone| bone.height)
                .fold(0.0, f64::max)
        })
        .collect();
    let bones_bottom = top + band_heights.iter().sum::<f64>();
    let head_top = bones_bottom + 16.0;
    let height = head_top + parts.head_height + 16.0;

    elements.push(line(
        (spine_x, top),
        (spine_x, head_top - 6.0),
        "chartlet-fish-spine",
    ));
    elements.push(Element::Polyline(arrowhead(
        (spine_x, head_top),
        (0.0, 1.0),
        true,
        "chartlet-fish-arrow",
    )));
    push_head(
        &mut elements,
        &parts,
        &fishbone.effect,
        (
            spine_x.clamp(margin + head_width / 2.0, width - margin - head_width / 2.0),
            head_top + parts.head_height / 2.0,
            head_width,
        ),
    );

    let mut band_top = top;
    for (band, band_height) in band_heights.iter().enumerate() {
        let band_bottom = band_top + band_height;
        let start_y = band_top + BOX_HEIGHT;
        for bone in parts.bones.iter().filter(|bone| bone.index / 2 == band) {
            let left_side = bone.index % 2 == 0;
            // The outer end of the bone is a slant away from the spine; the box beside the spine
            // keeps clear of it.
            let outer_x = if left_side {
                spine_x - slant
            } else {
                spine_x + slant
            };
            elements.push(line(
                (outer_x, start_y),
                (spine_x, band_bottom),
                "chartlet-fish-bone",
            ));
            push_category(
                &mut elements,
                (spec, bone.index),
                (
                    if left_side {
                        spine_x - 6.0
                    } else {
                        spine_x + 6.0
                    },
                    band_top,
                    half - 12.0,
                ),
                Some(left_side),
                (warnings, metrics),
            );
            let lean = |y: f64| slant * (y - start_y) / (band_bottom - start_y);
            let left_ends = move |y: f64, offset: f64| (margin + offset, outer_x + lean(y));
            let right_ends = move |y: f64, _: f64| (outer_x - lean(y), width - margin);
            let (text_left, ends): (f64, &dyn Fn(f64, f64) -> (f64, f64)) = if left_side {
                (margin, &left_ends)
            } else {
                (spine_x + slant + 6.0, &right_ends)
            };
            push_rows(
                &mut elements,
                &bone.rows,
                start_y + 8.0,
                &Ribs {
                    text_left,
                    indent,
                    ends,
                },
            );
        }
        band_top = band_bottom;
    }
    warn_growth_advising(spec, (width, height), "use fewer bones", warnings);
    Scene {
        width: spec.width,
        height: pixels(height.max(f64::from(spec.height))),
        elements,
    }
}

/// The diagram in words: the effect, then every category with its causes and their sub-causes.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let fishbone = fishbone(spec);
    let causes = fishbone
        .bones
        .iter()
        .map(FishboneBoneSpec::cause_count)
        .sum();
    let mut summary = text::fishbone_opening(
        spec.locale,
        &fishbone.effect,
        (fishbone.bones.len(), causes),
    );
    for bone in &fishbone.bones {
        let listed: Vec<String> = bone
            .causes
            .iter()
            .map(|cause| {
                if cause.causes.is_empty() {
                    cause.label.clone()
                } else {
                    let subs: Vec<&str> =
                        cause.causes.iter().map(|sub| sub.label.as_str()).collect();
                    format!("{} ({})", cause.label, subs.join(", "))
                }
            })
            .collect();
        summary.push(' ');
        summary.push_str(&bone.label);
        summary.push_str(if listed.is_empty() { "." } else { ": " });
        summary.push_str(&listed.join("; "));
        if !listed.is_empty() {
            summary.push('.');
        }
    }
    summary
}

/// One row per item in drawing order: the effect, each category, each cause and sub-cause, with
/// what it belongs to.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let fishbone = fishbone(spec);
    let kind = |row| text::fishbone_row(spec.locale, row).to_owned();
    let mut rows = vec![vec![
        kind(FishboneRow::Effect),
        fishbone.effect.clone(),
        String::new(),
    ]];
    for bone in &fishbone.bones {
        rows.push(vec![
            kind(FishboneRow::Category),
            bone.label.clone(),
            fishbone.effect.clone(),
        ]);
        for cause in &bone.causes {
            rows.push(vec![
                kind(FishboneRow::Cause),
                cause.label.clone(),
                bone.label.clone(),
            ]);
            for sub in &cause.causes {
                rows.push(vec![
                    kind(FishboneRow::Subcause),
                    sub.label.clone(),
                    cause.label.clone(),
                ]);
            }
        }
    }
    DataTable {
        caption: format!("{} {}", spec.locale.words().data_for, spec.title),
        columns: text::belongs_columns(spec.locale)
            .iter()
            .map(|column| (*column).to_owned())
            .collect(),
        rows,
    }
}
