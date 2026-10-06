use super::{
    ChartSpec, ChartType, Dataset, LayerRef, MAX_SERIES, NumberStyle, Series, ValueFormat,
    atlas::place_count,
    timechart::{Gaps, LayerSpec, Mark},
};
use crate::{error::ChartError, time::TimeValue};

impl ChartSpec {
    pub(crate) fn dataset(&self) -> Dataset {
        if self.series.is_empty() {
            Dataset {
                categories: self.data.iter().map(|point| point.label.clone()).collect(),
                series: vec![Series {
                    name: None,
                    values: self.data.iter().map(|point| point.value).collect(),
                    style: None,
                }],
            }
        } else {
            Dataset {
                categories: self.categories.clone(),
                series: self
                    .series
                    .iter()
                    .map(|series| Series {
                        name: Some(series.name.clone()),
                        values: series.values.clone(),
                        style: None,
                    })
                    .collect(),
            }
        }
    }

    /// The dataset behind the data table: a time chart reports one row per timestamp, a topicmap
    /// chart one row per topic and island, an atlas one row per region, every other chart one row
    /// per category.
    pub(crate) fn table_dataset(&self) -> Dataset {
        match self.chart_type {
            ChartType::Multiples if self.is_bar_multiples() => self.bar_multiples_dataset(),
            ChartType::Time | ChartType::Multiples => {
                let zone = self.time_zone().unwrap_or_default();
                self.time_dataset(zone, true)
            }
            ChartType::Topicmap => self.topicmap_dataset(),
            ChartType::Atlas => self.atlas_dataset(),
            ChartType::Stripes => self.stripes_dataset(),
            ChartType::Calendar => self.calendar_dataset(),
            ChartType::Rangebar => self.rangebar_dataset(),
            ChartType::Boxplot => self.boxplot_dataset(),
            ChartType::Bar | ChartType::Line => self.dataset(),
            ChartType::Sequence
            | ChartType::Flow
            | ChartType::State
            | ChartType::Architecture
            | ChartType::Tree
            | ChartType::Timeline
            | ChartType::Waterfall
            | ChartType::Waffle
            | ChartType::Parliament => {
                unreachable!("a diagram's table holds text, see its data_table")
            }
        }
    }

    /// One row per year, `Missing` where the year has no value.
    fn stripes_dataset(&self) -> Dataset {
        let stripes = self
            .stripes
            .as_ref()
            .expect("validated stripes charts carry a stripes block");
        Dataset {
            categories: stripes.years().map(|year| year.to_string()).collect(),
            series: vec![Series {
                name: None,
                values: stripes.values.clone(),
                style: None,
            }],
        }
    }

    /// One row per day that has an entry, in calendar order.
    fn calendar_dataset(&self) -> Dataset {
        let calendar = self
            .calendar
            .as_ref()
            .expect("validated calendar charts carry a calendar block");
        let days = calendar.sorted_days();
        Dataset {
            categories: days.iter().map(|day| day.date.clone()).collect(),
            series: vec![Series {
                name: None,
                values: days.iter().map(|day| day.value).collect(),
                style: None,
            }],
        }
    }

    /// One row per range with its low, high and, where any range has one, its central value.
    /// A modeled range says so in its row label, since the table cannot show the hatching.
    /// One row per box and one column per number of its summary.
    fn boxplot_dataset(&self) -> Dataset {
        let words = self.locale.words();
        let summaries = self.box_summaries();
        let column = |name: &str, number: fn(&super::BoxSummary) -> f64| Series {
            name: Some(name.to_owned()),
            values: summaries
                .iter()
                .map(|summary| Some(number(summary)))
                .collect(),
            style: None,
        };
        Dataset {
            categories: self.boxes.iter().map(|boxed| boxed.label.clone()).collect(),
            series: vec![
                column(words.minimum, |summary| summary.min),
                column(words.quartile_1, |summary| summary.q1),
                column(words.median, |summary| summary.median),
                column(words.quartile_3, |summary| summary.q3),
                column(words.maximum, |summary| summary.max),
            ],
        }
    }

    fn rangebar_dataset(&self) -> Dataset {
        let words = self.locale.words();
        let mut series = vec![
            Series {
                name: Some(words.range_low.to_owned()),
                values: self.ranges.iter().map(|range| Some(range.low)).collect(),
                style: None,
            },
            Series {
                name: Some(words.range_high.to_owned()),
                values: self.ranges.iter().map(|range| Some(range.high)).collect(),
                style: None,
            },
        ];
        if self.ranges.iter().any(|range| range.weight.is_some()) {
            series.push(Series {
                name: Some(words.weight.to_owned()),
                values: self.ranges.iter().map(|range| range.weight).collect(),
                style: None,
            });
        }
        if self.ranges.iter().any(|range| range.mid.is_some()) {
            series.insert(
                1,
                Series {
                    name: Some(words.range_mid.to_owned()),
                    values: self.ranges.iter().map(|range| range.mid).collect(),
                    style: None,
                },
            );
        }
        Dataset {
            categories: self
                .ranges
                .iter()
                .map(|range| {
                    let notes: Vec<&str> = range
                        .group
                        .as_deref()
                        .into_iter()
                        .chain(range.modeled.then_some(words.modeled))
                        .chain(range.summary.then_some(words.summary))
                        .collect();
                    if notes.is_empty() {
                        range.label.clone()
                    } else {
                        format!("{} ({})", range.label, notes.join(", "))
                    }
                })
                .collect(),
            series,
        }
    }

    /// Topics and islands as rows, with their value and path count as columns.
    /// The table behind an atlas: one row per region, naming the realm it lies in. The realms
    /// themselves get no row — they hold no number of their own, only the sum of their regions,
    /// and a row repeating that sum would invite reading it as a sixth region.
    fn atlas_dataset(&self) -> Dataset {
        let atlas = self
            .atlas
            .as_ref()
            .expect("validated atlas charts carry an atlas block");
        let regions = || {
            atlas
                .realms
                .iter()
                .flat_map(|realm| realm.regions.iter().map(move |region| (realm, region)))
        };
        Dataset {
            categories: regions()
                .map(|(realm, region)| format!("{} · {}", realm.label, region.label))
                .collect(),
            series: vec![
                Series {
                    name: Some(self.locale.words().entries.to_owned()),
                    values: regions().map(|(_, region)| Some(region.value)).collect(),
                    style: None,
                },
                Series {
                    name: Some(self.locale.words().places.to_owned()),
                    values: regions()
                        .map(|(_, region)| Some(place_count(region.places.len())))
                        .collect(),
                    style: None,
                },
            ],
        }
    }

    fn topicmap_dataset(&self) -> Dataset {
        let topicmap = self
            .topicmap
            .as_ref()
            .expect("validated topicmap charts carry a topicmap block");
        let all = || topicmap.topics.iter().chain(topicmap.islands.iter());
        let total: f64 = all().map(|topic| topic.value).sum();
        Dataset {
            categories: all().map(|topic| topic.label.clone()).collect(),
            series: vec![
                Series {
                    name: Some(self.locale.words().entries.to_owned()),
                    values: all().map(|topic| Some(topic.value)).collect(),
                    style: None,
                },
                Series {
                    name: Some(self.locale.words().paths.to_owned()),
                    values: all().map(|topic| Some(f64::from(topic.points))).collect(),
                    style: None,
                },
                Series {
                    name: Some(self.locale.words().share.to_owned()),
                    // Of every entry on the map, islands included: the table lists them too.
                    // Rounded to two decimal places of a percent — a share carried to twelve
                    // digits would claim a precision the underlying counts do not have.
                    values: all()
                        .map(|topic| Some(((topic.value / total) * 10_000.0).round() / 10_000.0))
                        .collect(),
                    style: Some(NumberStyle {
                        format: ValueFormat::Percent,
                        locale: self.locale,
                        ..NumberStyle::default()
                    }),
                },
            ],
        }
    }

    /// The time axis as configured, or its default when the specification omitted it.
    pub(crate) fn time_zone(&self) -> Result<crate::time::TimeZone, ChartError> {
        if self.time_axis.kind == super::TimeAxisKind::Number {
            return Ok(crate::time::TimeZone::numeric(self.locale));
        }
        crate::time::TimeZone::parse(&self.time_axis.timezone)
            .map(|zone| zone.with_locale(self.locale))
            .ok_or_else(|| {
                ChartError::new(
                    "invalid_timezone",
                    "/timeAxis/timezone",
                    "expected UTC or a fixed offset such as +02:00",
                )
            })
    }

    /// Every layer of every pane, in the order they are drawn.
    pub(crate) fn layers(&self) -> impl Iterator<Item = &LayerSpec> {
        self.panes.iter().flat_map(|pane| pane.layers.iter())
    }

    /// Every layer with its index among all layers (which keys its declared color), the index of
    /// its pane and its index within that pane (which its specification path uses).
    pub(crate) fn indexed_layers(&self) -> impl Iterator<Item = LayerRef<'_>> {
        self.panes
            .iter()
            .enumerate()
            .flat_map(|(pane, spec)| {
                spec.layers
                    .iter()
                    .enumerate()
                    .map(move |(local, layer)| (pane, local, layer))
            })
            .enumerate()
            .map(|(global, (pane, local, layer))| LayerRef {
                global,
                pane,
                local,
                layer,
            })
    }

    /// Every layer that draws data, see [`Self::indexed_layers`].
    pub(crate) fn data_layers(&self) -> impl Iterator<Item = LayerRef<'_>> {
        self.indexed_layers().filter(|entry| entry.layer.is_data())
    }

    /// With `gaps: "collapse"`, the slots of the time axis in order: every distinct timestamp an
    /// observation of a data layer uses in any pane, a missing value and a candle included. Each
    /// slot takes the same share of the axis. `None` while the axis keeps the distances in time.
    pub(crate) fn time_slots(&self, zone: crate::time::TimeZone) -> Option<Vec<i64>> {
        (self.time_axis.gaps == Gaps::Collapse).then(|| {
            let mut slots: Vec<i64> = self
                .data_layers()
                .flat_map(|entry| entry.layer.resolved_times(zone))
                .collect();
            slots.sort_unstable();
            slots.dedup();
            slots
        })
    }

    /// The names of the data layers, once each in order of first appearance. In small multiples
    /// a name is one series across every panel, so it keeps one color and one legend entry.
    pub(crate) fn series_names(&self) -> Vec<Option<String>> {
        let mut names: Vec<Option<String>> = Vec::new();
        for entry in self.data_layers() {
            if !names.contains(&entry.layer.name) {
                names.push(entry.layer.name.clone());
            }
        }
        names
    }

    /// Which palette color a data layer takes when it declares none. In a time chart that is its
    /// position among the chart's data layers other than candles, across all panes, so that no two
    /// entries of the one legend share a color; a layer from the fifth position on takes the first
    /// palette color no layer before it holds, which validation guarantees exists because at most
    /// [`MAX_SERIES`] layers go without a color of their own. In small multiples it is the
    /// position of its name among all names.
    pub(crate) fn palette_index(&self, layer: &LayerSpec) -> usize {
        let index = if self.chart_type == ChartType::Multiples {
            self.series_names()
                .iter()
                .position(|name| *name == layer.name)
                .unwrap_or(0)
        } else {
            let data = || {
                self.layers()
                    .filter(|candidate| candidate.is_data() && candidate.mark != Mark::Ohlc)
                    .enumerate()
            };
            let mut taken = [false; MAX_SERIES];
            for (position, candidate) in data() {
                if position < MAX_SERIES && candidate.color.is_none() {
                    taken[position] = true;
                }
            }
            let mut index = 0;
            for (position, candidate) in data() {
                let slot = if position < MAX_SERIES {
                    position
                } else if candidate.color.is_none() {
                    let free = taken.iter().position(|taken| !taken).unwrap_or(0);
                    taken[free] = true;
                    free
                } else {
                    0
                };
                if std::ptr::eq(candidate, layer) {
                    index = slot;
                    break;
                }
            }
            index
        };
        index.min(MAX_SERIES - 1)
    }

    /// How finely one data layer's times are written: as the layer declares, else as the time
    /// axis declares, else as finely as its own observations need.
    pub(crate) fn layer_precision(
        &self,
        layer: &super::LayerSpec,
        zone: crate::time::TimeZone,
    ) -> crate::time::Precision {
        match layer.precision.or(self.time_axis.precision) {
            Some(precision) if !zone.is_numeric() => precision.into(),
            _ => crate::time::Precision::of(layer.resolved_times(zone).into_iter(), zone),
        }
    }

    /// How finely the chart names its times: as the time axis declares, else as finely as its
    /// observations need.
    pub(crate) fn time_precision(&self, zone: crate::time::TimeZone) -> crate::time::Precision {
        match self.time_axis.precision {
            Some(precision) if !zone.is_numeric() => precision.into(),
            _ => self.observed_precision(zone),
        }
    }

    /// How finely the observations of all data layers need their times named, whatever the
    /// specification declares. The time axis spaces its ticks by it, so that a declared
    /// precision changes how times are named but not how many ticks there are.
    pub(crate) fn observed_precision(&self, zone: crate::time::TimeZone) -> crate::time::Precision {
        let epochs: Vec<i64> = self
            .data_layers()
            .flat_map(|entry| entry.layer.resolved_times(zone))
            .collect();
        crate::time::Precision::of(epochs.into_iter(), zone)
    }

    /// How values are written: the value format, the fixed decimals of the axis that applies, and
    /// the locale.
    pub(crate) fn number_style(&self) -> NumberStyle {
        let axis = match self.chart_type {
            ChartType::Time => self.panes.first().map(|pane| &pane.value_axis),
            ChartType::Bar
            | ChartType::Line
            | ChartType::Rangebar
            | ChartType::Boxplot
            | ChartType::Waterfall
            | ChartType::Multiples => Some(&self.value_axis),
            ChartType::Topicmap
            | ChartType::Atlas
            | ChartType::Stripes
            | ChartType::Calendar
            | ChartType::Sequence
            | ChartType::Flow
            | ChartType::State
            | ChartType::Architecture
            | ChartType::Tree
            | ChartType::Timeline
            | ChartType::Waffle
            | ChartType::Parliament => None,
        };
        NumberStyle {
            format: self.value_format(),
            decimals: axis.and_then(|axis| axis.decimals),
            locale: self.locale,
            thousands: axis.is_some_and(|axis| axis.thousands_separator),
        }
    }

    /// How the value axis writes its ticks: a percent stack measures shares, whatever format its
    /// values have.
    pub(crate) fn axis_style(&self) -> NumberStyle {
        let style = self.number_style();
        if self.stack == Some(super::Stack::Percent) {
            NumberStyle {
                format: ValueFormat::Percent,
                decimals: None,
                ..style
            }
        } else {
            style
        }
    }

    /// How the values of one pane are written: a time chart's pane has its own value axis, every
    /// other chart writes all values alike.
    pub(crate) fn pane_style(&self, pane_index: usize) -> NumberStyle {
        if self.chart_type == ChartType::Time || self.is_bar_multiples() {
            NumberStyle {
                format: self.panes[pane_index].value_axis.format,
                decimals: self.panes[pane_index].value_axis.decimals,
                locale: self.locale,
                thousands: self.panes[pane_index].value_axis.thousands_separator,
            }
        } else {
            self.number_style()
        }
    }

    /// The value format that applies to the chart: the pane's format for a time chart, otherwise
    /// the single top-level value axis.
    pub(crate) fn value_format(&self) -> ValueFormat {
        match self.chart_type {
            ChartType::Time => self
                .panes
                .first()
                .map_or(ValueFormat::Number, |pane| pane.value_axis.format),
            ChartType::Topicmap
            | ChartType::Atlas
            | ChartType::Stripes
            | ChartType::Calendar
            | ChartType::Sequence
            | ChartType::Flow
            | ChartType::State
            | ChartType::Architecture
            | ChartType::Tree
            | ChartType::Timeline
            | ChartType::Waffle
            | ChartType::Parliament => ValueFormat::Number,
            ChartType::Bar
            | ChartType::Line
            | ChartType::Rangebar
            | ChartType::Boxplot
            | ChartType::Waterfall
            | ChartType::Multiples => self.value_axis.format,
        }
    }

    /// Timestamps × layers: one row per timestamp that any data layer uses, one column per data
    /// layer, and `None` where a layer has no observation or a `null` value at that timestamp.
    /// With `bounds`, a layer with a band adds a column for its lower and one for its upper edge.
    ///
    /// Timestamps and layer values are owned because the row labels are formatted timestamps
    /// rather than text taken from the specification.
    /// A column name followed by the unit of its pane's value axis, if the axis has one and the
    /// name heads a `table` column; the description writes the unit after the values instead.
    fn with_unit(&self, name: Option<String>, pane: usize, table: bool) -> Option<String> {
        if !table {
            return name;
        }
        match (name, self.axis_unit(pane)) {
            (Some(name), Some(unit)) => Some(format!("{name} ({unit})")),
            // A single unnamed layer is the table's "Value" column; the unit still belongs to it.
            (None, Some(unit)) => Some(format!("{} ({unit})", self.locale.words().value)),
            (name, None) => name,
        }
    }

    /// The table of a time chart: a row per time, labelled as finely as the finest layer with an
    /// observation at that time.
    pub(crate) fn time_dataset(&self, zone: crate::time::TimeZone, bounds: bool) -> Dataset {
        let mut categories: Vec<i64> = self
            .data_layers()
            .flat_map(|entry| entry.layer.resolved_times(zone))
            .collect();
        categories.sort_unstable();
        categories.dedup();

        let column = |pairs: Vec<(i64, f64)>| -> Vec<Option<f64>> {
            categories
                .iter()
                .map(|epoch| {
                    pairs
                        .iter()
                        .find_map(|(point, value)| (point == epoch).then_some(*value))
                })
                .collect()
        };

        // A part of a layer, such as a band edge or a candle's open, is named after the layer.
        let part_name = |name: &Option<String>, part: &str| {
            if let Some(name) = name {
                format!("{name} ({part})")
            } else {
                let mut chars = part.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        };
        let words = self.locale.words();
        let mut series = Vec::new();
        for (entry, name) in self.data_layers().zip(self.layer_names()) {
            let layer = entry.layer;
            let name = self.with_unit(name, entry.pane, bounds);
            // Every pane of a time chart writes its values by its own value axis.
            let style = (self.chart_type == ChartType::Time).then(|| self.pane_style(entry.pane));
            if layer.mark == Mark::Ohlc {
                let candles = layer.resolved_candles(zone);
                for (index, part) in [words.open, words.high, words.low, words.close]
                    .into_iter()
                    .enumerate()
                {
                    series.push(Series {
                        name: Some(part_name(&name, part)),
                        values: column(
                            candles
                                .iter()
                                .map(|(epoch, values)| (*epoch, values[index]))
                                .collect(),
                        ),
                        style,
                    });
                }
                continue;
            }
            series.push(Series {
                name: name.clone(),
                values: column(layer.resolved_points(zone)),
                style,
            });
            if bounds && layer.has_band() {
                let band = layer.resolved_band(zone);
                for (edge, pick) in [(words.lower, 0), (words.upper, 1)] {
                    series.push(Series {
                        name: Some(part_name(&name, edge)),
                        values: column(
                            band.iter()
                                .map(|(epoch, lower, upper)| {
                                    (*epoch, if pick == 0 { *lower } else { *upper })
                                })
                                .collect(),
                        ),
                        style,
                    });
                }
            }
        }

        let layers: Vec<(Vec<i64>, crate::time::Precision)> = self
            .data_layers()
            .map(|entry| {
                (
                    entry.layer.resolved_times(zone),
                    self.layer_precision(entry.layer, zone),
                )
            })
            .collect();
        Dataset {
            categories: categories
                .iter()
                .map(|epoch| {
                    layers
                        .iter()
                        .filter(|(times, _)| times.contains(epoch))
                        .map(|(_, precision)| *precision)
                        .reduce(crate::time::Precision::at_least)
                        .expect("every row comes from a layer")
                        .format(*epoch, zone)
                })
                .collect(),
            series,
        }
    }

    /// The name of every data layer, defaulting to `Value` for a single unnamed layer. In small
    /// multiples the panel title comes first, so that a column names both.
    pub(crate) fn layer_names(&self) -> impl Iterator<Item = Option<String>> + '_ {
        let named = self.data_layers().count() > 1;
        self.data_layers().map(move |entry| {
            let (pane_index, layer) = (entry.pane, entry.layer);
            let name = layer
                .name
                .clone()
                .or_else(|| named.then(|| self.locale.words().value.to_owned()));
            match (&self.panes[pane_index].title, name) {
                (Some(title), Some(name)) if self.chart_type == ChartType::Multiples => {
                    Some(if layer.name.is_some() {
                        format!("{title} · {name}")
                    } else {
                        title.clone()
                    })
                }
                (_, name) => name,
            }
        })
    }

    /// Returns a copy of this time chart restricted to the observations from `from` to `to`, both
    /// included. Every data layer stays, even when the window leaves it empty, so that colors and
    /// the legend match the full chart; a vertical reference line outside the window is left out.
    pub(crate) fn windowed(&self, from: i64, to: i64) -> ChartSpec {
        let zone = self.time_zone().unwrap_or_default();
        let inside = |time: &TimeValue| {
            time.resolve(zone)
                .is_ok_and(|epoch| (from..=to).contains(&epoch))
        };
        let mut spec = self.clone();
        // A window shows its own span: the declared ends and tick step of the whole axis do not
        // apply to it.
        spec.time_axis.min = None;
        spec.time_axis.max = None;
        spec.time_axis.step = None;
        for pane in &mut spec.panes {
            pane.layers
                .retain(|layer| layer.time.as_ref().is_none_or(inside));
            // A zone that misses the window is left out; one that reaches beyond it is cut at
            // the window's edges, so it cannot widen the time axis.
            pane.layers.retain(|layer| {
                let resolve = |time: &Option<TimeValue>| {
                    time.as_ref().and_then(|time| time.resolve(zone).ok())
                };
                layer.mark != Mark::Band
                    || (resolve(&layer.from).is_none_or(|start| start < to)
                        && resolve(&layer.to).is_none_or(|end| end > from))
            });
            for layer in &mut pane.layers {
                // The last step ends at `stepEnd` only in a window that holds the last
                // observation, and then at the latest at the window's end.
                let last = layer
                    .points
                    .last()
                    .and_then(|point| point.time.resolve(zone).ok());
                #[allow(clippy::cast_precision_loss)]
                if let Some(end) = layer
                    .step_end
                    .as_ref()
                    .and_then(|time| time.resolve(zone).ok())
                {
                    layer.step_end = last
                        .filter(|last| (from..=to).contains(last))
                        .map(|_| TimeValue::Number(end.min(to) as f64));
                }
                layer.points.retain(|point| inside(&point.time));
                layer.data.retain(|candle| inside(&candle.time));
                if layer.mark == Mark::Band {
                    #[allow(clippy::cast_precision_loss)]
                    let clamp = |time: &mut Option<TimeValue>| {
                        if let Some(epoch) = time.as_ref().and_then(|time| time.resolve(zone).ok())
                            && !(from..=to).contains(&epoch)
                        {
                            *time = Some(TimeValue::Number(epoch.clamp(from, to) as f64));
                        }
                    };
                    clamp(&mut layer.from);
                    clamp(&mut layer.to);
                }
            }
        }
        // With gaps collapsed, an annotation needs an observation of the window to stand on: a
        // reference line or marker the first one at or after its time, a zone one between its
        // edges.
        if let Some(slots) = spec.time_slots(zone) {
            let resolve =
                |time: &Option<TimeValue>| time.as_ref().and_then(|time| time.resolve(zone).ok());
            for pane in &mut spec.panes {
                pane.layers.retain(|layer| match layer.mark {
                    Mark::Annotation => {
                        resolve(&layer.time).is_none_or(|time| slots.last() >= Some(&time))
                    }
                    Mark::Band if layer.from.is_some() || layer.to.is_some() => {
                        let start = resolve(&layer.from).unwrap_or(i64::MIN);
                        let end = resolve(&layer.to).unwrap_or(i64::MAX);
                        slots.iter().any(|slot| (start..=end).contains(slot))
                    }
                    _ => true,
                });
            }
        }
        spec
    }

    /// Returns a copy of this specification with categories and values sliced to `from..=to`.
    pub(crate) fn sliced(&self, from: usize, to: usize) -> ChartSpec {
        let mut spec = self.clone();
        if self.data.is_empty() {
            spec.categories = self.categories[from..=to].to_vec();
            spec.series = self
                .series
                .iter()
                .map(|s| {
                    let mut series = s.clone();
                    series.values = s.values[from..=to].to_vec();
                    series
                })
                .collect();
        } else {
            spec.data = self.data[from..=to].to_vec();
        }
        spec
    }
}
