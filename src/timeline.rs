//! Timelines: one row per item over a shared time axis. A phase is a bar from its start to its
//! end, a milestone a diamond on its day, a marker a dashed line across all rows, and an arrow
//! runs from an item to the ones that follow it. The description and the data table list every
//! item with its days, so that the picture is never the only place they are said.

use crate::{
    DataTable,
    diagram::{self, arrowhead, pixels, rounded, warn_growth},
    error::ChartWarning,
    layout::{CONTENT_LEFT, LABEL_SIZE, count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, Span, TimelineSpec},
    text,
    time::{self, TimeZone},
};

/// Height of the row of one item.
const ROW: f64 = 30.0;
/// Thickness of a phase bar, and half the height of a milestone's diamond.
const BAR: f64 = 14.0;
const DIAMOND: f64 = 8.0;
/// A phase is drawn this wide at least, so that a day-long phase does not vanish.
const MIN_PHASE: f64 = 3.0;
/// Gap between two legend entries, and the room for a legend swatch.
const LEGEND_GAP: f64 = 16.0;
const LEGEND_SWATCH: f64 = 16.0;
/// How far an arrow leaves its item before it turns.
const TURN: f64 = 8.0;

const GROUP_CLASSES: [&str; crate::spec::MAX_SERIES] = [
    "chartlet-timeline-phase chartlet-timeline-group-1",
    "chartlet-timeline-phase chartlet-timeline-group-2",
    "chartlet-timeline-phase chartlet-timeline-group-3",
    "chartlet-timeline-phase chartlet-timeline-group-4",
];
const MILESTONE_CLASSES: [&str; crate::spec::MAX_SERIES] = [
    "chartlet-timeline-milestone chartlet-timeline-group-1",
    "chartlet-timeline-milestone chartlet-timeline-group-2",
    "chartlet-timeline-milestone chartlet-timeline-group-3",
    "chartlet-timeline-milestone chartlet-timeline-group-4",
];

fn timeline(spec: &ChartSpec) -> &TimelineSpec {
    spec.timeline
        .as_ref()
        .expect("validated timelines carry a timeline block")
}

fn zone(spec: &ChartSpec) -> TimeZone {
    crate::spec::timeline_zone().with_locale(spec.locale)
}

/// Whether any day of the timeline carries a time of day, so that its dates are written with it.
fn has_time_of_day(timeline: &TimelineSpec) -> bool {
    let utc = crate::spec::timeline_zone();
    time::any_has_time_of_day(
        timeline
            .items
            .iter()
            .flat_map(|item| [item.span().begin(), item.span().finish()])
            .chain(
                timeline
                    .markers
                    .iter()
                    .map(|marker| marker.at.resolve(utc).expect("validated")),
            ),
        utc,
    )
}

fn write_day(spec: &ChartSpec, epoch: i64, with_time: bool) -> String {
    if with_time {
        time::format_datetime(epoch, zone(spec))
    } else {
        time::format_date(epoch, zone(spec))
    }
}

/// An item's days as text: `2026-01-05 – 2026-03-01` for a phase, one day for a milestone.
fn days(spec: &ChartSpec, span: Span, with_time: bool) -> String {
    match span {
        Span::Phase { start, end } => format!(
            "{} – {}",
            write_day(spec, start, with_time),
            write_day(spec, end, with_time)
        ),
        Span::Milestone { at } => write_day(spec, at, with_time),
    }
}

/// Where everything of one timeline stands: the plot, the axis and the rows.
struct Frame<'a> {
    spec: &'a ChartSpec,
    timeline: &'a TimelineSpec,
    with_time: bool,
    margin: f64,
    /// The left edge of the plot, and the room left of it for the names of the items.
    left: f64,
    gutter: f64,
    plot_width: f64,
    top: f64,
    /// The days the axis runs from and to, in Unix seconds.
    first: i64,
    last: i64,
    /// Whether the dates stand beside the items.
    dates: bool,
}

/// Seconds as a number to place things by; a timeline spans far less than 2^52 of them.
#[allow(clippy::cast_precision_loss)]
const fn seconds(value: i64) -> f64 {
    value as f64
}

impl Frame<'_> {
    /// The position of a day across the plot.
    fn x(&self, epoch: i64) -> f64 {
        self.left + self.plot_width * seconds(epoch - self.first) / seconds(self.last - self.first)
    }

    /// The middle of the row of item `index`.
    fn center(&self, index: usize) -> f64 {
        self.top + ROW * (count(index) + 0.5)
    }

    fn plot_height(&self) -> f64 {
        count(self.timeline.items.len()) * ROW
    }

    fn width(&self) -> f64 {
        f64::from(self.spec.width)
    }
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let timeline = timeline(spec);
    let compact = spec.width < crate::layout::NARROW;
    let width = f64::from(spec.width);
    let margin = if compact { 12.0 } else { 24.0 };
    let with_time = has_time_of_day(timeline);
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    // Room for the title, then the legend of the groups, then the labels of the markers.
    let legend_top = 46.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let legend_bottom = push_legend(&mut elements, warnings, spec, (margin, legend_top), metrics);
    let top = legend_bottom.max(legend_top)
        + if timeline.markers.is_empty() {
            14.0
        } else {
            30.0
        };

    let widest = timeline
        .items
        .iter()
        .map(|item| metrics.width(&item.label, LABEL_SIZE))
        .fold(0.0, f64::max);
    let (low, high) = if compact {
        (80.0, 150.0)
    } else {
        (110.0, 240.0)
    };
    let gutter = (widest + 20.0).clamp(low, high).ceil();
    // The longest date text, which stands beside a bar or diamond when `showValues` asks for it;
    // on a phone the dates stay in the tooltips and the table.
    let dates = spec.show_values && !compact;
    let right = if dates {
        timeline
            .items
            .iter()
            .map(|item| metrics.width(&days(spec, item.span(), with_time), LABEL_SIZE))
            .fold(0.0, f64::max)
            .min(width * 0.35)
            + 24.0
    } else {
        8.0
    } + margin;
    let (first, last) = axis_span(timeline);
    let frame = Frame {
        spec,
        timeline,
        with_time,
        margin,
        left: margin + gutter,
        gutter,
        plot_width: (width - margin - gutter - right).max(60.0),
        top,
        first,
        last,
        dates,
    };
    let height = f64::from(spec.height).max(top + frame.plot_height() + 46.0);
    warn_growth(spec, width, height, warnings);
    frame.push_axis(&mut elements, metrics);
    frame.push_markers(&mut elements, warnings, metrics);
    // Arrows go below the items, so that a bar stays whole where an arrow reaches it.
    frame.push_arrows(&mut elements);
    frame.push_items(&mut elements, warnings, metrics);
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The days the axis runs from and to: the first and last of any item or marker, a day wider on
/// either side for a single day, and a little room at both ends.
fn axis_span(timeline: &TimelineSpec) -> (i64, i64) {
    let utc = crate::spec::timeline_zone();
    let (first, last) = timeline
        .items
        .iter()
        .flat_map(|item| [item.span().begin(), item.span().finish()])
        .chain(
            timeline
                .markers
                .iter()
                .map(|marker| marker.at.resolve(utc).expect("validated")),
        )
        .fold((i64::MAX, i64::MIN), |(first, last), day| {
            (first.min(day), last.max(day))
        });
    if first == last {
        (first - 86_400, last + 86_400)
    } else {
        let pad = (last - first) / 50;
        (first - pad, last + pad)
    }
}

impl Frame<'_> {
    /// Gridlines and the labels of the axis.
    fn push_axis(&self, elements: &mut Vec<Element>, _metrics: &impl TextMetrics) {
        let ticks = (self.plot_width / 64.0).max(2.0).floor();
        let max_ticks = usize::try_from(whole(ticks)).expect("small");
        let label_y = self.top + self.plot_height() + 22.0;
        for tick in time::ticks(
            self.first,
            self.last,
            zone(self.spec),
            max_ticks,
            self.with_time,
            None,
        ) {
            let x = self.x(tick.epoch);
            elements.push(Element::Line(Line {
                x1: x,
                y1: self.top,
                x2: x,
                y2: self.top + self.plot_height(),
                class: "chartlet-grid",
            }));
            elements.push(Element::Text(Text {
                x,
                y: label_y,
                class: "chartlet-tick",
                anchor: TextAnchor::Middle,
                content: tick.label,
            }));
        }
    }

    /// Dashed lines across all rows with their labels on top; they lie behind the items.
    fn push_markers(
        &self,
        elements: &mut Vec<Element>,
        warnings: &mut Vec<ChartWarning>,
        metrics: &impl TextMetrics,
    ) {
        let utc = crate::spec::timeline_zone();
        for (index, marker) in self.timeline.markers.iter().enumerate() {
            let x = self.x(marker.at.resolve(utc).expect("validated"));
            elements.push(Element::Line(Line {
                x1: x,
                y1: self.top - 6.0,
                x2: x,
                y2: self.top + self.plot_height(),
                class: "chartlet-timeline-marker",
            }));
            elements.push(Element::Text(Text {
                x,
                y: self.top - 10.0,
                class: "chartlet-timeline-marker-label",
                anchor: TextAnchor::Middle,
                content: fit_text(
                    &marker.label,
                    self.plot_width / 2.0,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &format!("/timeline/markers/{index}/label"),
                ),
            }));
        }
    }

    /// An arrow from every item to each item that follows it.
    fn push_arrows(&self, elements: &mut Vec<Element>) {
        for (index, item) in self.timeline.items.iter().enumerate() {
            for id in &item.after {
                let before = self.timeline.item(id).expect("validated");
                // An arrow leaves a diamond at its right tip and reaches one at its left tip.
                let out = match self.timeline.items[before].span() {
                    Span::Milestone { at } => self.x(at) + DIAMOND,
                    Span::Phase { end, .. } => self.x(end),
                };
                let into = match item.span() {
                    Span::Milestone { at } => self.x(at) - DIAMOND,
                    Span::Phase { start, .. } => self.x(start),
                };
                push_arrow(
                    elements,
                    (out, self.center(before)),
                    (into, self.center(index)),
                );
            }
        }
    }

    /// The bar or diamond of every item, its dates beside it and its name in the gutter.
    fn push_items(
        &self,
        elements: &mut Vec<Element>,
        warnings: &mut Vec<ChartWarning>,
        metrics: &impl TextMetrics,
    ) {
        let groups = self.timeline.groups();
        let class_of = |index: usize, phase: bool| {
            let group = self.timeline.items[index]
                .group
                .as_deref()
                .and_then(|group| groups.iter().position(|known| *known == group));
            match (group, phase) {
                (Some(group), true) => GROUP_CLASSES[group],
                (Some(group), false) => MILESTONE_CLASSES[group],
                (None, true) => "chartlet-timeline-phase",
                (None, false) => "chartlet-timeline-milestone",
            }
        };
        for (index, item) in self.timeline.items.iter().enumerate() {
            let y = self.center(index);
            let span = item.span();
            let tooltip = Some(format!(
                "{}: {}",
                item.label,
                days(self.spec, span, self.with_time)
            ));
            let label_end = match span {
                Span::Phase { start, end } => {
                    let (x1, x2) = (self.x(start), self.x(end));
                    elements.push(Element::Rect(Rect {
                        x: x1,
                        y: y - BAR / 2.0,
                        width: (x2 - x1).max(MIN_PHASE),
                        height: BAR,
                        class: class_of(index, true),
                        series_index: None,
                        style_index: None,
                        tooltip,
                    }));
                    x2.max(x1 + MIN_PHASE)
                }
                Span::Milestone { at } => {
                    let x = self.x(at);
                    elements.push(Element::Polyline(diagram::polyline(
                        vec![
                            (x - DIAMOND, y),
                            (x, y - DIAMOND),
                            (x + DIAMOND, y),
                            (x, y + DIAMOND),
                        ],
                        class_of(index, false),
                        tooltip,
                    )));
                    x + DIAMOND
                }
            };
            if self.dates {
                self.push_dates(elements, index, label_end, metrics);
            }
            elements.push(Element::Text(Text {
                x: self.left - 12.0,
                y: y + 4.5,
                class: "chartlet-label",
                anchor: TextAnchor::End,
                content: fit_text(
                    &item.label,
                    self.gutter - 16.0,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &format!("/timeline/items/{index}/label"),
                ),
            }));
        }
    }

    /// The days of item `index` beside it: on its right, past the arrows that leave it, and on
    /// its left when they do not fit there.
    fn push_dates(
        &self,
        elements: &mut Vec<Element>,
        index: usize,
        label_end: f64,
        metrics: &impl TextMetrics,
    ) {
        let item = &self.timeline.items[index];
        let content = days(self.spec, item.span(), self.with_time);
        let reach = metrics.width(&content, LABEL_SIZE);
        let followed = self.timeline.items.iter().any(|other| {
            other
                .after
                .iter()
                .any(|id| Some(id.as_str()) == item.id.as_deref())
        });
        let start = label_end + if followed { TURN + 8.0 } else { 8.0 };
        let (x, anchor) = if start + reach <= self.width() - self.margin {
            (start, TextAnchor::Start)
        } else {
            (self.x(item.span().begin()) - DIAMOND - 4.0, TextAnchor::End)
        };
        elements.push(Element::Text(Text {
            x,
            y: self.center(index) + 4.0,
            class: "chartlet-value",
            anchor,
            content,
        }));
    }
}

/// A small whole number from a float that was floored.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const fn whole(value: f64) -> u64 {
    value as u64
}

/// The arrow from where an item ends to where the next one starts: out to the right, across to
/// the row of the next item and in to its start.
fn push_arrow(elements: &mut Vec<Element>, from: (f64, f64), to: (f64, f64)) {
    let points = if to.0 >= from.0 + 2.0 * TURN {
        vec![
            from,
            (from.0 + TURN, from.1),
            (from.0 + TURN, to.1),
            (to.0 - diagram::HEAD, to.1),
        ]
    } else {
        // The next item starts before this one ends: round the long way, between the two rows.
        let middle = f64::midpoint(from.1, to.1);
        vec![
            from,
            (from.0 + TURN, from.1),
            (from.0 + TURN, middle),
            (to.0 - TURN, middle),
            (to.0 - TURN, to.1),
            (to.0 - diagram::HEAD, to.1),
        ]
    };
    elements.push(Element::Polyline(diagram::polyline(
        rounded(&points),
        "chartlet-timeline-link",
        None,
    )));
    elements.push(Element::Polyline(arrowhead(
        (to.0, to.1),
        (1.0, 0.0),
        true,
        "chartlet-timeline-head",
    )));
}

/// The legend of the groups, one row or more under the title; returns where it ends.
fn push_legend(
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    spec: &ChartSpec,
    (left, top): (f64, f64),
    metrics: &impl TextMetrics,
) -> f64 {
    let groups = timeline(spec).groups();
    if groups.is_empty() {
        return top;
    }
    let right = f64::from(spec.width) - CONTENT_LEFT.min(left);
    let (mut x, mut y) = (left, top);
    for (index, group) in groups.iter().enumerate() {
        let reach = LEGEND_SWATCH + metrics.width(group, LABEL_SIZE);
        if x > left && x + reach > right {
            x = left;
            y += 20.0;
        }
        elements.push(Element::Rect(Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
            class: GROUP_CLASSES[index],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: x + LEGEND_SWATCH,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                group,
                (right - x - LEGEND_SWATCH).max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/timeline/items/{}/group", first_with(spec, group)),
            ),
        }));
        x += reach + LEGEND_GAP;
    }
    y + 26.0
}

fn first_with(spec: &ChartSpec, group: &str) -> usize {
    timeline(spec)
        .items
        .iter()
        .position(|item| item.group.as_deref() == Some(group))
        .expect("a group comes from an item")
}

/// The timeline in sentences: how many items it has and over which days, then every item in
/// order with its days and what it follows.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let timeline = timeline(spec);
    let with_time = has_time_of_day(timeline);
    let phases = timeline
        .items
        .iter()
        .filter(|item| matches!(item.span(), Span::Phase { .. }))
        .count();
    let (first, last) = timeline
        .items
        .iter()
        .fold((i64::MAX, i64::MIN), |(first, last), item| {
            (
                first.min(item.span().begin()),
                last.max(item.span().finish()),
            )
        });
    let mut description = text::timeline_opening(
        spec.locale,
        (phases, timeline.items.len() - phases),
        (
            &write_day(spec, first, with_time),
            &write_day(spec, last, with_time),
        ),
    );
    for item in &timeline.items {
        let follows: Vec<&str> = item
            .after
            .iter()
            .map(|id| {
                timeline.items[timeline.item(id).expect("validated")]
                    .label
                    .as_str()
            })
            .collect();
        description.push(' ');
        description.push_str(&text::timeline_item(
            spec.locale,
            &item.label,
            &days(spec, item.span(), with_time),
            &follows,
        ));
    }
    if !timeline.markers.is_empty() {
        let utc = crate::spec::timeline_zone();
        let markers: Vec<String> = timeline
            .markers
            .iter()
            .map(|marker| {
                format!(
                    "{} ({})",
                    marker.label,
                    write_day(spec, marker.at.resolve(utc).expect("validated"), with_time)
                )
            })
            .collect();
        description.push_str(&text::timeline_markers(spec.locale, &markers.join(", ")));
    }
    description
}

/// One row per item in order: its kind, days, group and what it follows.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let timeline = timeline(spec);
    let words = spec.locale.words();
    let with_time = has_time_of_day(timeline);
    let grouped = !timeline.groups().is_empty();
    let followed = timeline.items.iter().any(|item| !item.after.is_empty());
    let mut columns = vec![
        words.item.to_owned(),
        words.message_kind.to_owned(),
        words.begin.to_owned(),
        words.finish.to_owned(),
    ];
    if grouped {
        columns.push(words.group.to_owned());
    }
    if followed {
        columns.push(words.follows.to_owned());
    }
    let rows = timeline
        .items
        .iter()
        .map(|item| {
            let span = item.span();
            let (kind, begin, finish) = match span {
                Span::Phase { start, end } => (
                    words.phase,
                    write_day(spec, start, with_time),
                    write_day(spec, end, with_time),
                ),
                Span::Milestone { at } => (
                    words.milestone,
                    write_day(spec, at, with_time),
                    String::new(),
                ),
            };
            let mut row = vec![item.label.clone(), kind.to_owned(), begin, finish];
            if grouped {
                row.push(item.group.clone().unwrap_or_default());
            }
            if followed {
                row.push(
                    item.after
                        .iter()
                        .map(|id| {
                            timeline.items[timeline.item(id).expect("validated")]
                                .label
                                .as_str()
                        })
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
