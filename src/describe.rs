use std::fmt::Write as _;

use crate::{
    ChartSpec, ChartType, architecture, calendar, flow, layout, ohlc, rangebar, sequence,
    spec::{self, Dataset, LayerRef},
    state, stripes, text, time, tree,
};

pub(crate) fn automatic_description(spec: &ChartSpec) -> String {
    match spec.chart_type {
        ChartType::Multiples if spec.is_bar_multiples() => return panel_bars_description(spec),
        ChartType::Time | ChartType::Multiples => return time_description(spec),
        ChartType::Topicmap => return topicmap_description(spec),
        ChartType::Atlas => return atlas_description(spec),
        ChartType::Stripes => return stripes::description(spec),
        ChartType::Calendar => return calendar::description(spec),
        ChartType::Rangebar => return rangebar::description(spec),
        ChartType::Boxplot => return crate::boxplot::description(spec),
        ChartType::Sequence => return sequence::description(spec),
        ChartType::Flow => return flow::description(spec),
        ChartType::State => return state::description(spec),
        ChartType::Architecture => return architecture::description(spec),
        ChartType::Tree => return tree::description(spec),
        ChartType::Timeline => return crate::timeline::description(spec),
        ChartType::Waterfall => return crate::waterfall::description(spec),
        ChartType::Bar | ChartType::Line => {}
    }
    let dataset = spec.dataset();
    let locale = spec.locale;
    let words = locale.words();
    let line = spec.chart_type == ChartType::Line;
    let show = |value| layout::format_value(value, spec.number_style());
    let highest_value = dataset
        .values()
        .max_by(f64::total_cmp)
        .expect("validated charts contain data");
    let lowest_value = dataset
        .values()
        .min_by(f64::total_cmp)
        .expect("validated charts contain data");
    let categories = dataset.categories.len();
    let grouped = dataset.series.len() > 1;
    let equal = highest_value.total_cmp(&lowest_value).is_eq();

    if !grouped && equal {
        return text::equal_values(locale, line, categories, &show(highest_value));
    }
    let names = dataset
        .series
        .iter()
        .filter_map(|series| series.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let mut description =
        text::categories_opening(locale, line, categories, dataset.series.len(), &names);
    if equal {
        write!(
            description,
            " {}: {}.",
            words.all_values,
            show(highest_value)
        )
    } else {
        write!(
            description,
            " {}: {} ({}). {}: {} ({}).",
            words.highest,
            show(highest_value),
            labels_at_value(&dataset, highest_value, locale),
            words.lowest,
            show(lowest_value),
            labels_at_value(&dataset, lowest_value, locale)
        )
    }
    .expect("writing to String cannot fail");
    if grouped {
        let missing = dataset
            .series
            .iter()
            .flat_map(|series| &series.values)
            .filter(|value| value.is_none())
            .count();
        description.push_str(&text::missing_values(locale, missing));
    }
    description.push_str(&describe_stack(spec, &dataset, &show));
    description.push_str(&describe_groups(spec));
    if spec.has_error_bars() {
        description.push_str(&text::error_bars(locale));
    }
    if !spec.references.is_empty() {
        let references = spec
            .references
            .iter()
            .map(|reference| format!("{} {} {}", reference.label, words.at, show(reference.value)))
            .collect::<Vec<_>>();
        write!(
            description,
            " {}: {}.",
            words.reference_lines,
            references.join("; ")
        )
        .expect("writing to String cannot fail");
    }
    description
}

/// The groups of a bar chart, each with the labels of its bars; empty without groups.
fn describe_groups(spec: &ChartSpec) -> String {
    let groups: Vec<String> = spec
        .data_groups()
        .into_iter()
        .map(|group| {
            let labels: Vec<&str> = spec
                .data
                .iter()
                .filter(|point| point.group.as_deref() == Some(group))
                .map(|point| point.label.as_str())
                .collect();
            format!("{group}: {}", labels.join(", "))
        })
        .collect();
    if groups.is_empty() {
        String::new()
    } else {
        text::range_groups(spec.locale, &groups.join("; "))
    }
}

/// The sentence about a stack: its highest and lowest total, or that it shows shares.
fn describe_stack(spec: &ChartSpec, dataset: &Dataset, show: &impl Fn(f64) -> String) -> String {
    let locale = spec.locale;
    match spec.stack {
        Some(spec::Stack::Percent) => text::stacked_shares(locale).to_owned(),
        Some(spec::Stack::Normal) => {
            let totals: Vec<(f64, &str)> = dataset
                .categories
                .iter()
                .enumerate()
                .map(|(index, category)| {
                    let total = dataset
                        .series
                        .iter()
                        .filter_map(|series| series.values[index])
                        .sum::<f64>();
                    (total, category.as_str())
                })
                .collect();
            let highest = totals
                .iter()
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .expect("validated charts contain categories");
            let lowest = totals
                .iter()
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .expect("validated charts contain categories");
            text::stacked_totals(
                locale,
                (&show(highest.0), highest.1),
                (&show(lowest.0), lowest.1),
            )
        }
        None => String::new(),
    }
}

fn labels_at_value(dataset: &Dataset, value: f64, locale: spec::Locale) -> String {
    let grouped = dataset.series.len() > 1;
    dataset
        .categories
        .iter()
        .enumerate()
        .flat_map(|(index, category)| {
            dataset
                .series
                .iter()
                .filter(move |series| {
                    series.values[index]
                        .is_some_and(|series_value| series_value.total_cmp(&value).is_eq())
                })
                .map(move |series| match series.name.as_deref() {
                    Some(name) if grouped => text::series_at(locale, name, category),
                    _ => category.clone(),
                })
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Describes a time series: its range, its layers, and the extremes; then what the bands and the
/// reference lines add. Small multiples name their panels first, since the panel is what a reader
/// compares.
/// Small multiples of bars: the panels, each with its highest and its lowest category.
fn panel_bars_description(spec: &ChartSpec) -> String {
    let locale = spec.locale;
    let words = locale.words();
    let dataset = spec.bar_multiples_dataset();
    let mut description =
        text::panel_bars_opening(locale, spec.panes.len(), dataset.categories.len());
    for (index, pane) in spec.panes.iter().enumerate() {
        let style = spec.pane_style(index);
        let show = |value| {
            with_unit(
                layout::format_value(value, style),
                pane.value_axis.unit.as_deref(),
            )
        };
        let known = |highest: bool| {
            let mut entries: Vec<(usize, f64)> = pane
                .values
                .iter()
                .enumerate()
                .filter_map(|(at, value)| value.map(|value| (at, value)))
                .collect();
            crate::sort::by(&mut entries, |a, b| {
                if highest {
                    b.1.total_cmp(&a.1)
                } else {
                    a.1.total_cmp(&b.1)
                }
            });
            entries[0]
        };
        let (top, bottom) = (known(true), known(false));
        write!(
            description,
            " {}: {} {} ({}), {} {} ({}).",
            pane.title.as_deref().unwrap_or_default(),
            words.highest,
            show(top.1),
            dataset.categories[top.0],
            words.lowest,
            show(bottom.1),
            dataset.categories[bottom.0],
        )
        .expect("writing to String cannot fail");
    }
    let notes: Vec<String> = spec
        .panes
        .iter()
        .filter_map(|pane| {
            Some(format!(
                "{}: {}",
                pane.title.as_deref()?,
                pane.note.as_deref()?
            ))
        })
        .collect();
    if !notes.is_empty() {
        description.push_str(&text::panel_notes(locale, &notes.join("; ")));
    }
    description
}

fn time_description(spec: &ChartSpec) -> String {
    if spec.chart_type == ChartType::Time
        && (spec.panes.len() > 1 || spec.layers().any(|layer| layer.mark == spec::Mark::Ohlc))
    {
        return stacked_description(spec);
    }
    let locale = spec.locale;
    let words = locale.words();
    let zone = spec.time_zone().unwrap_or_default();
    let dataset = spec.time_dataset(zone, false);
    let show = |value| value_with_unit(spec, value);
    let points = dataset.categories.len();
    let range = format!(
        "{} {} {} {}",
        words.from,
        dataset
            .categories
            .first()
            .expect("validated time charts have points"),
        words.to,
        dataset
            .categories
            .last()
            .expect("validated time charts have points")
    );
    let names = spec
        .series_names()
        .into_iter()
        .flatten()
        .map(|name| {
            let modeled = spec
                .data_layers()
                .any(|entry| entry.layer.name.as_ref() == Some(&name) && entry.layer.modeled);
            if modeled {
                format!("{name} ({})", words.modeled)
            } else {
                name
            }
        })
        .collect::<Vec<_>>();
    let mut description = if spec.chart_type == ChartType::Multiples {
        let panels = spec
            .panes
            .iter()
            .filter_map(|pane| pane.title.clone())
            .collect::<Vec<_>>();
        text::multiples_opening(locale, &panels, &range, &names, !spec.independent_axes)
    } else {
        // A single series names itself only through the title; several are listed.
        let listed = if dataset.series.len() > 1 {
            names.as_slice()
        } else {
            &[]
        };
        text::axis_noun(
            locale,
            zone,
            text::time_opening(
                locale,
                points,
                &range,
                listed,
                names.len() == 1 && spec.data_layers().any(|entry| entry.layer.modeled),
            ),
        )
    };

    describe_panel_notes(spec, &mut description);

    // Panels on axes of their own measure different things; one highest value across them would
    // compare what cannot be compared.
    if spec.independent_axes {
        describe_additions(spec, zone, &mut description);
        return description;
    }
    let highest_value = dataset
        .values()
        .max_by(f64::total_cmp)
        .expect("validated time charts contain values");
    let lowest_value = dataset
        .values()
        .min_by(f64::total_cmp)
        .expect("validated time charts contain values");
    if highest_value.total_cmp(&lowest_value).is_eq() {
        write!(
            description,
            " {}: {}.",
            words.all_values,
            show(highest_value)
        )
    } else {
        write!(
            description,
            " {}: {} ({}). {}: {} ({}).",
            words.highest,
            show(highest_value),
            labels_at_value(&dataset, highest_value, locale),
            words.lowest,
            show(lowest_value),
            labels_at_value(&dataset, lowest_value, locale)
        )
    }
    .expect("writing to String cannot fail");

    describe_additions(spec, zone, &mut description);
    description
}

/// What a time chart adds to its opening and its values: collapsed gaps, missing values, areas,
/// bands, reference lines, zones and point markers.
fn describe_additions(spec: &ChartSpec, zone: time::TimeZone, description: &mut String) {
    if spec.time_axis.gaps == spec::Gaps::Collapse {
        description.push_str(text::gaps_collapsed(spec.locale));
    }
    let missing = spec
        .data_layers()
        .map(|entry| entry.layer.missing_values())
        .sum();
    description.push_str(&text::missing_values(spec.locale, missing));
    describe_areas(spec, description);
    describe_bands(spec, description);
    describe_rules(spec, zone, description);
    describe_zones_and_markers(spec, zone, description);
}

/// Describes a time chart with stacked panes or candles: its range and layers, the panes, and
/// then pane by pane what its candles did and where its lines peak, each in the pane's own
/// number format.
fn stacked_description(spec: &ChartSpec) -> String {
    let locale = spec.locale;
    let words = locale.words();
    let zone = spec.time_zone().unwrap_or_default();
    let dataset = spec.time_dataset(zone, false);
    let range = format!(
        "{} {} {} {}",
        words.from,
        dataset
            .categories
            .first()
            .expect("validated time charts have points"),
        words.to,
        dataset
            .categories
            .last()
            .expect("validated time charts have points")
    );
    let names = spec
        .data_layers()
        .filter_map(|entry| {
            let name = entry.layer.name.clone()?;
            Some(if entry.layer.modeled {
                format!("{name} ({})", words.modeled)
            } else {
                name
            })
        })
        .collect::<Vec<_>>();
    let layered = spec.data_layers().count() > 1;
    let mut description = text::axis_noun(
        locale,
        zone,
        text::time_opening(
            locale,
            dataset.categories.len(),
            &range,
            if layered { names.as_slice() } else { &[] },
            !layered && spec.data_layers().any(|entry| entry.layer.modeled),
        ),
    );
    let labels: Vec<String> = spec
        .panes
        .iter()
        .enumerate()
        .map(|(index, pane)| {
            pane.value_axis
                .title
                .clone()
                .unwrap_or_else(|| format!("{} {}", words.pane, index + 1))
        })
        .collect();
    if spec.panes.len() > 1 {
        description.push_str(&text::panes(locale, &labels));
    }
    for (pane_index, label) in labels.iter().enumerate() {
        let style = spec.pane_style(pane_index);
        let layers: Vec<LayerRef> = spec
            .data_layers()
            .filter(|entry| entry.pane == pane_index)
            .collect();
        for entry in layers
            .iter()
            .filter(|entry| entry.layer.mark == spec::Mark::Ohlc)
        {
            let precision = spec.layer_precision(entry.layer, zone);
            description.push_str(&ohlc::describe(spec, *entry, zone, precision, style));
        }
        let observations: Vec<(f64, String)> = layers
            .iter()
            .filter(|entry| entry.layer.mark != spec::Mark::Ohlc)
            .flat_map(|entry| {
                let precision = spec.layer_precision(entry.layer, zone);
                entry
                    .layer
                    .resolved_points(zone)
                    .into_iter()
                    .map(move |(epoch, value)| {
                        let time = precision.format(epoch, zone);
                        let place = match &entry.layer.name {
                            Some(name) if layered => text::series_at(locale, name, &time),
                            _ => time,
                        };
                        (value, place)
                    })
            })
            .collect();
        describe_extremes(
            &observations,
            (spec.panes.len() > 1).then_some(label.as_str()),
            (style, spec.axis_unit(pane_index)),
            locale,
            &mut description,
        );
    }
    describe_additions(spec, zone, &mut description);
    description
}

/// The highest and the lowest of a pane's observations, each with every place it occurs;
/// `label` names the pane when the chart has several.
fn describe_extremes(
    observations: &[(f64, String)],
    label: Option<&str>,
    (style, unit): (spec::NumberStyle, Option<&str>),
    locale: spec::Locale,
    description: &mut String,
) {
    let words = locale.words();
    let (Some(highest), Some(lowest)) = (
        observations
            .iter()
            .map(|(value, _)| *value)
            .max_by(f64::total_cmp),
        observations
            .iter()
            .map(|(value, _)| *value)
            .min_by(f64::total_cmp),
    ) else {
        return;
    };
    let at = |target: f64| {
        observations
            .iter()
            .filter(|(value, _)| value.total_cmp(&target).is_eq())
            .map(|(_, place)| place.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let show = |value| with_unit(layout::format_value(value, style), unit);
    let prefix = label.map_or_else(String::new, |label| format!(" {label} –"));
    if highest.total_cmp(&lowest).is_eq() {
        write!(
            description,
            "{prefix} {}: {}.",
            words.all_values,
            show(highest)
        )
    } else {
        write!(
            description,
            "{prefix} {}: {} ({}). {}: {} ({}).",
            words.highest,
            show(highest),
            at(highest),
            words.lowest,
            show(lowest),
            at(lowest)
        )
    }
    .expect("writing to String cannot fail");
}

/// The notes under the panel titles of small multiples, if any panel has one.
fn describe_panel_notes(spec: &ChartSpec, description: &mut String) {
    let notes: Vec<String> = spec
        .panes
        .iter()
        .filter_map(|pane| {
            Some(format!(
                "{}: {}",
                pane.title.as_deref()?,
                pane.note.as_deref()?
            ))
        })
        .collect();
    if !notes.is_empty() {
        description.push_str(&text::panel_notes(spec.locale, &notes.join("; ")));
    }
}

/// The sentences about filled areas, if any layer is one: stacked areas from the bottom up, then
/// the areas filled down to zero.
fn describe_areas(spec: &ChartSpec, description: &mut String) {
    let zone = spec.time_zone().unwrap_or_default();
    let stacked: Vec<&str> = spec
        .data_layers()
        .filter(|entry| spec.stack_base(*entry, zone).is_some())
        .filter_map(|entry| entry.layer.name.as_deref())
        .collect();
    if !stacked.is_empty() {
        description.push_str(&text::stacked_areas(spec.locale, &stacked.join(", ")));
    }
    let areas: Vec<LayerRef> = spec
        .data_layers()
        .filter(|entry| {
            entry.layer.mark == spec::Mark::Area && spec.stack_base(*entry, zone).is_none()
        })
        .collect();
    if areas.is_empty() {
        return;
    }
    let mut names: Vec<String> = Vec::new();
    for entry in areas {
        if let Some(name) = &entry.layer.name
            && !names.contains(name)
        {
            names.push(name.clone());
        }
    }
    description.push_str(&text::areas(spec.locale, &names));
}

/// The sentence about uncertainty bands, if any line has one.
fn describe_bands(spec: &ChartSpec, description: &mut String) {
    let banded = spec
        .data_layers()
        .filter(|entry| entry.layer.has_band())
        .count();
    if banded == 0 {
        return;
    }
    let hatched = spec
        .data_layers()
        .any(|entry| entry.layer.has_band() && entry.layer.modeled);
    description.push_str(&text::bands(spec.locale, banded, hatched));
}

/// The sentence that names every reference line and where it lies.
fn describe_rules(spec: &ChartSpec, zone: time::TimeZone, description: &mut String) {
    let words = spec.locale.words();
    let rules = spec
        .indexed_layers()
        .filter(|entry| entry.layer.is_rule())
        .filter_map(|entry| {
            let layer = entry.layer;
            let label = layer.label.as_deref()?;
            if let Some(value) = layer.value {
                return Some(format!(
                    "{label} {} {}",
                    words.at,
                    layout::format_value(value, spec.pane_style(entry.pane))
                ));
            }
            let epoch = layer.time.as_ref()?.resolve(zone).ok()?;
            Some(format!(
                "{label} {} {}",
                words.at,
                time::Precision::of(std::iter::once(epoch), zone)
                    .at_least(spec.time_precision(zone))
                    .format(epoch, zone)
            ))
        })
        .collect::<Vec<_>>();
    if !rules.is_empty() {
        write!(
            description,
            " {}: {}.",
            words.reference_lines,
            rules.join("; ")
        )
        .expect("writing to String cannot fail");
    }
}

/// The sentences that name every zone with its extent and every point marker with its time and
/// value.
fn describe_zones_and_markers(spec: &ChartSpec, zone: time::TimeZone, description: &mut String) {
    let words = spec.locale.words();
    let zones: Vec<String> = spec
        .indexed_layers()
        .filter(|entry| entry.layer.mark == spec::Mark::Band)
        .filter_map(|entry| {
            let label = entry.layer.label.as_deref()?;
            Some(format!(
                "{label}, {}",
                layout::zone_extent(spec, entry.layer, zone, spec.pane_style(entry.pane))
            ))
        })
        .collect();
    if !zones.is_empty() {
        write!(description, " {}: {}.", words.zones, zones.join("; "))
            .expect("writing to String cannot fail");
    }
    let markers: Vec<String> = spec
        .indexed_layers()
        .filter(|entry| entry.layer.is_marker())
        .filter_map(|entry| {
            let label = entry.layer.label.as_deref()?;
            Some(format!(
                "{label} {} {}",
                words.at,
                layout::marker_position(
                    entry.layer,
                    zone,
                    spec.time_precision(zone),
                    spec.pane_style(entry.pane),
                )
            ))
        })
        .collect();
    if !markers.is_empty() {
        write!(description, " {}: {}.", words.markers, markers.join("; "))
            .expect("writing to String cannot fail");
    }
}

/// Describes a topic map: its area count, the largest and smallest topic, and the strongest
/// neighborhood, if any is declared.
/// What the landscape says in one sentence: how it is divided, and where the weight lies.
fn atlas_description(spec: &ChartSpec) -> String {
    let atlas = spec
        .atlas
        .as_ref()
        .expect("validated atlas charts carry an atlas block");
    let show = |value| layout::format_value(value, spec.number_style());
    let realms = atlas.realms.len();
    let regions: usize = atlas.realms.iter().map(|realm| realm.regions.len()).sum();
    let places: usize = atlas
        .realms
        .iter()
        .flat_map(|realm| &realm.regions)
        .map(|region| region.places.len())
        .sum();
    let largest = atlas
        .realms
        .iter()
        .flat_map(|realm| &realm.regions)
        .max_by(|a, b| a.value.total_cmp(&b.value))
        .expect("validated atlas charts have at least one region");
    text::atlas_sentence(
        spec.locale,
        realms,
        regions,
        (&largest.label, &show(largest.value)),
        places,
    )
}

fn topicmap_description(spec: &ChartSpec) -> String {
    let topicmap = spec
        .topicmap
        .as_ref()
        .expect("validated topicmap charts carry a topicmap block");
    let show = |value| layout::format_value(value, spec.number_style());
    let areas = topicmap.topics.len();
    let largest = topicmap
        .topics
        .iter()
        .max_by(|a, b| a.value.total_cmp(&b.value))
        .expect("validated topicmap charts have at least one topic");
    let smallest = topicmap
        .topics
        .iter()
        .min_by(|a, b| a.value.total_cmp(&b.value))
        .expect("validated topicmap charts have at least one topic");

    let mut description = text::topicmap_opening(
        spec.locale,
        areas,
        topicmap.islands.len(),
        (&largest.label, &show(largest.value)),
        (&smallest.label, &show(smallest.value)),
    );

    // The strongest neighbourhoods, named as relationships in the data rather than as something
    // the drawing does: a weak route may have been dropped before it was ever drawn.
    let mut strongest: Vec<&spec::TopicLinkSpec> = topicmap.links.iter().collect();
    crate::sort::by(&mut strongest, |a, b| b.weight.total_cmp(&a.weight));
    let named: Vec<(&str, &str)> = strongest
        .iter()
        .take(3)
        .map(|link| (link.from.as_str(), link.to.as_str()))
        .collect();
    if !named.is_empty() {
        description.push_str(&text::neighbours(spec.locale, &named));
    }
    description
}

/// A written value followed by the unit of its value axis, if the axis has one.
fn with_unit(value: String, unit: Option<&str>) -> String {
    match unit {
        Some(unit) => format!("{value} {unit}"),
        None => value,
    }
}

/// A value as a single-pane time chart or small multiples write it, with the unit of their value
/// axis.
fn value_with_unit(spec: &ChartSpec, value: f64) -> String {
    with_unit(
        layout::format_value(value, spec.number_style()),
        spec.axis_unit(0),
    )
}
