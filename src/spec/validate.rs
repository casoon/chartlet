use super::untagged::NO_VARIANT;
use super::{
    AxisScale, BoxDisplay, ChartSpec, ChartType, LegendPlacement, MAX_AXIS_TICKS, MAX_DECIMALS,
    Mark, Orientation, PaneSpec, RangeDisplay, RangeSort, Stack, Theme, Tooltips, ValueAxisSpec,
    ValueFormat, type_name,
};
use crate::error::ChartError;

impl ChartSpec {
    /// The blueprint look is made for diagrams; a chart with axes and series keeps light or dark.
    pub(super) fn validate_theme(&self) -> Result<(), ChartError> {
        crate::theme::validate(&self.tokens)?;
        if let Some(typography) = &self.typography {
            typography.validate()?;
        }
        let diagram = matches!(
            self.chart_type,
            ChartType::Sequence
                | ChartType::Flow
                | ChartType::State
                | ChartType::Architecture
                | ChartType::Tree
                | ChartType::Classes
                | ChartType::Network
                | ChartType::Cycle
                | ChartType::Layers
                | ChartType::Plan
        );
        if self.theme == Theme::Blueprint && !diagram {
            return Err(ChartError::new(
                "option_not_supported",
                "/theme",
                "the blueprint theme is made for diagrams (sequence, flow, state, architecture, tree, classes, network, cycle, layers, plan); use light or dark",
            ));
        }
        if !self.tags && !diagram {
            return Err(ChartError::new(
                "option_not_supported",
                "/tags",
                "type tags belong to diagrams (flow, architecture); remove `tags` or use a diagram type",
            ));
        }
        Ok(())
    }

    /// Every value a value axis has to place, with its path: the values, band and zone edges,
    /// candles, reference lines and point markers, by the axis they belong to — `None` for the
    /// top-level value axis, `Some(pane)` for the own axis of a time chart's pane. Small multiples
    /// share the top-level axis.
    fn axis_values(&self) -> Vec<(Option<usize>, f64, String)> {
        let mut values = Vec::new();
        for (index, point) in self.data.iter().enumerate() {
            if let Some(value) = point.value {
                values.push((None, value, format!("/data/{index}/value")));
            }
        }
        for (series_index, series) in self.series.iter().enumerate() {
            for (index, value) in series.values.iter().enumerate() {
                if let Some(value) = value {
                    values.push((
                        None,
                        *value,
                        format!("/series/{series_index}/values/{index}"),
                    ));
                }
            }
        }
        for (index, point) in self.data.iter().enumerate() {
            for (name, value) in [("lower", point.lower), ("upper", point.upper)] {
                if let Some(value) = value {
                    values.push((None, value, format!("/data/{index}/{name}")));
                }
            }
        }
        for (index, reference) in self.references.iter().enumerate() {
            values.push((None, reference.value, format!("/references/{index}/value")));
        }
        for (index, range) in self.ranges.iter().enumerate() {
            for (name, value) in [
                ("low", Some(range.low)),
                ("high", Some(range.high)),
                ("mid", range.mid),
            ] {
                if let Some(value) = value {
                    values.push((None, value, format!("/ranges/{index}/{name}")));
                }
            }
        }
        for (pane_index, pane) in self.panes.iter().enumerate() {
            let axis = (self.chart_type != ChartType::Multiples || self.is_bar_multiples())
                .then_some(pane_index);
            for (index, value) in pane.values.iter().enumerate() {
                if let Some(value) = value {
                    values.push((axis, *value, format!("/panes/{pane_index}/values/{index}")));
                }
            }
            for (layer_index, layer) in pane.layers.iter().enumerate() {
                let path = format!("/panes/{pane_index}/layers/{layer_index}");
                for (index, point) in layer.points.iter().enumerate() {
                    for (name, value) in [
                        ("value", point.value),
                        ("lower", point.lower),
                        ("upper", point.upper),
                    ] {
                        if let Some(value) = value {
                            values.push((axis, value, format!("{path}/points/{index}/{name}")));
                        }
                    }
                }
                for (index, candle) in layer.data.iter().enumerate() {
                    values.push((axis, candle.low, format!("{path}/data/{index}/low")));
                    values.push((axis, candle.high, format!("{path}/data/{index}/high")));
                }
                for (name, value) in [
                    ("value", layer.value),
                    ("bottom", layer.bottom),
                    ("top", layer.top),
                ] {
                    if let Some(value) = value {
                        values.push((axis, value, format!("{path}/{name}")));
                    }
                }
            }
        }
        values
    }

    /// The value axes and their paths: the top-level one, and the own axis of every time chart
    /// pane.
    fn value_axes(&self) -> Vec<(Option<usize>, &ValueAxisSpec, String)> {
        let mut axes = vec![(None, &self.value_axis, "/valueAxis".to_owned())];
        if self.chart_type == ChartType::Time || self.is_bar_multiples() {
            for (index, pane) in self.panes.iter().enumerate() {
                axes.push((
                    Some(index),
                    &pane.value_axis,
                    format!("/panes/{index}/valueAxis"),
                ));
            }
        }
        axes
    }

    /// The positive and the negative total of every category of a stacked bar chart.
    fn stack_totals(&self) -> Vec<f64> {
        (0..self.categories.len())
            .flat_map(|index| {
                let values = self.series.iter().filter_map(|series| series.values[index]);
                let positive: f64 = values.clone().filter(|value| *value > 0.0).sum();
                let negative: f64 = values.filter(|value| *value < 0.0).sum();
                [positive, negative]
            })
            .collect()
    }

    /// A declared value step yields at most [`MAX_AXIS_TICKS`] ticks across the values and the
    /// declared ends, and an exact axis at least two: a tiny step would otherwise draw millions.
    fn validate_tick_count(
        &self,
        (axis, axis_index): (&ValueAxisSpec, Option<usize>),
        step: f64,
        values: &[(Option<usize>, f64, String)],
        path: &str,
    ) -> Result<(), ChartError> {
        let own = values
            .iter()
            .filter(|(owner, _, _)| *owner == axis_index)
            .map(|(_, value, _)| *value);
        // A stacked pane reaches the totals of its areas.
        let zone = self.time_zone().unwrap_or_default();
        let stacked: Vec<f64> = match axis_index {
            Some(pane) if self.panes[pane].stack.is_some() => self
                .data_layers()
                .filter(|entry| entry.pane == pane)
                .flat_map(|entry| self.drawn_points(entry, zone))
                .map(|(_, value)| value)
                // Stacked areas grow from zero.
                .chain([0.0])
                .collect(),
            _ => Vec::new(),
        };
        let mut values: Vec<f64> = own.chain(stacked).chain(axis.min).chain(axis.max).collect();
        // Bars are measured from zero; a stack reaches the totals of its categories, a percent
        // stack runs from 0 to 1.
        if self.chart_type == ChartType::Bar {
            values.push(0.0);
            match self.stack {
                Some(Stack::Percent) => values = vec![0.0, 1.0],
                Some(Stack::Normal | Stack::Stream) => values.extend(self.stack_totals()),
                None => {}
            }
        }
        let (low, high) = values
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                (low.min(*value), high.max(*value))
            });
        if !low.is_finite() {
            return Ok(());
        }
        let ticks = (high / step).floor() - (low / step).ceil() + 1.0;
        let few = axis.exact && ticks < 2.0;
        if ticks > MAX_AXIS_TICKS || few {
            return Err(ChartError::new(
                "invalid_axis_range",
                format!("{path}/step"),
                format!(
                    "a step of {step} gives {} ticks across {} to {}; choose one that gives 2 to {MAX_AXIS_TICKS}",
                    ticks.max(0.0),
                    low + 0.0,
                    high + 0.0
                ),
            ));
        }
        Ok(())
    }

    /// A logarithmic axis places only values above zero, and neither a stack nor an area, which
    /// are measured from zero. A declared `step` is a positive distance. An `exact` axis has both
    /// ends, and every value lies between them, so that nothing reaches outside the plot.
    pub(super) fn validate_value_axes(&self) -> Result<(), ChartError> {
        let values = self.axis_values();
        for (axis_index, axis, path) in self.value_axes() {
            let log = axis.scale == AxisScale::Log;
            let refuse = |field: &str, reason: &str| {
                Err(ChartError::new(
                    "option_not_supported",
                    format!("{path}/{field}"),
                    reason.to_owned(),
                ))
            };
            if let Some(unit) = &axis.unit {
                if !matches!(self.chart_type, ChartType::Time | ChartType::Multiples) {
                    return refuse(
                        "unit",
                        "a unit at the top tick belongs to a time chart or small multiples; use the axis title",
                    );
                }
                validate_text(unit, &format!("{path}/unit"), 20)?;
            }
            if log && self.stack.is_some() {
                return refuse(
                    "scale",
                    "a stack is measured from zero and takes no logarithmic axis",
                );
            }
            if log && (axis.step.is_some() || axis.exact) {
                return refuse("step", "a logarithmic axis ticks at powers of ten");
            }
            if axis.exact && self.stack.is_some() {
                return refuse("exact", "a stack's totals decide its axis");
            }
            if let Some(step) = axis.step {
                validate_number(step, &format!("{path}/step"))?;
                if step <= 0.0 {
                    return Err(ChartError::new(
                        "invalid_value",
                        format!("{path}/step"),
                        "the step between ticks must be above zero",
                    ));
                }
            }
            if let (true, Some(min), Some(max)) = (axis.exact, axis.min, axis.max)
                && self.chart_type == ChartType::Bar
                && !(min..=max).contains(&0.0)
            {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    format!("{path}/min"),
                    "bars start at zero, so an exact axis of a bar chart includes zero",
                ));
            }
            if axis.exact && (axis.min.is_none() || axis.max.is_none()) {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    format!("{path}/exact"),
                    "an exact axis needs both min and max",
                ));
            }
            if log {
                positive_bounds(axis, &path)?;
            }
            for (_, value, value_path) in values.iter().filter(|(owner, _, _)| *owner == axis_index)
            {
                if log {
                    positive(*value, value_path.clone())?;
                }
                if let (true, Some(min), Some(max)) = (axis.exact, axis.min, axis.max)
                    && !(min..=max).contains(value)
                {
                    return Err(ChartError::new(
                        "value_outside_axis",
                        value_path.clone(),
                        format!("the value lies outside the exact axis from {min} to {max}"),
                    ));
                }
            }
            if let Some(step) = axis.step {
                self.validate_tick_count((axis, axis_index), step, &values, &path)?;
            }
            if log {
                let panes: Vec<usize> = match axis_index {
                    Some(pane) => vec![pane],
                    None if self.chart_type == ChartType::Multiples => {
                        (0..self.panes.len()).collect()
                    }
                    None => Vec::new(),
                };
                for pane in panes {
                    reject_log_area(&self.panes[pane], pane)?;
                }
            }
        }
        Ok(())
    }

    /// Fixed decimals stay in a readable range; a declared axis range is finite and ordered.
    fn validate_decimals(&self) -> Result<(), ChartError> {
        let axes = std::iter::once(("/valueAxis/decimals".to_owned(), &self.value_axis)).chain(
            self.panes.iter().enumerate().map(|(index, pane)| {
                (
                    format!("/panes/{index}/valueAxis/decimals"),
                    &pane.value_axis,
                )
            }),
        );
        for (path, axis) in axes {
            let base = path.trim_end_matches("/decimals");
            for (name, bound) in [("min", axis.min), ("max", axis.max)] {
                if let Some(bound) = bound {
                    validate_number(bound, &format!("{base}/{name}"))?;
                }
            }
            if let (Some(min), Some(max)) = (axis.min, axis.max)
                && min >= max
            {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    format!("{base}/max"),
                    "valueAxis.max must be greater than valueAxis.min",
                ));
            }
            if axis
                .decimals
                .is_some_and(|decimals| decimals > MAX_DECIMALS)
            {
                return Err(ChartError::new(
                    "invalid_decimals",
                    path,
                    format!("use 0 to {MAX_DECIMALS} decimal places"),
                ));
            }
        }
        Ok(())
    }

    /// The size of the chart and of its mobile variant.
    fn validate_size(&self) -> Result<(), ChartError> {
        if self.sparkline && self.chart_type == ChartType::Time {
            for (path, value, range) in [
                ("/width", self.width, 60..=600),
                ("/height", self.height, 16..=200),
            ] {
                if !range.contains(&value) {
                    return Err(ChartError::new(
                        "invalid_dimension",
                        path,
                        format!(
                            "a sparkline's {} must be between {} and {}",
                            &path[1..],
                            range.start(),
                            range.end()
                        ),
                    ));
                }
            }
            return Ok(());
        }
        if !(200..=2_400).contains(&self.width) {
            return Err(ChartError::new(
                "invalid_dimension",
                "/width",
                "width must be between 200 and 2400",
            ));
        }
        if !(160..=1_600).contains(&self.height) {
            return Err(ChartError::new(
                "invalid_dimension",
                "/height",
                "height must be between 160 and 1600",
            ));
        }
        if let Some(mobile) = &self.mobile {
            for (path, value, range) in [
                ("/mobile/width", mobile.width, 200..=600),
                ("/mobile/height", mobile.height, 160..=1_600),
                ("/mobile/breakpoint", mobile.breakpoint, 320..=1_600),
            ] {
                if !range.contains(&value) {
                    return Err(ChartError::new(
                        "invalid_dimension",
                        path,
                        format!(
                            "{} must be between {} and {}",
                            &path[1..],
                            range.start(),
                            range.end()
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_metadata(&self, check_size: bool) -> Result<(), ChartError> {
        if self.schema_version != 1 {
            return Err(ChartError::new(
                "unsupported_schema_version",
                "/schemaVersion",
                "expected schemaVersion 1",
            ));
        }
        if self.chart_type == ChartType::Line && self.orientation != Orientation::Vertical {
            return Err(ChartError::new(
                "option_not_supported",
                "/orientation",
                "orientation is only available for bar charts",
            ));
        }
        self.validate_decimals()?;
        validate_text(&self.title, "/title", 200)?;
        if let Some(description) = &self.description {
            validate_text(description, "/description", 1_000)?;
        }
        if let Some(source) = &self.source {
            validate_text(source, "/source", 300)?;
        }
        validate_optional_text(
            self.category_axis.title.as_ref(),
            "/categoryAxis/title",
            100,
        )?;
        validate_optional_text(self.value_axis.title.as_ref(), "/valueAxis/title", 100)?;

        if check_size {
            self.validate_size()?;
        }
        self.validate_zoom_steps()
    }

    /// Rejects a block that belongs to another chart type, so that it is never silently ignored.
    #[allow(clippy::too_many_lines)]
    pub(super) fn reject_foreign_blocks(&self) -> Result<(), ChartError> {
        let own = self.chart_type;
        for (index, pane) in self.panes.iter().enumerate() {
            let Some(note) = &pane.note else {
                if pane.note_emphasis {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("/panes/{index}/noteEmphasis"),
                        "noteEmphasis needs a note",
                    ));
                }
                continue;
            };
            let path = format!("/panes/{index}/note");
            if own != ChartType::Multiples {
                return Err(ChartError::new(
                    "option_not_supported",
                    path,
                    "a note sits under the title of a small-multiples panel",
                ));
            }
            validate_text(note, &path, 160)?;
        }
        if own != ChartType::Time
            && let Some(index) = self.panes.iter().position(|pane| pane.stack.is_some())
        {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{index}/stack"),
                "stacked areas belong to the pane of a time chart",
            ));
        }
        for (field, present, owner) in [
            ("/stripes", self.stripes.is_some(), ChartType::Stripes),
            ("/calendar", self.calendar.is_some(), ChartType::Calendar),
            ("/ranges", !self.ranges.is_empty(), ChartType::Rangebar),
            ("/boxes", !self.boxes.is_empty(), ChartType::Boxplot),
            ("/showMean", self.show_mean, ChartType::Boxplot),
            (
                "/violinScale",
                !self.violin_scale.is_width(),
                ChartType::Boxplot,
            ),
            ("/showBox", self.show_box, ChartType::Boxplot),
            (
                "/boxDisplay",
                self.box_display != BoxDisplay::Box,
                ChartType::Boxplot,
            ),
            (
                "/display",
                self.display != RangeDisplay::Bar,
                ChartType::Rangebar,
            ),
            ("/sort", self.sort != RangeSort::None, ChartType::Rangebar),
            ("/timeline", self.timeline.is_some(), ChartType::Timeline),
            ("/classes", self.classes.is_some(), ChartType::Classes),
            ("/network", self.network.is_some(), ChartType::Network),
            ("/cycle", self.cycle.is_some(), ChartType::Cycle),
            ("/layers", self.layer_stack.is_some(), ChartType::Layers),
            ("/plan", self.plan.is_some(), ChartType::Plan),
            ("/scatter", self.scatter.is_some(), ChartType::Scatter),
            ("/survival", self.survival.is_some(), ChartType::Survival),
            ("/sankey", self.sankey.is_some(), ChartType::Sankey),
            ("/treemap", self.treemap.is_some(), ChartType::Treemap),
            (
                "/parliament",
                self.parliament.is_some(),
                ChartType::Parliament,
            ),
            ("/waffle", self.waffle.is_some(), ChartType::Waffle),
            ("/funnel", self.funnel.is_some(), ChartType::Funnel),
            ("/radar", self.radar.is_some(), ChartType::Radar),
            ("/polar", self.polar.is_some(), ChartType::Polar),
            ("/mosaic", self.mosaic.is_some(), ChartType::Mosaic),
            ("/heatmap", self.heatmap.is_some(), ChartType::Heatmap),
            ("/venn", self.venn.is_some(), ChartType::Venn),
            ("/valuemap", self.valuemap.is_some(), ChartType::Valuemap),
            ("/journey", self.journey.is_some(), ChartType::Journey),
            ("/storymap", self.storymap.is_some(), ChartType::Storymap),
            ("/kanban", self.kanban.is_some(), ChartType::Kanban),
            ("/fishbone", self.fishbone.is_some(), ChartType::Fishbone),
            ("/waterfall", self.waterfall.is_some(), ChartType::Waterfall),
            ("/sequence", self.sequence.is_some(), ChartType::Sequence),
            ("/flow", self.flow.is_some(), ChartType::Flow),
            ("/state", self.state.is_some(), ChartType::State),
            (
                "/architecture",
                self.architecture.is_some(),
                ChartType::Architecture,
            ),
            ("/tree", self.tree.is_some(), ChartType::Tree),
            ("/columns", self.columns.is_some(), ChartType::Multiples),
            (
                "/references",
                !self.references.is_empty() && own != ChartType::Rangebar,
                ChartType::Bar,
            ),
            ("/stack", self.stack.is_some(), ChartType::Bar),
            (
                "/legend",
                self.legend == LegendPlacement::End,
                ChartType::Time,
            ),
            ("/sparkline", self.sparkline, ChartType::Time),
            (
                "/tooltips",
                self.tooltips == Tooltips::Markers,
                ChartType::Time,
            ),
            ("/patterns", self.patterns, ChartType::Bar),
            ("/slope", self.slope, ChartType::Line),
            ("/rank", self.rank, ChartType::Line),
            (
                "/independentAxes",
                self.independent_axes,
                ChartType::Multiples,
            ),
            (
                "/mobile/columns",
                self.mobile
                    .as_ref()
                    .is_some_and(|mobile| mobile.columns.is_some()),
                ChartType::Multiples,
            ),
        ] {
            if present && own != owner {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    format!(
                        "{field} belongs to a {} chart; remove it or change the type",
                        type_name(owner)
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Rejects the fields that only a bar, line, or time chart has, naming the field and the fix.
    /// Neither map has axes, series or panes. `noun` and `source` name the type in the message,
    /// so the error says what to do rather than only what is wrong.
    pub(super) fn reject_map_options(&self, noun: &str, source: &str) -> Result<(), ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    format!("{noun} is drawn from {source}; remove this field"),
                ));
            }
        }
        if self.orientation != Orientation::Vertical {
            return Err(ChartError::new(
                "option_not_supported",
                "/orientation",
                "orientation is only available for bar charts",
            ));
        }
        if self.category_axis.title.is_some() {
            return Err(ChartError::new(
                "option_not_supported",
                "/categoryAxis/title",
                format!("{noun} has no category axis"),
            ));
        }
        if self.value_axis.title.is_some()
            || self.value_axis.format != ValueFormat::Number
            || self.value_axis.min.is_some()
            || self.value_axis.max.is_some()
            || self.value_axis.thousands_separator
            || self.value_axis.scale != AxisScale::Linear
            || self.value_axis.reverse
            || self.value_axis.step.is_some()
            || self.value_axis.exact
        {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis",
                format!("{noun} has no value axis"),
            ));
        }
        if !self.time_axis.is_default() {
            return Err(ChartError::new(
                "option_not_supported",
                "/timeAxis",
                format!("{noun} has no time axis"),
            ));
        }
        Ok(())
    }
}

/// The JSON Pointer of the value a data error of `serde_json` is about, such as `/data/0/value`
/// (the document root is `/`), and the message to report.
///
/// `serde_json` gives such an error the position of the last byte it read rather than a path. A
/// scan of the input up to that byte finds the members around it, which make up the path. Some
/// errors are about the object or array itself, not about the member at that byte: those raised
/// once all of it has been read, and those at a key that repeats or that names no variant of an
/// enum written as an object.
pub(super) fn json_pointer(input: &str, error: &serde_json::Error) -> (String, String) {
    let bytes = input.as_bytes();
    // serde_json counts lines from 1 and columns in bytes: `column` bytes of the line are read.
    let read = bytes
        .split_inclusive(|byte| *byte == b'\n')
        .take(error.line().saturating_sub(1))
        .map(<[u8]>::len)
        .sum::<usize>()
        + error.column();
    let message = error.to_string();
    let untagged = message.starts_with(NO_VARIANT);
    let about_container = untagged
        || [
            "missing field",
            "duplicate field",
            "invalid length",
            "unknown variant",
        ]
        .iter()
        .any(|start| message.starts_with(start));
    let (frames, closed) = scan(bytes, read);
    let drop_member = about_container && (closed || frames.last().is_some_and(Frame::before_value));
    let pointer = pointer(&frames, drop_member);
    if !untagged {
        return (pointer, message);
    }
    // The values read by `untagged` fail at their own end. The message keeps the position these
    // errors have always had: where the object or array around the value, which used to report
    // them, has read on to its next member or past its own end.
    let in_array = matches!(
        frames.iter().rev().nth(usize::from(closed)),
        Some(Frame::Array { .. })
    );
    let mut at = skip_whitespace(bytes, read);
    at = match bytes.get(at) {
        Some(b'}') if !in_array => at + 1,
        Some(b']') if in_array => at + 1,
        Some(b',') if in_array => skip_whitespace(bytes, at + 1),
        _ => at,
    };
    let line_start = bytes[..at]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |newline| newline + 1);
    let line = bytes[..at].split(|byte| *byte == b'\n').count();
    let text = message
        .rsplit_once(" at line ")
        .map_or(message.as_str(), |(text, _)| text);
    let message = format!("{text} at line {line} column {}", at - line_start);
    (pointer, message)
}

/// An object or array around the byte an error is at, as [`scan`] tracks it.
#[derive(Debug, PartialEq)]
enum Frame {
    /// The key of the member being read, `None` before the first key and after a comma, and
    /// whether that member's value has begun.
    Object { key: Option<String>, value: bool },
    /// The index of the element being read.
    Array { index: usize },
}

impl Frame {
    /// Between a key and its value: where an error about the key itself sits.
    fn before_value(&self) -> bool {
        matches!(
            self,
            Self::Object {
                key: Some(_),
                value: false
            }
        )
    }
}

/// The objects and arrays around the last of the first `read` bytes of `bytes`, outermost
/// first, and whether that byte closes the innermost one, which is then still in the list. The
/// input past that byte need not be valid JSON.
fn scan(bytes: &[u8], read: usize) -> (Vec<Frame>, bool) {
    let mut frames: Vec<Frame> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b':') {
            at += 1;
            continue;
        }
        if at >= read {
            // The error lies before this token, in the whitespace after the previous one.
            break;
        }
        match byte {
            b'{' | b'[' => {
                begin_value(&mut frames);
                frames.push(if byte == b'{' {
                    Frame::Object {
                        key: None,
                        value: false,
                    }
                } else {
                    Frame::Array { index: 0 }
                });
                at += 1;
            }
            b'}' | b']' => {
                if at + 1 == read {
                    return (frames, true);
                }
                frames.pop();
                at += 1;
            }
            b',' => {
                match frames.last_mut() {
                    Some(Frame::Object { key, value }) => (*key, *value) = (None, false),
                    Some(Frame::Array { index }) => *index += 1,
                    None => {}
                }
                at += 1;
            }
            _ => {
                let end = token_end(bytes, at);
                if let Some(Frame::Object {
                    key: key @ None, ..
                }) = frames.last_mut()
                {
                    // A key, with its escapes resolved: `"a\/b"` is the key `a/b`.
                    *key = Some(serde_json::from_slice(&bytes[at..end]).unwrap_or_default());
                } else {
                    begin_value(&mut frames);
                }
                if end >= read {
                    break;
                }
                at = end;
            }
        }
    }
    (frames, false)
}

/// Marks the value of the member being read in the innermost object as begun.
fn begin_value(frames: &mut [Frame]) {
    if let Some(Frame::Object { value, .. }) = frames.last_mut() {
        *value = true;
    }
}

/// One past the last byte of the string, number or literal that starts at `start`.
fn token_end(bytes: &[u8], start: usize) -> usize {
    if bytes[start] != b'"' {
        return (start..bytes.len())
            .find(|&at| {
                matches!(
                    bytes[at],
                    b',' | b'}' | b']' | b':' | b' ' | b'\t' | b'\n' | b'\r'
                )
            })
            .unwrap_or(bytes.len());
    }
    // In a string, a backslash escapes the next byte, so `\"` does not end it. Brackets are
    // plain text, and so are the bytes of other characters, which are all 0x80 and above.
    let mut at = start + 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b'"' => return at + 1,
            _ => at += 1,
        }
    }
    bytes.len()
}

fn skip_whitespace(bytes: &[u8], mut at: usize) -> usize {
    while bytes
        .get(at)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
    {
        at += 1;
    }
    at
}

/// The JSON Pointer of the member being read in each frame, without the innermost member with
/// `drop_member`. `~` is written `~0` and `/` is written `~1`.
fn pointer(frames: &[Frame], drop_member: bool) -> String {
    let count = frames.len() - usize::from(drop_member && !frames.is_empty());
    let mut pointer = String::new();
    for frame in &frames[..count] {
        match frame {
            Frame::Object { key: Some(key), .. } => {
                pointer.push('/');
                pointer.push_str(&key.replace('~', "~0").replace('/', "~1"));
            }
            Frame::Object { key: None, .. } => {}
            Frame::Array { index } => {
                pointer.push('/');
                pointer.push_str(&index.to_string());
            }
        }
    }
    if pointer.is_empty() {
        "/".to_owned()
    } else {
        pointer
    }
}

/// A value on a logarithmic axis: above zero.
pub(super) fn positive(value: f64, path: String) -> Result<(), ChartError> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(ChartError::new(
            "invalid_value",
            path,
            "a logarithmic axis takes values above zero only",
        ))
    }
}

/// The declared range of a logarithmic axis: above zero.
pub(super) fn positive_bounds(axis: &ValueAxisSpec, path: &str) -> Result<(), ChartError> {
    for (name, bound) in [("min", axis.min), ("max", axis.max)] {
        if let Some(bound) = bound {
            positive(bound, format!("{path}/{name}"))?;
        }
    }
    Ok(())
}

/// An area is filled down to zero, so a pane with one takes no logarithmic axis.
fn reject_log_area(pane: &PaneSpec, pane_index: usize) -> Result<(), ChartError> {
    for (layer_index, layer) in pane.layers.iter().enumerate() {
        if layer.mark == Mark::Area {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{pane_index}/layers/{layer_index}/mark"),
                "an area is filled down to zero and takes no logarithmic axis",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_number(value: f64, path: &str) -> Result<(), ChartError> {
    if !value.is_finite() {
        return Err(ChartError::new(
            "non_finite_value",
            path,
            "value must be finite",
        ));
    }
    let magnitude = value.abs();
    if magnitude > 1e100 || (magnitude > 0.0 && magnitude < 1e-100) {
        return Err(ChartError::new(
            "unsupported_numeric_range",
            path,
            "value magnitude must be zero or between 1e-100 and 1e100",
        ));
    }
    Ok(())
}

pub(super) fn validate_optional_text(
    value: Option<&String>,
    path: &str,
    max_length: usize,
) -> Result<(), ChartError> {
    if let Some(value) = value {
        validate_text(value, path, max_length)?;
    }
    Ok(())
}

pub(super) fn validate_text(value: &str, path: &str, max_length: usize) -> Result<(), ChartError> {
    let length = value.chars().count();
    if value.trim().is_empty() {
        return Err(ChartError::new(
            "empty_text",
            path,
            "value must contain visible text",
        ));
    }
    if length > max_length {
        return Err(ChartError::new(
            "text_too_long",
            path,
            format!("value must not exceed {max_length} characters"),
        ));
    }
    if value.chars().any(|character| !is_xml_character(character)) {
        return Err(ChartError::new(
            "invalid_xml_character",
            path,
            "text contains a control character that XML 1.0 cannot represent",
        ));
    }
    Ok(())
}

fn is_xml_character(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
        || ('\u{20}'..='\u{D7FF}').contains(&character)
        || ('\u{E000}'..='\u{FFFD}').contains(&character)
        || ('\u{10000}'..='\u{10FFFF}').contains(&character)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::json_pointer;
    use crate::spec::{MainPath, ZoomBound};
    use crate::time::TimeValue;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct Document {
        #[serde(default)]
        label: String,
        #[serde(default)]
        items: Vec<Item>,
        #[serde(default, rename = "a/b~c")]
        odd: Option<f64>,
        #[serde(default)]
        path: Option<MainPath>,
        #[serde(default)]
        when: Option<TimeValue>,
        #[serde(default)]
        bound: Option<ZoomBound>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct Item {
        n: f64,
    }

    /// The path and message of the error `input` raises.
    fn located(input: &str) -> (String, String) {
        let error = serde_json::from_str::<Document>(input).unwrap_err();
        json_pointer(input, &error)
    }

    fn path(input: &str) -> String {
        located(input).0
    }

    #[test]
    fn finds_values_in_nested_objects_and_arrays() {
        assert_eq!(path(r#"{"items":[{"n":1},{"n":"x"}]}"#), "/items/1/n");
        assert_eq!(path(r#"{"odd":1,"a/b~c":"x"}"#), "/odd");
        assert_eq!(path(r#"{"a/b~c":"x"}"#), "/a~1b~0c");
        assert_eq!(path(r#"{"a\/b\u007ec":"x"}"#), "/a~1b~0c");
    }

    #[test]
    fn reads_pretty_and_compact_json_alike() {
        let pretty =
            "{\n  \"items\": [\n    { \"n\": 1 },\n    {\r\n\t\"n\" : \"x\"\n    }\n  ]\n}\n";
        assert_eq!(path(pretty), "/items/1/n");
        assert_eq!(path(r#"{"items":[{"n":1},{"n":"x"}]}"#), path(pretty));
    }

    #[test]
    fn skips_strings_with_escapes_brackets_and_other_characters() {
        let input = r#"{"label":"a\"]},[{\\\"","items":[{"n":true}]}"#;
        assert_eq!(path(input), "/items/0/n");
        let input = r#"{"label":"Grüße ✓ 😀 \u00e4","Grüße":1,"items":[{"n":1},{"n":"ä"}]}"#;
        assert_eq!(path(input), "/Grüße");
        let input = r#"{"label":"Grüße ✓ 😀 \u00e4","items":[{"n":1},{"n":"ä"}]}"#;
        assert_eq!(path(input), "/items/1/n");
    }

    #[test]
    fn tells_errors_in_a_key_from_errors_in_a_value() {
        // An unknown key is about the member, a repeated one about the object it repeats in.
        assert_eq!(path(r#"{"items":[{"m":1}]}"#), "/items/0/m");
        assert_eq!(path(r#"{"items":[{"n":1,"n":2}]}"#), "/items/0");
        assert_eq!(path(r#"{"items":[{"n":1,"n" : 2}]}"#), "/items/0");
        assert_eq!(path(r#"{"label":"a","label":"b"}"#), "/");
        // A missing key or a short array is found once the whole object or array is read.
        assert_eq!(path(r#"{"items":[{"n":1},{}]}"#), "/items/1");
        assert_eq!(path(r#"{"items":[[]]}"#), "/items/0");
        assert_eq!(path(r#"{"items":[{"n":1}],"label":{}}"#), "/label");
    }

    #[test]
    fn finds_errors_before_the_end_of_the_input() {
        assert_eq!(path(r#"{"items":[{"n":"x""#), "/items/0/n");
        let (path, message) = located(r#"{"when":true"#);
        assert_eq!(path, "/when");
        assert!(message.ends_with("at line 1 column 12"), "{message}");
    }

    #[test]
    fn reports_values_of_several_shapes_at_their_own_path() {
        for (input, path, column) in [
            (r#"{"path":[1]}"#, "/path", 12),
            (r#"{"path":[1],"label":"a"}"#, "/path", 11),
            (r#"{"path":{"from":"a"}}"#, "/path", 21),
            (r#"{"path":{"from":"a","from":"b","to":"c"}}"#, "/path", 41),
            (r#"{"path":{"from":"a","to":"b","x":"c"}}"#, "/path", 38),
            (r#"{"when":[]}"#, "/when", 11),
            (r#"{"when" : false , "label":"a"}"#, "/when", 16),
            (r#"{"bound":true}"#, "/bound", 14),
        ] {
            let (found, message) = located(input);
            assert_eq!(found, path, "{input}");
            let name = if path == "/path" {
                "MainPath"
            } else if path == "/when" {
                "TimeValue"
            } else {
                "ZoomBound"
            };
            assert_eq!(
                message,
                format!(
                    "data did not match any variant of untagged enum {name} at line 1 column {column}"
                ),
                "{input}"
            );
        }
        let pretty = "{\n  \"path\": [\n    \"a\",\n    2\n  ]\n}";
        assert_eq!(
            located(pretty),
            (
                "/path".to_owned(),
                "data did not match any variant of untagged enum MainPath at line 6 column 1"
                    .to_owned()
            )
        );
    }

    #[test]
    fn reads_each_shape_into_its_variant() {
        for (input, read) in [
            (
                r#"{"path":{"to":"b","from":"a"},"when":"2026-03-01","bound":-3}"#,
                r#"Some(Between { from: "a", to: "b" }) Some(Text("2026-03-01")) Some(Time(Number(-3.0)))"#,
            ),
            (
                r#"{"path":["a","b"],"when":7,"bound":7}"#,
                r#"Some(Steps(["a", "b"])) Some(Number(7.0)) Some(Index(7))"#,
            ),
            (
                r#"{"path":[],"when":-0.5,"bound":"2026"}"#,
                r#"Some(Steps([])) Some(Number(-0.5)) Some(Time(Text("2026")))"#,
            ),
            (r#"{"bound":7.5}"#, "None None Some(Time(Number(7.5)))"),
        ] {
            let document: Document = serde_json::from_str(input).unwrap();
            let found = format!(
                "{:?} {:?} {:?}",
                document.path, document.when, document.bound
            );
            assert_eq!(found, read, "{input}");
        }
    }
}
