mod atlas;
mod calendar;
mod color;
mod contour;
mod diverging;
mod error;
mod layout;
mod metrics;
mod noise;
pub mod qr;
mod rangebar;
mod render;
mod scene;
mod spec;
mod stripes;
mod text;
mod time;

use std::fmt::Write as _;

pub use error::{ChartError, ChartWarning};
pub use metrics::{BuiltinMetrics, TextMetrics};
use spec::Dataset;
pub use spec::{
    CalendarDay, CalendarLayout, CalendarSpec, CartoucheSpec, CategoryAxisSpec, ChartSpec,
    ChartType, Corner, DataPoint, Gaps, LayerSpec, Mark, OhlcPoint, Orientation, PaneSpec,
    RangeSpec, SeriesSpec, Shape, StripesSpec, Stroke, Theme, TimeAxisSpec, TimePoint,
    TopicLinkSpec, TopicMapSpec, TopicSpec, ValueAxisSpec, ValueFormat, ZoomStep,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderFormat {
    Svg,
    Html,
}

#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    pub id_prefix: Option<String>,
    pub table_mode: TableMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TableMode {
    #[default]
    Details,
    Visible,
}

#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub content: String,
    pub warnings: Vec<ChartWarning>,
}

/// Parses and renders a chart specification.
///
/// # Errors
///
/// Returns a structured error when parsing, validation, or rendering fails.
pub fn render_json(
    input: &str,
    format: RenderFormat,
    options: &RenderOptions,
) -> Result<RenderOutput, ChartError> {
    let spec = ChartSpec::from_json(input)?;
    render(&spec, format, options)
}

/// Validates and renders an already parsed chart specification.
///
/// # Errors
///
/// Returns a structured error when the specification or render context is invalid.
pub fn render(
    spec: &ChartSpec,
    format: RenderFormat,
    options: &RenderOptions,
) -> Result<RenderOutput, ChartError> {
    render_with_metrics(spec, format, options, &BuiltinMetrics)
}

/// Renders with caller-provided deterministic text metrics.
///
/// # Errors
///
/// Returns a structured error when the specification or render context is invalid.
pub fn render_with_metrics(
    spec: &ChartSpec,
    format: RenderFormat,
    options: &RenderOptions,
    metrics: &impl TextMetrics,
) -> Result<RenderOutput, ChartError> {
    let mut warnings = spec.validate()?;
    let id_prefix = match &options.id_prefix {
        Some(id_prefix) => {
            validate_id_prefix(id_prefix)?;
            id_prefix.clone()
        }
        None => default_id_prefix(spec)?,
    };

    // Zoom steps render as pre-computed variants switched by radio buttons, which only the
    // HTML profile can carry. The pure SVG profile stays a single static chart.
    if format == RenderFormat::Html && spec.zoom_steps.len() > 1 {
        let mut panels = Vec::new();
        for (index, step) in spec.zoom_steps.iter().enumerate() {
            let sliced = spec.sliced(step.from, step.to);
            let panel_prefix = format!("{id_prefix}-z{index}");
            let svg = render_panel(&sliced, &panel_prefix, metrics, &mut warnings);
            panels.push((step.label.clone(), svg));
        }
        dedupe_warnings(&mut warnings);
        return Ok(RenderOutput {
            content: render::html_zoom(&panels, spec, options.table_mode, &id_prefix),
            warnings,
        });
    }

    let svg = render_panel(spec, &id_prefix, metrics, &mut warnings);
    let content = match format {
        RenderFormat::Svg => svg,
        RenderFormat::Html => render::html(&svg, spec, options.table_mode, &id_prefix),
    };
    Ok(RenderOutput { content, warnings })
}

/// Lays out and serializes one chart, using its explicit description or a generated one.
fn render_panel(
    spec: &ChartSpec,
    id_prefix: &str,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> String {
    let description = spec
        .description
        .clone()
        .unwrap_or_else(|| automatic_description(spec));
    let scene = layout::layout(spec, warnings, metrics);
    render::svg(&scene, spec, &description, id_prefix)
}

/// Zoom panels repeat the same data, so identical warnings would otherwise appear once per panel.
fn dedupe_warnings(warnings: &mut Vec<ChartWarning>) {
    let mut seen = std::collections::BTreeSet::new();
    warnings.retain(|warning| seen.insert((warning.code, warning.path.clone())));
}

fn validate_id_prefix(value: &str) -> Result<(), ChartError> {
    let mut characters = value.chars();
    let valid_start = characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic());
    let valid_rest = characters
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if !valid_start || !valid_rest || value.len() > 64 {
        return Err(ChartError::new(
            "invalid_id_prefix",
            "/render/idPrefix",
            "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
        ));
    }
    Ok(())
}

fn default_id_prefix(spec: &ChartSpec) -> Result<String, ChartError> {
    let canonical = serde_json::to_vec(spec).map_err(|error| {
        ChartError::new(
            "serialization_failed",
            "/",
            format!("could not canonicalize the chart specification: {error}"),
        )
    })?;
    let hash = canonical
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    Ok(format!("chartlet-{hash:016x}"))
}

fn automatic_description(spec: &ChartSpec) -> String {
    match spec.chart_type {
        ChartType::Time | ChartType::Multiples => return time_description(spec),
        ChartType::Topicmap => return topicmap_description(spec),
        ChartType::Atlas => return atlas_description(spec),
        ChartType::Stripes => return stripes::description(spec),
        ChartType::Calendar => return calendar::description(spec),
        ChartType::Rangebar => return rangebar::description(spec),
        ChartType::Bar | ChartType::Line => {}
    }
    let dataset = spec.dataset();
    let chart = chart_type_name(spec.chart_type);
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
        return format!(
            "{chart} with {categories} equal values: {} each.",
            show(highest_value)
        );
    }
    let mut description = if grouped {
        let names = dataset
            .series
            .iter()
            .filter_map(|series| series.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{chart} with {categories} categories and {} series ({names}).",
            dataset.series.len()
        )
    } else {
        format!("{chart} with {categories} categories.")
    };
    if equal {
        write!(description, " All values: {}.", show(highest_value))
    } else {
        write!(
            description,
            " Highest: {} ({}). Lowest: {} ({}).",
            show(highest_value),
            labels_at_value(&dataset, highest_value, spec.locale),
            show(lowest_value),
            labels_at_value(&dataset, lowest_value, spec.locale)
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
        match missing {
            0 => {}
            1 => description.push_str(" 1 value is missing."),
            missing => write!(description, " {missing} values are missing.")
                .expect("writing to String cannot fail"),
        }
    }
    description
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
fn time_description(spec: &ChartSpec) -> String {
    let locale = spec.locale;
    let words = locale.words();
    let zone = spec.time_zone().unwrap_or_default();
    let precision = spec.time_precision(zone);
    let dataset = spec.time_dataset(zone, precision, false);
    let show = |value| layout::format_value(value, spec.number_style());
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
        text::multiples_opening(locale, &panels, &range, &names)
    } else {
        // A single series names itself only through the title; several are listed.
        let listed = if dataset.series.len() > 1 {
            names.as_slice()
        } else {
            &[]
        };
        text::time_opening(
            locale,
            points,
            &range,
            listed,
            names.len() == 1 && spec.data_layers().any(|entry| entry.layer.modeled),
        )
    };

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

    describe_bands(spec, &mut description);
    describe_rules(spec, zone, &mut description);
    description
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
    let show = |value| layout::format_value(value, spec.number_style());
    let rules = spec
        .layers()
        .filter(|layer| layer.mark == spec::Mark::Annotation)
        .filter_map(|layer| {
            let label = layer.label.as_deref()?;
            if let Some(value) = layer.value {
                return Some(format!("{label} {} {}", words.at, show(value)));
            }
            let epoch = layer.time.as_ref()?.resolve(zone).ok()?;
            Some(format!(
                "{label} {} {}",
                words.at,
                time::Precision::of(std::iter::once(epoch), zone).format(epoch, zone)
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
    let marked = if places > 0 {
        format!(" {places} places are marked.")
    } else {
        String::new()
    };
    format!(
        "Knowledge landscape of {realms} realms and {regions} regions; largest is {} with {}.{marked}",
        largest.label,
        show(largest.value)
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

    let mut description = format!(
        "Topic map with {areas} area{}",
        if areas == 1 { "" } else { "s" }
    );
    match topicmap.islands.len() {
        0 => description.push('.'),
        1 => description.push_str(" and one island."),
        islands => {
            write!(description, " and {islands} islands.").expect("writing to String cannot fail");
        }
    }
    write!(
        description,
        " Largest: {} with {} entries, smallest: {} with {}.",
        largest.label,
        show(largest.value),
        smallest.label,
        show(smallest.value)
    )
    .expect("writing to String cannot fail");

    // The strongest neighbourhoods, named as relationships in the data rather than as something
    // the drawing does: a weak route may have been dropped before it was ever drawn.
    let mut strongest: Vec<&spec::TopicLinkSpec> = topicmap.links.iter().collect();
    strongest.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    let named: Vec<String> = strongest
        .iter()
        .take(3)
        .map(|link| format!("{} and {}", link.from, link.to))
        .collect();
    if !named.is_empty() {
        write!(description, " Closest neighbours: {}.", named.join(", "))
            .expect("writing to String cannot fail");
    }
    description
}

const fn chart_type_name(chart_type: ChartType) -> &'static str {
    match chart_type {
        ChartType::Bar => "Bar chart",
        ChartType::Line => "Line chart",
        ChartType::Time => "Time chart",
        ChartType::Topicmap => "Topic map",
        ChartType::Atlas => "Knowledge landscape",
        ChartType::Stripes => "Warming stripes",
        ChartType::Calendar => "Calendar",
        ChartType::Rangebar => "Range chart",
        ChartType::Multiples => "Small multiples",
    }
}

#[cfg(test)]
mod tests {
    use super::{BuiltinMetrics, RenderFormat, RenderOptions, TextMetrics, render_json};

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "bar",
        "title": "Profit & loss",
        "data": [
            {"label": "North <East>", "value": 12},
            {"label": "South", "value": -4}
        ]
    }"#;

    #[test]
    fn renders_accessible_deterministic_svg() {
        let first = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        let second = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert_eq!(first.content, second.content);
        assert!(first.content.contains("role=\"img\""));
        assert!(first.content.contains("<title id="));
        assert!(first.content.contains("<desc id="));
        assert!(first.content.contains("North &lt;East&gt;"));
        assert!(first.content.contains("height=\""));
    }

    #[test]
    fn html_contains_complete_data_table() {
        let output = render_json(SPEC, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(output.content.contains("<figure"));
        assert!(output.content.contains("<details"));
        assert!(output.content.contains("<table>"));
        assert!(output.content.contains("<td>−4</td>"));
    }

    #[test]
    fn deserialization_errors_use_json_pointers() {
        for (input, code, path) in [
            (
                SPEC.replace("\"title\"", "\"colour\": 1, \"title\""),
                "invalid_spec",
                "/colour",
            ),
            (
                SPEC.replace("12}", "\"12\"}"),
                "invalid_spec",
                "/data/0/value",
            ),
            (
                SPEC.replace("\"title\": \"Profit & loss\",", ""),
                "invalid_spec",
                "/",
            ),
            ("{\"schemaVersion\": 1,".to_owned(), "invalid_json", "/"),
        ] {
            let error =
                render_json(&input, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
            assert_eq!((error.code, error.path.as_str()), (code, path), "{input}");
        }
    }

    #[test]
    fn rejects_duplicate_labels_with_path() {
        let duplicate = SPEC.replace("South", "North <East>");
        let error =
            render_json(&duplicate, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "duplicate_label");
        assert_eq!(error.path, "/data/1/label");
    }

    #[test]
    fn explicit_prefix_prevents_duplicate_ids() {
        let first = render_json(
            SPEC,
            RenderFormat::Svg,
            &RenderOptions {
                id_prefix: Some("first".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap();
        let second = render_json(
            SPEC,
            RenderFormat::Svg,
            &RenderOptions {
                id_prefix: Some("second".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(first.content.contains("first-title"));
        assert!(second.content.contains("second-title"));
    }

    #[test]
    fn preserves_small_values_in_every_text_alternative() {
        let small = SPEC.replace("12}", "0.001}");
        let output = render_json(&small, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(output.content.contains("<td>0.001</td>"));
        assert!(output.content.contains("0.001"));
    }

    #[test]
    fn rejects_values_outside_the_supported_scale_range() {
        let extreme = SPEC.replace("12}", "1e308}");
        let error =
            render_json(&extreme, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "unsupported_numeric_range");
        assert_eq!(error.path, "/data/0/value");
    }

    #[test]
    fn rejects_xml_control_characters() {
        let invalid = SPEC.replace("Profit & loss", "Bad\\u0000title");
        let error =
            render_json(&invalid, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "invalid_xml_character");
        assert_eq!(error.path, "/title");
    }

    #[test]
    fn equal_values_do_not_invent_extrema() {
        let tied = SPEC.replace("-4", "12");
        let output = render_json(&tied, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(
            output
                .content
                .contains("Bar chart with 2 equal values: 12 each.")
        );
    }

    #[test]
    fn visible_table_mode_omits_details_disclosure() {
        let output = render_json(
            SPEC,
            RenderFormat::Html,
            &RenderOptions {
                table_mode: super::TableMode::Visible,
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(output.content.contains("<div class=\"chartlet-data\">"));
        assert!(!output.content.contains("<details"));
    }

    #[test]
    fn line_chart_preserves_missing_values_as_gaps() {
        let specification = include_str!("../examples/monthly-trend.json");
        let output =
            render_json(specification, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert_eq!(output.content.matches("<polyline").count(), 2);
        assert_eq!(output.content.matches("<circle").count(), 5);
        assert!(output.content.contains("<td>Missing</td>"));
    }

    const GROUPED: &str = r#"{
        "schemaVersion": 1,
        "type": "bar",
        "title": "Budget and actual",
        "categories": ["Jan", "Feb", "Mar"],
        "series": [
            {"name": "Budget", "values": [120, 150, 140]},
            {"name": "Actual", "values": [130, 145, null]}
        ]
    }"#;

    #[test]
    fn grouped_bars_render_legend_bars_and_series_table() {
        let output = render_json(GROUPED, RenderFormat::Html, &RenderOptions::default()).unwrap();
        // Every bar plus one legend swatch per series.
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-bar chartlet-series-1\"")
                .count(),
            4
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-bar chartlet-series-2\"")
                .count(),
            3
        );
        assert!(
            output
                .content
                .contains("<th scope=\"col\">Budget</th><th scope=\"col\">Actual</th>")
        );
        assert!(
            output
                .content
                .contains("<th scope=\"row\">Mar</th><td>140</td><td>Missing</td>")
        );
        assert!(output.content.contains(
            "Bar chart with 3 categories and 2 series (Budget, Actual). Highest: 150 (Budget in Feb). Lowest: 120 (Budget in Jan). 1 value is missing."
        ));
    }

    #[test]
    fn grouped_bars_reject_mismatched_series_length() {
        let short = GROUPED.replace("[130, 145, null]", "[130, 145]");
        let error = render_json(&short, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "series_length_mismatch");
        assert_eq!(error.path, "/series/1/values");
    }

    #[test]
    fn data_and_series_are_mutually_exclusive() {
        let both = GROUPED.replace(
            "\"categories\"",
            "\"data\": [{\"label\": \"Jan\", \"value\": 1}], \"categories\"",
        );
        let error = render_json(&both, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "conflicting_data_shape");
        assert_eq!(error.path, "/data");
    }

    #[test]
    fn line_charts_do_not_accept_series_yet() {
        let line = GROUPED.replace("\"bar\"", "\"line\"");
        let error = render_json(&line, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "option_not_supported");
        assert_eq!(error.path, "/series");
    }

    #[test]
    fn single_series_output_carries_no_series_styles() {
        let output = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(!output.content.contains("chartlet-series-"));
    }

    #[test]
    fn short_negative_bar_keeps_its_label_readable() {
        let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "orientation": "horizontal",
            "title": "Change",
            "valueAxis": {"format": "percent"},
            "data": [
                {"label": "A", "value": 0.3},
                {"label": "B", "value": -0.005}
            ]
        }"#;
        let output =
            render_json(specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(
            output
                .content
                .contains("text-anchor=\"end\" class=\"chartlet-value\">−0.5%</text>")
        );
        assert!(!output.content.contains("class=\"chartlet-value-inverse\""));
    }

    #[test]
    fn bar_chart_rejects_missing_values() {
        let missing = SPEC.replace("12}", "null}");
        let error =
            render_json(&missing, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "missing_bar_value");
        assert_eq!(error.path, "/data/0/value");
    }

    #[test]
    fn grouped_bars_render_series_filter_checkboxes() {
        let output = render_json(GROUPED, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            output
                .content
                .contains("<fieldset class=\"chartlet-filter\">")
        );
        assert!(output.content.contains("class=\"series-0\" checked"));
        assert!(output.content.contains("data-series=\"0\""));
        assert!(output.content.contains("<text data-series=\"0\""));
        assert!(output.content.contains("<title>Jan – Budget: 120</title>"));
    }

    #[test]
    fn single_series_bars_carry_native_tooltips() {
        let output = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(
            output
                .content
                .contains("<title>North &lt;East&gt;: 12</title>")
        );
        assert!(!output.content.contains("data-series"));
    }

    #[test]
    fn zoom_steps_render_radio_selectable_panels() {
        let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Quarterly revenue",
            "data": [
                {"label": "Q1", "value": 320},
                {"label": "Q2", "value": 345},
                {"label": "Q3", "value": 380},
                {"label": "Q4", "value": 410}
            ],
            "zoomSteps": [
                {"label": "First half", "from": 0, "to": 1},
                {"label": "All", "from": 0, "to": 3}
            ]
        }"#;
        let output =
            render_json(specification, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            output
                .content
                .contains("<fieldset class=\"chartlet-zoom\">")
        );
        assert!(
            output
                .content
                .contains("class=\"chartlet-panel chartlet-panel-0\"")
        );
        assert!(
            output
                .content
                .contains("class=\"chartlet-panel chartlet-panel-1\"")
        );
        assert!(output.content.contains("class=\"zoom-0\" checked"));
        // Every panel gets its own non-colliding accessibility IDs.
        assert_eq!(output.content.matches("<title id=").count(), 2);
    }

    #[test]
    fn a_single_zoom_step_is_rejected() {
        let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Quarterly revenue",
            "data": [
                {"label": "Q1", "value": 320},
                {"label": "Q2", "value": 345}
            ],
            "zoomSteps": [
                {"label": "All", "from": 0, "to": 1}
            ]
        }"#;
        let error = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("one zoom step cannot provide a choice");
        assert_eq!(error.code, "not_enough_zoom_steps");
        assert_eq!(error.path, "/zoomSteps");
    }

    const TIME: &str = r#"{
        "schemaVersion": 1,
        "type": "time",
        "title": "Daily orders",
        "timeAxis": {"timezone": "UTC", "title": "Day"},
        "panes": [
            {
                "valueAxis": {"title": "Orders"},
                "layers": [
                    {
                        "mark": "line",
                        "name": "Orders",
                        "points": [
                            {"time": "2026-03-01", "value": 10},
                            {"time": "2026-03-02", "value": 15},
                            {"time": "2026-03-03", "value": 20}
                        ]
                    }
                ]
            }
        ]
    }"#;

    /// A time chart with `points` hourly observations in one layer, for the density rules.
    fn hourly(points: usize, extra: &str) -> String {
        let sample = (0..points)
            .map(|index| {
                let offset = i64::try_from(index).expect("test sizes fit in an i64") * 3_600;
                format!(
                    "{{\"time\": {}, \"value\": {}}}",
                    1_770_000_000 + offset,
                    index % 5 + 1
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Hourly load",
                "timeAxis": {{"timezone": "UTC"}},
                "panes": [
                    {{
                        "valueAxis": {{"title": "Load"}},
                        "layers": [{{"mark": "line", "name": "Load"{extra}, "points": [{sample}]}}]
                    }}
                ]
            }}"#
        )
    }

    #[test]
    fn german_locale_writes_description_table_and_numbers_in_german() {
        let spec = TIME
            .replace(
                "\"type\": \"time\",",
                "\"type\": \"time\", \"locale\": \"de\", \"source\": \"Shop\",",
            )
            .replace(
                "\"valueAxis\": {\"title\": \"Orders\"}",
                "\"valueAxis\": {\"title\": \"Orders\", \"decimals\": 1}",
            )
            .replace("\"value\": 15}", "\"value\": -1.5}");
        let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
        let content = &html.content;
        assert!(content.contains(
            "Zeitreihe mit 3 Punkten von 2026-03-01 bis 2026-03-03. Höchster Wert: 20,0 (2026-03-03). Niedrigster Wert: \u{2212}1,5 (2026-03-02)."
        ));
        assert!(content.contains("<summary>Diagrammdaten anzeigen</summary>"));
        assert!(content.contains("<caption>Daten zu Daily orders</caption>"));
        assert!(content.contains("<th scope=\"col\">Zeit</th>"));
        assert!(content.contains("<td>\u{2212}1,5</td>"));
        assert!(content.contains("<p class=\"chartlet-source\">Quelle: Shop</p>"));
    }

    #[test]
    fn a_thin_line_is_drawn_thin_in_the_plot_and_in_the_legend() {
        let spec = TIME.replace(
            "\"name\": \"Orders\",",
            "\"name\": \"Orders\", \"stroke\": \"thin\",",
        ).replace(
            "]\n            }\n        ]",
            ", {\"mark\": \"line\", \"name\": \"Mean\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 12}, {\"time\": \"2026-03-03\", \"value\": 18}]}]\n            }\n        ]",
        );
        let svg = render_ok(&spec).content;
        assert!(svg.contains(".chartlet-line-thin{stroke-width:1}"));
        // The plotted line and its legend sample.
        assert_eq!(
            svg.matches("class=\"chartlet-line chartlet-line-thin chartlet-line-series-1\"")
                .count(),
            2
        );
        assert_eq!(
            svg.matches("class=\"chartlet-line chartlet-line-series-2\"")
                .count(),
            2
        );
        let rule = TIME.replace(
            "\"name\": \"Orders\",",
            "\"name\": \"Orders\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 1}, {\"time\": \"2026-03-02\", \"value\": 2}]}, {\"mark\": \"annotation\", \"value\": 1, \"label\": \"Limit\", \"stroke\": \"thin\"}, {\"mark\": \"line\", \"name\": \"Other\",",
        );
        assert_eq!(
            render_err(&rule),
            (
                "option_not_supported",
                "/panes/0/layers/1/stroke".to_owned()
            )
        );
    }

    #[test]
    fn a_locale_is_refused_where_texts_are_not_translated_yet() {
        let spec = SPEC.replace(
            "\"type\": \"bar\",",
            "\"type\": \"bar\", \"locale\": \"de\",",
        );
        assert_eq!(
            render_err(&spec),
            ("locale_not_supported", "/locale".to_owned())
        );
        let spec = TIME.replace(
            "\"valueAxis\": {\"title\": \"Orders\"}",
            "\"valueAxis\": {\"title\": \"Orders\", \"decimals\": 9}",
        );
        assert_eq!(
            render_err(&spec),
            ("invalid_decimals", "/panes/0/valueAxis/decimals".to_owned())
        );
    }

    #[test]
    fn time_chart_renders_a_line_marks_and_a_time_axis() {
        let svg = render_json(TIME, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(svg.content.contains("<polyline"));
        assert_eq!(svg.content.matches("class=\"chartlet-point\"").count(), 3);
        assert!(
            svg.content
                .contains("<title>2026-03-01 – Orders: 10</title>")
        );
        assert!(svg.content.contains(">2026-03-01<"));
        assert!(svg.content.contains(">Day<"));
        assert!(svg.content.contains(">Orders<"));
        assert!(svg.warnings.is_empty(), "{:?}", svg.warnings);

        let html = render_json(TIME, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            html.content
                .contains("<th scope=\"col\">Time</th><th scope=\"col\">Orders</th>")
        );
        assert!(
            html.content
                .contains("<th scope=\"row\">2026-03-03</th><td>20</td>")
        );
    }

    #[test]
    fn time_chart_without_a_description_gets_a_generated_one() {
        let output = render_json(TIME, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(output.content.contains(
            "Time chart with 3 points from 2026-03-01 to 2026-03-03. Highest: 20 (2026-03-03). Lowest: 10 (2026-03-01)."
        ));
    }

    #[test]
    fn the_timezone_moves_the_wall_clock_labels() {
        // A bare date is wall-clock midnight in the configured zone, so the offset does not move
        // it: +02:00 keeps the date labels.
        let bare = TIME.replace("\"timezone\": \"UTC\"", "\"timezone\": \"+02:00\"");
        let bare = render_json(&bare, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(bare.content.contains(">2026-03-01<"));
        assert!(!bare.content.contains(">2026-03-01 02:00<"));

        // Absolute seconds are read in the zone, so midnight in UTC is two in the morning
        // in +02:00 and the labels carry a time of day.
        let seconds = TIME
            .replace("\"time\": \"2026-03-01\"", "\"time\": 1772323200")
            .replace("\"time\": \"2026-03-02\"", "\"time\": 1772409600")
            .replace("\"time\": \"2026-03-03\"", "\"time\": 1772496000");
        let utc = render_json(&seconds, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(utc.content.contains(">2026-03-01<"));
        assert!(!utc.content.contains("00:00<"));

        let shifted = seconds.replace("\"timezone\": \"UTC\"", "\"timezone\": \"+02:00\"");
        let shifted = render_json(&shifted, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(shifted.content.contains(">2026-03-02 00:00<"));
        assert!(shifted.content.contains(">2026-03-01 12:00<"));

        // The data table names the observation in the zone as well.
        let zone = seconds.replace("\"timezone\": \"UTC\"", "\"timezone\": \"+02:00\"");
        let html = render_json(&zone, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            html.content
                .contains("<th scope=\"row\">2026-03-01 02:00</th>")
        );
    }

    #[test]
    fn timestamps_that_do_not_increase_are_rejected() {
        let reversed = TIME.replace("\"value\": 10", "\"value\": 99").replace(
            "\"time\": \"2026-03-01\", \"value\": 99",
            "\"time\": \"2026-03-03\", \"value\": 99",
        );
        let error = render_json(&reversed, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("a backwards series has no direction");
        assert_eq!(error.code, "unordered_time");
        assert_eq!(error.path, "/panes/0/layers/0/points/1/time");
    }

    #[test]
    fn fractional_and_out_of_range_timestamps_are_rejected() {
        for (time, message) in [
            ("12.5", "Unix seconds must be a whole number"),
            (
                "8000000000",
                "timestamps must lie between 1700-01-01 and 2200-01-01",
            ),
            (
                "\"March 1st\"",
                "expected an ISO 8601 date such as 2026-03-01 or 2026-03-01T12:00:00Z, or a year such as 1850",
            ),
        ] {
            let broken = TIME.replace("\"time\": \"2026-03-01\"", &format!("\"time\": {time}"));
            let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default())
                .expect_err("a timestamp outside the contract is refused");
            assert_eq!(
                (error.code, error.path.as_str()),
                ("invalid_time", "/panes/0/layers/0/points/0/time")
            );
            assert_eq!(error.message, message, "{time}");
        }
    }

    #[test]
    fn a_line_needs_two_observations() {
        let error = render_json(&hourly(1, ""), RenderFormat::Svg, &RenderOptions::default())
            .expect_err("a single observation is not a line");
        assert_eq!(error.code, "empty_series");
        assert_eq!(error.path, "/panes/0/layers/0/points");
    }

    #[test]
    fn dense_layers_are_drawn_as_a_line_without_markers() {
        let sparse = render_json(
            &hourly(60, ""),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .expect("60 observations stay readable");
        assert_eq!(
            sparse.content.matches("class=\"chartlet-point\"").count(),
            60
        );

        // One more observation and the markers would sit closer than eleven pixels apart.
        let dense = render_json(
            &hourly(61, ""),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap();
        assert!(dense.content.contains("<polyline"));
        assert_eq!(dense.content.matches("class=\"chartlet-point\"").count(), 0);
    }

    #[test]
    fn dense_layers_are_reported_once_they_outnumber_the_plot_pixels() {
        // The default width is 800, of which 704 pixels are plot.
        let output = render_json(
            &hourly(1_000, ""),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap();
        assert_eq!(output.warnings.len(), 1);
        assert_eq!(output.warnings[0].code, "dense_chart");
        assert_eq!(output.warnings[0].path, "/panes/0/layers/0/points");
        assert!(
            output.warnings[0]
                .message
                .contains("1000 observations exceed the 704 horizontal pixels"),
            "{}",
            output.warnings[0].message
        );
    }

    #[test]
    fn declared_layer_colors_reach_the_stylesheet() {
        let specification = include_str!("../examples/revenue-vs-forecast.json");
        let svg = render_json(specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(svg.content.contains(".chartlet-style-0{stroke:#7ea6ff}"));
        // A bare var() would drop the series on a page that never defines the variable.
        assert!(
            svg.content
                .contains(".chartlet-style-1{stroke:var(--chart-forecast,currentColor)}")
        );
        assert_eq!(svg.content.matches("chartlet-theme-dark").count(), 2);
        assert!(svg.content.contains("class=\"chartlet-background\""));
        assert!(svg.content.contains("class=\"chartlet-legend\""));
    }

    #[test]
    fn an_invalid_color_is_reported_and_replaced_by_the_neutral_tone() {
        let declared = hourly(3, ", \"color\": \"red\"");
        let output = render_json(&declared, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert_eq!(output.warnings.len(), 1);
        assert_eq!(output.warnings[0].code, "color_not_supported");
        assert_eq!(output.warnings[0].path, "/panes/0/layers/0/color");
        assert!(output.content.contains(".chartlet-style-0{stroke:#667085}"));
    }

    #[test]
    fn marks_from_later_milestones_say_when_they_arrive() {
        for mark in ["area", "ohlc", "band"] {
            let declared = TIME.replace("\"mark\": \"line\"", &format!("\"mark\": \"{mark}\""));
            let error = render_json(&declared, RenderFormat::Svg, &RenderOptions::default())
                .expect_err("an unbuilt mark is refused, not ignored");
            assert_eq!(error.code, "mark_not_implemented");
            assert_eq!(error.path, "/panes/0/layers/0/mark");
            assert!(error.message.contains(mark), "{}", error.message);
        }
    }

    #[test]
    fn a_null_value_is_rejected_until_gaps_arrive() {
        let gapped = TIME.replace("\"value\": 15", "\"value\": null");
        let error = render_json(&gapped, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("a gap cannot be told apart from a missing observation yet");
        assert_eq!(error.code, "option_not_supported");
        assert_eq!(error.path, "/panes/0/layers/0/points/1/value");
    }

    #[test]
    fn fields_that_belong_to_another_mark_are_rejected_by_name() {
        let annotated = TIME.replace(
            "\"name\": \"Orders\"",
            "\"name\": \"Orders\", \"label\": \"Peak\"",
        );
        let error = render_json(&annotated, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("a label belongs to an annotation");
        assert_eq!(error.code, "option_not_supported");
        assert_eq!(error.path, "/panes/0/layers/0/label");
        assert!(error.message.contains("annotation"), "{}", error.message);
    }

    #[test]
    fn time_charts_reject_the_bar_and_line_fields() {
        for (field, path) in [
            ("\"data\": [{\"label\": \"Jan\", \"value\": 1}],", "/data"),
            ("\"categories\": [\"Jan\"],", "/categories"),
            (
                "\"zoomSteps\": [{\"label\": \"A\", \"from\": 0, \"to\": 0}, {\"label\": \"B\", \"from\": 0, \"to\": 1}],",
                "/zoomSteps",
            ),
        ] {
            let mixed = TIME.replace(
                "\"schemaVersion\": 1,",
                &format!("\"schemaVersion\": 1, {field}"),
            );
            let error = render_json(&mixed, RenderFormat::Svg, &RenderOptions::default())
                .expect_err("a time chart is drawn from panes and layers");
            assert_eq!(error.code, "option_not_supported", "{field}");
            assert_eq!(error.path, path, "{field}");
        }
    }

    #[test]
    fn more_than_one_pane_is_rejected_in_this_alpha() {
        let two = TIME.replace(
            "\"panes\": [",
            "\"panes\": [{\"layers\": [{\"mark\": \"line\", \"name\": \"Load\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 1}, {\"time\": \"2026-03-02\", \"value\": 2}]}]},",
        );
        let error = render_json(&two, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("one pane is all this alpha draws");
        assert_eq!(error.code, "too_many_panes");
        assert_eq!(error.path, "/panes");
    }

    #[test]
    fn layer_names_are_required_and_unique_within_a_pane() {
        let second = "{\"mark\": \"line\", \"name\": \"Orders\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 1}, {\"time\": \"2026-03-02\", \"value\": 2}]}";
        let duplicated = TIME.replace("\"layers\": [", &format!("\"layers\": [{second},"));
        let error = render_json(&duplicated, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("two layers with one name cannot be told apart");
        assert_eq!(error.code, "duplicate_series");
        assert_eq!(error.path, "/panes/0/layers/1/name");

        let anonymous = TIME.replace("\"name\": \"Orders\", ", "");
        let error = render_json(&anonymous, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(error.warnings.is_empty());
    }

    // --- Uncertainty bands, reference lines, stripes, calendars, range bars, small multiples ---

    fn render_ok(spec: &str) -> super::RenderOutput {
        render_json(spec, RenderFormat::Svg, &RenderOptions::default())
            .unwrap_or_else(|error| panic!("should render: {error}"))
    }

    fn render_err(spec: &str) -> (&'static str, String) {
        let error = render_json(spec, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("should be refused");
        (error.code, error.path)
    }

    /// A time chart with one measured and one modeled layer; `band` is spliced into every point
    /// of the modeled layer and `extra` into its layer object.
    fn projection(band: &str, extra: &str) -> String {
        format!(
            r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Projection",
                "showValues": false,
                "panes": [{{
                    "layers": [
                        {{"mark": "line", "name": "Measured", "points": [
                            {{"time": "2000", "value": 0.4}},
                            {{"time": "2010", "value": 0.6}},
                            {{"time": "2020", "value": 1.0}}
                        ]}},
                        {{"mark": "line", "name": "Scenario", "modeled": true{extra}, "points": [
                            {{"time": "2020", "value": 1.0{band}}},
                            {{"time": "2050", "value": 1.8{band}}}
                        ]}}
                    ]
                }}]
            }}"#
        )
    }

    const BAND: &str = r#", "lower": 0.5, "upper": 2.5"#;

    #[test]
    fn a_modeled_band_is_hatched_dashed_and_named_modeled() {
        let output = render_ok(&projection(BAND, ""));
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // The band, its hatched copy, and the pattern it refers to by the chart's own prefix.
        assert!(svg.contains("class=\"chartlet-band chartlet-band-series-2\""));
        assert!(svg.contains("<pattern id=\""));
        assert!(svg.contains("-hatch-1\" width=\"6\""));
        assert!(svg.contains("fill=\"url(#chartlet-"));
        assert!(svg.contains("class=\"chartlet-hatch chartlet-style-1\""));
        assert!(svg.contains("chartlet-line chartlet-line-modeled chartlet-line-series-2"));
        assert!(svg.contains(">Scenario (modeled)<"));
        assert!(svg.contains("<title>2050 – Scenario: 1.8 (range 0.5 to 2.5)</title>"));
        assert!(svg.contains(
            "One line has a shaded band from its lower to its upper bound, hatched where modeled"
        ));
        // Annual data reads as years, not as January 1st.
        assert!(svg.contains("from 2000 to 2050"));

        let html = render_json(
            &projection(BAND, ""),
            RenderFormat::Html,
            &RenderOptions::default(),
        )
        .unwrap();
        assert!(html.content.contains(
            "<th scope=\"col\">Scenario</th><th scope=\"col\">Scenario (lower)</th><th scope=\"col\">Scenario (upper)</th>"
        ));
        assert!(html.content.contains(
            "<th scope=\"row\">2050</th><td>Missing</td><td>1.8</td><td>0.5</td><td>2.5</td>"
        ));
    }

    #[test]
    fn a_band_widens_the_value_scale() {
        let svg = render_ok(&projection(BAND, "")).content;
        // The upper edge of 2.5 lies above every value, so the axis has to reach it.
        assert!(svg.contains(">2.5<"), "{svg}");
    }

    #[test]
    fn a_band_needs_both_edges_on_every_point() {
        let half = projection(r#", "lower": 0.5"#, "");
        assert_eq!(
            render_err(&half),
            (
                "incomplete_band",
                "/panes/0/layers/1/points/0/upper".to_owned()
            )
        );
        let mixed = projection(BAND, "").replacen(
            r#""value": 1.8, "lower": 0.5, "upper": 2.5"#,
            r#""value": 1.8"#,
            1,
        );
        assert_eq!(
            render_err(&mixed),
            (
                "incomplete_band",
                "/panes/0/layers/1/points/1/lower".to_owned()
            )
        );
        let inverted = projection(r#", "lower": 2.5, "upper": 0.5"#, "");
        assert_eq!(
            render_err(&inverted),
            (
                "invalid_band",
                "/panes/0/layers/1/points/0/upper".to_owned()
            )
        );
    }

    #[test]
    fn a_value_outside_its_band_is_reported() {
        let output = render_ok(&projection(r#", "lower": 0.1, "upper": 0.2"#, ""));
        assert_eq!(output.warnings.len(), 2);
        assert_eq!(output.warnings[0].code, "value_outside_band");
        assert_eq!(output.warnings[0].path, "/panes/0/layers/1/points/0/value");
    }

    #[test]
    fn a_declared_color_paints_line_band_and_hatch() {
        let svg = render_ok(&projection(BAND, r##", "color": "#7c3aed""##)).content;
        assert!(svg.contains(".chartlet-band.chartlet-style-1{fill:#7c3aed;stroke:none}"));
        assert!(svg.contains("class=\"chartlet-hatch-line chartlet-style-1\""));
    }

    #[test]
    fn the_band_mark_points_to_the_line_band() {
        let declared = TIME.replace("\"mark\": \"line\"", "\"mark\": \"band\"");
        let error = render_json(&declared, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the zone band is a later milestone");
        assert_eq!(error.code, "mark_not_implemented");
        assert!(
            error.message.contains("lower and upper"),
            "{}",
            error.message
        );
    }

    fn with_rules(rules: &str) -> String {
        TIME.replace("\"layers\": [", &format!("\"layers\": [{rules},"))
    }

    #[test]
    fn reference_lines_are_drawn_labelled_and_described() {
        let spec = with_rules(
            r#"{"mark": "annotation", "value": 30, "label": "Target"},
               {"mark": "annotation", "time": "2026-03-02", "label": "Launch"}"#,
        );
        let output = render_ok(&spec);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert_eq!(svg.matches("class=\"chartlet-rule\"").count(), 2);
        assert!(svg.contains(">Target</text>"));
        assert!(svg.contains(">Launch</text>"));
        assert!(svg.contains("<title>Target: 30</title>"));
        assert!(svg.contains("<title>Launch: 2026-03-02</title>"));
        // 30 lies above every observation, so the axis reaches it.
        assert!(svg.contains(">30<"));
        assert!(svg.contains("Reference lines: Target at 30; Launch at 2026-03-02."));
        // Reference lines are neither series in the legend nor columns in the table.
        assert!(!svg.contains("class=\"chartlet-legend\""));
        let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            html.content
                .contains("<th scope=\"col\">Time</th><th scope=\"col\">Orders</th></tr>")
        );
    }

    #[test]
    fn reference_lines_are_validated_by_name() {
        for (rule, code, path) in [
            (
                r#"{"mark": "annotation", "value": 1}"#,
                "missing_label",
                "/panes/0/layers/0/label",
            ),
            (
                r#"{"mark": "annotation", "label": "Nowhere"}"#,
                "missing_position",
                "/panes/0/layers/0",
            ),
            (
                r#"{"mark": "annotation", "value": 1, "time": "2026-03-01", "label": "Point"}"#,
                "option_not_supported",
                "/panes/0/layers/0/value",
            ),
            (
                r#"{"mark": "annotation", "value": 1, "label": "L", "shape": "circle"}"#,
                "option_not_supported",
                "/panes/0/layers/0/shape",
            ),
            (
                r#"{"mark": "annotation", "value": 1, "label": "L", "name": "N"}"#,
                "option_not_supported",
                "/panes/0/layers/0/name",
            ),
            (
                r#"{"mark": "annotation", "time": "1600-01-01", "label": "L"}"#,
                "invalid_time",
                "/panes/0/layers/0/time",
            ),
        ] {
            assert_eq!(
                render_err(&with_rules(rule)),
                (code, path.to_owned()),
                "{rule}"
            );
        }
        let only_rules = r#"{"schemaVersion": 1, "type": "time", "title": "T", "panes": [
            {"layers": [{"mark": "annotation", "value": 1, "label": "L"}]}]}"#;
        assert_eq!(
            render_err(only_rules),
            ("empty_data", "/panes/0/layers".to_owned())
        );
        let seven = (0..7)
            .map(|index| {
                format!(r#"{{"mark": "annotation", "value": {index}, "label": "L{index}"}}"#)
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            render_err(&with_rules(&seven)),
            ("too_many_annotations", "/panes/0/layers".to_owned())
        );
    }

    const STRIPES: &str = r#"{
        "schemaVersion": 1,
        "type": "stripes",
        "title": "Stripes",
        "stripes": {"firstYear": 1850, "values": [-1, -0.5, null, 0, 0.5, 2], "min": -1, "max": 1}
    }"#;

    #[test]
    fn stripes_draw_one_colored_stripe_per_year() {
        let output = render_ok(STRIPES);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // Five values; the missing year stays empty.
        assert_eq!(svg.matches("class=\"chartlet-diverging-").count(), 5);
        assert!(svg.contains("class=\"chartlet-diverging-0\"><title>1850: −1</title>"));
        assert!(svg.contains("class=\"chartlet-diverging-4\"><title>1851: −0.5</title>"));
        assert!(svg.contains("class=\"chartlet-diverging-8\"><title>1853: 0</title>"));
        // Beyond max takes the outermost step.
        assert!(svg.contains("class=\"chartlet-diverging-16\"><title>1855: 2</title>"));
        assert!(svg.contains(">1850</text>"));
        assert!(svg.contains(">1855</text>"));
        assert!(svg.contains("--chartlet-diverging-8:#eeeeee"));
        assert!(svg.contains(
            "Warming stripes from 1850 to 1855, one stripe per year, on a diverging color scale around 0 with its outermost steps at −1 and 1. Lowest: −1 (1850). Highest: 2 (1855). 1 year has no value."
        ));
        let html = render_json(STRIPES, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(html.content.contains("<th scope=\"col\">Year</th>"));
        assert!(
            html.content
                .contains("<th scope=\"row\">1852</th><td>Missing</td>")
        );
    }

    #[test]
    fn stripes_are_validated_by_name() {
        for (from, to, code, path) in [
            ("\"min\": -1", "\"min\": 0", "invalid_scale", "/stripes/min"),
            ("\"max\": 1", "\"max\": -2", "invalid_scale", "/stripes/max"),
            (
                "\"firstYear\": 1850",
                "\"firstYear\": 9998",
                "invalid_year",
                "/stripes/firstYear",
            ),
            (
                "[-1, -0.5, null, 0, 0.5, 2]",
                "[null]",
                "empty_series",
                "/stripes/values",
            ),
            (
                "[-1, -0.5, null, 0, 0.5, 2]",
                "[]",
                "empty_data",
                "/stripes/values",
            ),
            (
                "\"title\": \"Stripes\",",
                "\"title\": \"Stripes\", \"panes\": [{}],",
                "option_not_supported",
                "/panes",
            ),
        ] {
            assert_eq!(
                render_err(&STRIPES.replace(from, to)),
                (code, path.to_owned()),
                "{to}"
            );
        }
        let missing = STRIPES.replace(
            r#""stripes": {"firstYear": 1850, "values": [-1, -0.5, null, 0, 0.5, 2], "min": -1, "max": 1}"#,
            "\"width\": 800",
        );
        assert_eq!(
            render_err(&missing),
            ("missing_stripes", "/stripes".to_owned())
        );
    }

    #[test]
    fn a_block_of_another_type_is_rejected() {
        let foreign = SPEC.replace(
            "\"title\": \"Profit & loss\",",
            "\"title\": \"Profit & loss\", \"stripes\": {\"firstYear\": 2000, \"values\": [1]},",
        );
        assert_eq!(
            render_err(&foreign),
            ("option_not_supported", "/stripes".to_owned())
        );
        let columns = TIME.replace("\"type\": \"time\",", "\"type\": \"time\", \"columns\": 2,");
        assert_eq!(
            render_err(&columns),
            ("option_not_supported", "/columns".to_owned())
        );
    }

    fn calendar(layout: &str, days: &str) -> String {
        format!(
            r#"{{
                "schemaVersion": 1,
                "type": "calendar",
                "title": "Calendar",
                "calendar": {{"year": 2024, "layout": "{layout}", "days": [{days}]}}
            }}"#
        )
    }

    const DAYS: &str = r#"{"date": "2024-01-01", "value": -2}, {"date": "2024-02-29", "value": 4}, {"date": "2024-12-31", "value": null}"#;

    #[test]
    fn a_calendar_has_a_cell_for_every_day_and_leaves_gaps_empty() {
        for layout in ["months", "weeks"] {
            let output = render_ok(&calendar(layout, DAYS));
            let svg = &output.content;
            assert!(output.warnings.is_empty(), "{:?}", output.warnings);
            // 2024 is a leap year: 366 cells, of which two carry a value; the key adds 17.
            assert_eq!(svg.matches("<rect").count(), 366 + 17, "{layout}");
            assert_eq!(
                svg.matches("class=\"chartlet-calendar-empty\"").count(),
                364
            );
            assert!(svg.contains("class=\"chartlet-diverging-4\"><title>2024-01-01: −2</title>"));
            assert!(svg.contains("class=\"chartlet-diverging-16\"><title>2024-02-29: 4</title>"));
            assert!(svg.contains("<title>2024-12-31: no value</title>"));
            assert!(svg.contains(
                "2 of 366 days have a value. Lowest: −2 (2024-01-01). Highest: 4 (2024-02-29)."
            ));
        }
        let weeks = render_ok(&calendar("weeks", DAYS)).content;
        assert!(weeks.contains(">Mon</text>"));
        let months = render_ok(&calendar("months", DAYS)).content;
        assert!(months.contains(">Feb</text>"));
        let html = render_json(
            &calendar("months", DAYS),
            RenderFormat::Html,
            &RenderOptions::default(),
        )
        .unwrap();
        assert!(
            html.content
                .contains("<th scope=\"row\">2024-02-29</th><td>4</td>")
        );
    }

    #[test]
    fn calendar_days_are_validated_by_name() {
        for (days, code, path) in [
            (
                r#"{"date": "2023-12-31", "value": 1}"#,
                "date_outside_year",
                "/calendar/days/0/date",
            ),
            (
                r#"{"date": "2024-02-30", "value": 1}"#,
                "invalid_date",
                "/calendar/days/0/date",
            ),
            (
                r#"{"date": "2024-03-01T12:00", "value": 1}"#,
                "invalid_date",
                "/calendar/days/0/date",
            ),
            (
                r#"{"date": "2024-03-01", "value": 1}, {"date": "2024-03-01", "value": 2}"#,
                "duplicate_date",
                "/calendar/days/1/date",
            ),
            (
                r#"{"date": "2024-03-01", "value": null}"#,
                "empty_series",
                "/calendar/days",
            ),
            ("", "empty_data", "/calendar/days"),
        ] {
            assert_eq!(
                render_err(&calendar("months", days)),
                (code, path.to_owned()),
                "{days}"
            );
        }
    }

    const RANGES: &str = r#"{
        "schemaVersion": 1,
        "type": "rangebar",
        "orientation": "horizontal",
        "title": "Contributions",
        "ranges": [
            {"label": "Observed", "low": 0.9, "high": 1.2, "mid": 1.05},
            {"label": "Natural", "low": -0.1, "high": 0.1, "modeled": true}
        ]
    }"#;

    #[test]
    fn range_bars_draw_spans_marks_and_hatching() {
        for orientation in ["horizontal", "vertical"] {
            let spec = RANGES.replace("\"horizontal\"", &format!("\"{orientation}\""));
            let output = render_ok(&spec);
            let svg = &output.content;
            assert_eq!(
                svg.matches("class=\"chartlet-range\"").count(),
                3,
                "{orientation}"
            );
            assert_eq!(svg.matches("class=\"chartlet-range-hatch\"").count(), 2);
            assert_eq!(svg.matches("class=\"chartlet-range-mid\"").count(), 1);
            assert!(svg.contains("-hatch-range\" width=\"6\""));
            assert!(svg.contains("<title>Observed: 1.05 (0.9 to 1.2)</title>"));
            assert!(svg.contains("<title>Natural: −0.1 to 0.1, modeled</title>"));
            assert!(svg.contains(">Hatched: modeled<"));
            assert!(svg.contains(
                "Range chart with 2 categories, each a span from low to high and a central value. Lowest low: −0.1 (Natural). Highest high: 1.2 (Observed). Modeled, drawn hatched: Natural."
            ));
        }
        let html = render_json(RANGES, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(html.content.contains(
            "<th scope=\"col\">Low</th><th scope=\"col\">Mid</th><th scope=\"col\">High</th>"
        ));
        assert!(html.content.contains(
            "<th scope=\"row\">Natural (modeled)</th><td>−0.1</td><td>Missing</td><td>0.1</td>"
        ));
    }

    #[test]
    fn ranges_are_validated_by_name() {
        for (from, to, code, path) in [
            (
                "\"low\": 0.9, \"high\": 1.2",
                "\"low\": 1.3, \"high\": 1.2",
                "invalid_range",
                "/ranges/0/high",
            ),
            (
                "\"mid\": 1.05",
                "\"mid\": 2",
                "mid_outside_range",
                "/ranges/0/mid",
            ),
            (
                "\"label\": \"Natural\"",
                "\"label\": \"Observed\"",
                "duplicate_label",
                "/ranges/1/label",
            ),
            (
                "\"orientation\": \"horizontal\",",
                "\"orientation\": \"horizontal\", \"data\": [{\"label\": \"A\", \"value\": 1}],",
                "option_not_supported",
                "/data",
            ),
        ] {
            assert_eq!(
                render_err(&RANGES.replace(from, to)),
                (code, path.to_owned()),
                "{to}"
            );
        }
    }

    fn multiples(panes: &str) -> String {
        format!(
            r#"{{
                "schemaVersion": 1,
                "type": "multiples",
                "title": "Sectors",
                "valueAxis": {{"title": "Emissions"}},
                "panes": [{panes}]
            }}"#
        )
    }

    fn panel(title: &str, scale: f64) -> String {
        format!(
            r#"{{"title": "{title}", "layers": [
                {{"mark": "line", "name": "History", "points": [{{"time": "2000", "value": {}}}, {{"time": "2010", "value": {}}}]}},
                {{"mark": "line", "name": "Pathway", "modeled": true, "points": [{{"time": "2010", "value": {}, "lower": 0, "upper": {}}}, {{"time": "2050", "value": 0, "lower": 0, "upper": {}}}]}},
                {{"mark": "annotation", "value": 0, "label": "Net zero"}}
            ]}}"#,
            10.0 * scale,
            12.0 * scale,
            12.0 * scale,
            14.0 * scale,
            2.0 * scale
        )
    }

    #[test]
    fn small_multiples_share_one_scale_and_one_legend() {
        let spec = multiples(
            &[
                panel("Energy", 2.0),
                panel("Industry", 1.0),
                panel("Transport", 0.5),
            ]
            .join(","),
        );
        let output = render_ok(&spec);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(svg.contains("class=\"chartlet-root chartlet-multiples\""));
        for title in ["Energy", "Industry", "Transport"] {
            assert!(svg.contains(&format!(">{title}</text>")));
        }
        // One shared scale: the top tick of the largest panel appears in every panel.
        assert_eq!(svg.matches(">30</text>").count(), 3);
        // One legend entry per name, not per panel; the same name keeps its color everywhere.
        assert_eq!(svg.matches("class=\"chartlet-legend\"").count(), 2);
        assert!(svg.contains(">Pathway (modeled)<"));
        assert_eq!(
            svg.matches("chartlet-line chartlet-line-modeled chartlet-line-series-2")
                .count(),
            // Three panels and the legend sample.
            4
        );
        assert!(svg.contains("<title>2050 – Industry · Pathway: 0 (range 0 to 2)</title>"));
        assert!(svg.contains(
            "Small multiples of 3 panels (Energy, Industry, Transport) with a shared value axis, each from 2000 to 2050, showing History, Pathway (modeled)."
        ));
        let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            html.content
                .contains("<th scope=\"col\">Energy · History</th>")
        );
        assert!(
            html.content
                .contains("<th scope=\"col\">Transport · Pathway (upper)</th>")
        );
    }

    #[test]
    fn small_multiples_are_validated_by_name() {
        let two = [panel("A", 1.0), panel("B", 1.0)].join(",");
        assert_eq!(
            render_err(&multiples(&panel("A", 1.0))),
            ("not_enough_panes", "/panes".to_owned())
        );
        assert_eq!(
            render_err(&multiples(&two.replacen("\"title\": \"A\", ", "", 1))),
            ("missing_title", "/panes/0/title".to_owned())
        );
        assert_eq!(
            render_err(&multiples(
                &two.replace("\"title\": \"B\"", "\"title\": \"A\"")
            )),
            ("duplicate_title", "/panes/1/title".to_owned())
        );
        assert_eq!(
            render_err(&multiples(&two.replacen(
                "\"title\": \"A\",",
                "\"title\": \"A\", \"valueAxis\": {\"title\": \"Own\"},",
                1
            ))),
            ("option_not_supported", "/panes/0/valueAxis".to_owned())
        );
        let columns = multiples(&two).replace(
            "\"type\": \"multiples\",",
            "\"type\": \"multiples\", \"columns\": 7,",
        );
        assert_eq!(
            render_err(&columns),
            ("invalid_columns", "/columns".to_owned())
        );
        let thirteen = (0..13)
            .map(|index| panel(&format!("P{index}"), 1.0))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            render_err(&multiples(&thirteen)),
            ("too_many_panes", "/panes".to_owned())
        );
        let titled = TIME.replace(
            "\"valueAxis\": {\"title\": \"Orders\"},",
            "\"title\": \"Pane\",",
        );
        assert_eq!(
            render_err(&titled),
            ("option_not_supported", "/panes/0/title".to_owned())
        );
    }

    const ATLAS: &str = r#"{
        "schemaVersion": 1,
        "type": "atlas",
        "title": "Knowledge landscape",
        "atlas": {
            "realms": [
                {
                    "label": "Machines",
                    "regions": [
                        {
                            "label": "Models",
                            "value": 40,
                            "places": [{ "label": "A first look", "weight": 2 }]
                        },
                        { "label": "Agents", "value": 12 }
                    ]
                },
                {
                    "label": "Craft",
                    "regions": [
                        { "label": "Tooling", "value": 30 },
                        { "label": "Accessibility", "value": 8 }
                    ]
                }
            ],
            "links": [{ "from": "Agents", "to": "Tooling", "weight": 0.4 }],
            "seed": 7
        }
    }"#;

    const TOPICMAP: &str = r#"{
        "schemaVersion": 1,
        "type": "topicmap",
        "title": "Insights themes",
        "topicmap": {
            "topics": [
                {"label": "AI in practice", "value": 291, "points": 41},
                {"label": "Engineering", "value": 282, "points": 24},
                {"label": "Cloud native", "value": 100, "points": 9},
                {"label": "Web marketing", "value": 95, "points": 7},
                {"label": "Management", "value": 69, "points": 5}
            ],
            "links": [
                {"from": "AI in practice", "to": "Engineering", "weight": 0.6}
            ],
            "islands": [
                {"label": "WASM", "value": 6, "points": 1}
            ],
            "cartouche": {"heading": "Topic map", "meta": "843 entries · 87 paths · 2026"}
        }
    }"#;

    #[test]
    fn topicmap_renders_areas_labels_and_links() {
        let output = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-area")
                .count(),
            6
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-link\"")
                .count(),
            1
        );
        assert!(output.content.contains(
            "Topic map with 5 areas and one island. Largest: AI in practice with 291 entries, smallest: Management with 69. Closest neighbours: AI in practice and Engineering."
        ));
        // Two depth lines around each of the six areas, a halo under every coastline, and one
        // point per path through a topic.
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-band\"")
                .count(),
            12
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-halo")
                .count(),
            6
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-point")
                .count(),
            41 + 24 + 9 + 7 + 5 + 1
        );
        // A coastline is a closed ring, not a circle.
        assert!(!output.content.contains("class=\"chartlet-topic-area\"/>"));

        // At this size no area carries a name at the type size the design brief asks for, so
        // every label sits beside the map with a leader line to its area. No name is lost.
        for label in [
            "AI in practice",
            "Engineering",
            "Cloud native",
            "Web marketing",
            "Management",
            "WASM",
        ] {
            assert!(output.content.contains(&format!(">{label}<")), "{label}");
        }
        assert_eq!(texts(&output.content, "chartlet-topic-outside").len(), 6);
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-leader\"")
                .count(),
            6
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-label")
                .count(),
            0
        );
        assert_eq!(
            output
                .warnings
                .iter()
                .map(|warning| (warning.code, warning.path.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("label_outside_area", "/topicmap/topics/0/label"),
                ("label_outside_area", "/topicmap/topics/1/label"),
                ("label_outside_area", "/topicmap/topics/2/label"),
                ("label_outside_area", "/topicmap/topics/3/label"),
                ("label_outside_area", "/topicmap/topics/4/label"),
                ("label_outside_area", "/topicmap/islands/0/label"),
            ]
        );
    }

    #[test]
    fn topicmap_draws_its_map_furniture_in_order() {
        let output = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        let at = |needle: &str| {
            output
                .content
                .find(needle)
                .unwrap_or_else(|| panic!("{needle} is missing"))
        };
        // The sea and its grid go underneath everything, the compass rose and the cartouche on
        // top of it: furniture must never hide an area, and an area must never hide the legend.
        assert!(at("class=\"chartlet-sea\"") < at("class=\"chartlet-topic-band\""));
        assert!(at("class=\"chartlet-graticule\"") < at("class=\"chartlet-topic-area"));
        assert!(at("class=\"chartlet-topic-area") < at("class=\"chartlet-compass-ring\""));
        assert!(at("class=\"chartlet-compass-ring\"") < at("class=\"chartlet-cartouche\""));
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-graticule\"")
                .count(),
            6
        );
        assert!(output.content.contains(">Topic map</text>"));
        assert!(
            output
                .content
                .contains(">843 entries · 87 paths · 2026</text>")
        );
        assert!(output.content.contains(">N</text>"));
    }

    #[test]
    fn topicmap_leaves_out_the_furniture_it_was_not_asked_for() {
        let bare = r#"{
            "schemaVersion": 1,
            "type": "topicmap",
            "title": "Plain",
            "topicmap": {
                "graticule": false,
                "compass": false,
                "topics": [
                    {"label": "One", "value": 40},
                    {"label": "Two", "value": 20}
                ]
            }
        }"#;
        // The stylesheet always carries every topic map rule, so this looks for drawn elements.
        let output = render_json(bare, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(!output.content.contains("class=\"chartlet-graticule\""));
        assert!(!output.content.contains("class=\"chartlet-compass-ring\""));
        assert!(!output.content.contains("class=\"chartlet-cartouche\""));
        // The sea stays: it is the surface the map is drawn on, not an optional ornament.
        assert!(output.content.contains("class=\"chartlet-sea\""));
    }

    #[test]
    fn topicmap_html_offers_a_radio_group_for_the_areas() {
        let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            output
                .content
                .contains("<fieldset class=\"chartlet-topic-picker\"><legend>Area</legend>")
        );
        // One radio per area, all in one group, the first one selected. Islands are not offered:
        // the picker is about what the map is of.
        assert_eq!(output.content.matches("type=\"radio\"").count(), 5);
        assert_eq!(
            output.content.matches("class=\"topic-0\" checked").count(),
            1
        );
        assert!(
            output
                .content
                .contains("class=\"topic-4\"> Management</label>")
        );
        assert!(!output.content.contains("WASM</label>"));
        // The radio group is named after the chart, so two maps on one page stay independent.
        assert!(output.content.contains("name=\"topic-chartlet-"));
    }

    #[test]
    fn topicmap_selection_styles_only_apply_once_something_is_selected() {
        let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
        // Every selection rule hangs off a checked radio. A browser without :has() drops them
        // all, which leaves the map in its plain state rather than in a half-applied one — and
        // panels the host page provides stay visible instead of being hidden with no way back.
        for rule in output
            .content
            .split('}')
            .filter(|rule| rule.contains(".chartlet-topic-panel"))
        {
            assert!(
                rule.contains(":has("),
                "a panel rule applies unconditionally: {rule}"
            );
        }
        assert!(output.content.contains(
            ".chartlet-wrapper:has(.chartlet-topic-picker input.topic-2:checked) .chartlet-topic-area.chartlet-topic-2{fill:var(--chartlet-accent)}"
        ));
        // The plain SVG profile carries no selection at all.
        let svg = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(!svg.content.contains("chartlet-topic-picker"));
    }

    #[test]
    fn topicmap_table_carries_entries_paths_and_share() {
        let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(output.content.contains(
            "<th scope=\"col\">Topic</th><th scope=\"col\">Entries</th><th scope=\"col\">Paths</th><th scope=\"col\">Share</th>"
        ));
        // 291 of 843 entries, as a percentage of the whole map, islands included.
        assert!(output.content.contains(
            "<th scope=\"row\">AI in practice</th><td>291</td><td>41</td><td>34.52%</td>"
        ));
        assert!(
            output
                .content
                .contains("<th scope=\"row\">WASM</th><td>6</td><td>1</td><td>0.71%</td>")
        );
    }

    #[test]
    fn topicmap_layout_is_deterministic() {
        let first = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        let second = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert_eq!(first.content, second.content);
    }

    /// Every `<text>` in the SVG as (x, size, anchor, content).
    fn texts(svg: &str, class: &str) -> Vec<(f64, f64, String, String)> {
        let mut found = Vec::new();
        for chunk in svg.split("<text ").skip(1) {
            let attributes = chunk.split('>').next().expect("a tag ends");
            // An element can carry more than one class, so this compares whole names: looking for
            // "chartlet-topic-outside" must not find "chartlet-topic-outside-value".
            let classes = attributes
                .split("class=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .unwrap_or_default();
            if !classes.split_whitespace().any(|name| name == class) {
                continue;
            }
            let read = |name: &str| {
                attributes
                    .split(&format!("{name}=\""))
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .map(str::to_owned)
            };
            let content = chunk
                .split_once('>')
                .and_then(|(_, rest)| rest.split_once("</text>"))
                .map(|(content, _)| content.to_owned())
                .expect("a text element has content");
            found.push((
                read("x").expect("x").parse().expect("a number"),
                read("font-size")
                    .expect("font-size")
                    .parse()
                    .expect("a number"),
                read("text-anchor").expect("anchor"),
                content,
            ));
        }
        found
    }

    #[test]
    fn topicmap_labels_moved_outside_stay_on_the_canvas_and_clear_of_each_other() {
        // Twenty-four areas: nearly every label has to go beside the map, which is where labels
        // start running off the canvas or landing on top of one another.
        let topics = (0..24)
            .map(|index| {
                format!(
                    r#"{{"label": "Topic number {index}", "value": {}}}"#,
                    20 + (index * 37) % 260
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let specification = format!(
            r#"{{"schemaVersion": 1, "type": "topicmap", "title": "Crowded",
                 "topicmap": {{"topics": [{topics}]}}}}"#
        );
        let output =
            render_json(&specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();

        let metrics = BuiltinMetrics;
        let mut placed: Vec<(f64, f64, f64, f64)> = Vec::new();
        for (x, size, anchor, content) in texts(&output.content, "chartlet-topic-outside") {
            let width = metrics.width(&content, size);
            let (left, right) = if anchor == "start" {
                (x, x + width)
            } else {
                (x - width, x)
            };
            assert!(left >= 0.0, "{content} starts off the canvas at {left}");
            assert!(right <= 800.0, "{content} ends off the canvas at {right}");
            placed.push((left, right, x, size));
        }
        assert!(placed.len() > 12, "most labels should sit outside");

        // No label may be dropped silently: whatever is not drawn is reported.
        let reported = output
            .warnings
            .iter()
            .filter(|warning| warning.code == "dense_chart")
            .count();
        assert_eq!(placed.len() + reported, 24);
    }

    #[test]
    fn topicmap_caps_links_per_topic_and_drops_the_weakest() {
        let specification = r#"{
            "schemaVersion": 1,
            "type": "topicmap",
            "title": "Busy hub",
            "topicmap": {
                "topics": [
                    {"label": "Hub", "value": 200},
                    {"label": "A", "value": 50},
                    {"label": "B", "value": 50},
                    {"label": "C", "value": 50},
                    {"label": "D", "value": 50},
                    {"label": "E", "value": 50}
                ],
                "links": [
                    {"from": "Hub", "to": "A", "weight": 0.9},
                    {"from": "Hub", "to": "B", "weight": 0.8},
                    {"from": "Hub", "to": "C", "weight": 0.7},
                    {"from": "Hub", "to": "D", "weight": 0.6},
                    {"from": "Hub", "to": "E", "weight": 0.5}
                ]
            }
        }"#;
        let output =
            render_json(specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-topic-link\"")
                .count(),
            3
        );
        assert_eq!(
            output
                .warnings
                .iter()
                .map(|warning| (warning.code, warning.path.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("link_dropped", "/topicmap/links/3"),
                ("link_dropped", "/topicmap/links/4"),
            ]
        );
    }

    #[test]
    fn atlas_renders_and_describes_its_landscape() {
        let svg = render_json(ATLAS, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(
            svg.content
                .contains("Knowledge landscape of 2 realms and 4 regions")
        );
        assert!(svg.content.contains("largest is Models with 40"));
        assert!(svg.content.contains("1 places are marked"));
    }

    #[test]
    fn atlas_draws_its_terrain_unless_it_is_turned_off() {
        let drawn = render_json(ATLAS, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        // The class is always in the stylesheet; what the switch decides is the elements.
        assert!(drawn.content.contains("class=\"chartlet-atlas-contour\""));
        let flat = ATLAS.replace(r#""seed": 7"#, r#""seed": 7, "contours": false"#);
        let plain = render_json(&flat, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(!plain.content.contains("class=\"chartlet-atlas-contour\""));
        // The land itself is untouched by the switch.
        assert!(plain.content.contains("class=\"chartlet-atlas-coast\""));
    }

    #[test]
    fn atlas_requires_an_atlas_block() {
        let missing = r#"{"schemaVersion": 1, "type": "atlas", "title": "Empty"}"#;
        let error = render_json(missing, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "missing_atlas");
        assert_eq!(error.path, "/atlas");
    }

    #[test]
    fn atlas_rejects_a_realm_without_regions() {
        let empty = ATLAS.replace(r#"{ "label": "Tooling", "value": 30 },"#, "");
        let empty = empty.replace(r#"{ "label": "Accessibility", "value": 8 }"#, "");
        let error = render_json(&empty, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "empty_realm");
        assert_eq!(error.path, "/atlas/realms/1/regions");
    }

    #[test]
    fn atlas_rejects_a_region_that_repeats_a_realm_name() {
        let clash = ATLAS.replace(r#""label": "Tooling""#, r#""label": "Machines""#);
        let error = render_json(&clash, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "duplicate_atlas_label");
        assert_eq!(error.path, "/atlas/realms/1/regions/0/label");
    }

    #[test]
    fn atlas_rejects_links_to_unknown_regions() {
        let broken = ATLAS.replace(r#""to": "Tooling""#, r#""to": "Machines""#);
        let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "unknown_region_link");
        assert_eq!(error.path, "/atlas/links/0/to");
    }

    #[test]
    fn atlas_rejects_area_damping_outside_its_range() {
        let broken = ATLAS.replace(r#""seed": 7"#, r#""seed": 7, "areaDamping": 1.5"#);
        let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "invalid_area_damping");
        assert_eq!(error.path, "/atlas/areaDamping");
    }

    #[test]
    fn atlas_warns_when_a_region_lists_more_places_than_it_claims() {
        let crowded = r#"{
            "schemaVersion": 1,
            "type": "atlas",
            "title": "Miscounted",
            "atlas": {
                "realms": [
                    {
                        "label": "Machines",
                        "regions": [
                            {
                                "label": "Models",
                                "value": 1,
                                "places": [{ "label": "One" }, { "label": "Two" }]
                            }
                        ]
                    }
                ]
            }
        }"#;
        let rendered = render_json(crowded, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(
            rendered
                .warnings
                .iter()
                .any(|warning| warning.code == "more_places_than_value"),
            "expected the miscount warning, got {:?}",
            rendered.warnings
        );
    }

    #[test]
    fn atlas_rejects_a_place_weight_outside_its_range() {
        let broken = ATLAS.replace(r#""weight": 2 }]"#, r#""weight": 9 }]"#);
        let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "place_weight_out_of_range");
        assert_eq!(error.path, "/atlas/realms/0/regions/0/places/0/weight");
    }

    #[test]
    fn atlas_rejects_series_alongside_its_realms() {
        let mixed = ATLAS.replace(
            r#""title": "Knowledge landscape","#,
            r#""title": "Knowledge landscape", "data": [{"label": "A", "value": 1}],"#,
        );
        let error = render_json(&mixed, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "option_not_supported");
        assert_eq!(error.path, "/data");
        assert!(
            error
                .message
                .contains("an atlas chart is drawn from atlas.realms")
        );
    }

    #[test]
    fn topicmap_requires_a_topicmap_block() {
        let missing = r#"{"schemaVersion": 1, "type": "topicmap", "title": "Empty"}"#;
        let error = render_json(missing, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "missing_topicmap");
        assert_eq!(error.path, "/topicmap");
    }

    #[test]
    fn topicmap_rejects_too_many_topics() {
        let topics = (0..41)
            .map(|i| format!(r#"{{"label": "T{i}", "value": 10}}"#))
            .collect::<Vec<_>>()
            .join(",");
        let spec = format!(
            r#"{{"schemaVersion": 1, "type": "topicmap", "title": "Big", "topicmap": {{"topics": [{topics}]}}}}"#
        );
        let error = render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "too_many_topics");
        assert_eq!(error.path, "/topicmap/topics");
    }

    #[test]
    fn topicmap_rejects_duplicate_labels_across_topics_and_islands() {
        let duplicate = TOPICMAP.replace(r#""label": "WASM""#, r#""label": "Management""#);
        let error =
            render_json(&duplicate, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "duplicate_topic_label");
        assert_eq!(error.path, "/topicmap/islands/0/label");
    }

    #[test]
    fn topicmap_rejects_links_to_unknown_labels() {
        let broken = TOPICMAP.replace(r#""to": "Engineering""#, r#""to": "Nonexistent""#);
        let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "unknown_topic_link");
        assert_eq!(error.path, "/topicmap/links/0/to");
    }

    #[test]
    fn topicmap_rejects_link_weight_outside_zero_to_one() {
        let broken = TOPICMAP.replace(r#""weight": 0.6"#, r#""weight": 1.5"#);
        let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "invalid_link_weight");
        assert_eq!(error.path, "/topicmap/links/0/weight");
    }

    #[test]
    fn topicmap_rejects_a_value_below_one() {
        let broken = TOPICMAP.replace(r#""value": 69"#, r#""value": 0"#);
        let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!(error.code, "topic_value_out_of_range");
        assert_eq!(error.path, "/topicmap/topics/4/value");
    }

    #[test]
    fn topicmap_warns_when_a_topic_is_too_small_for_its_label() {
        let small = TOPICMAP.replace(r#""value": 69, "points": 5"#, r#""value": 5, "points": 5"#);
        let output = render_json(&small, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(
            output
                .warnings
                .iter()
                .any(|warning| warning.code == "topic_too_small_for_label"
                    && warning.path == "/topicmap/topics/4/value"),
            "{:?}",
            output.warnings
        );
    }

    #[test]
    fn topicmap_rejects_bar_line_and_time_only_fields() {
        for (field, path) in [
            (r#""data": [{"label": "A", "value": 1}],"#, "/data"),
            (r#""categories": ["A"],"#, "/categories"),
            (r#""series": [{"name": "S", "values": [1]}],"#, "/series"),
            (
                r#""zoomSteps": [{"label": "A", "from": 0, "to": 0}, {"label": "B", "from": 0, "to": 1}],"#,
                "/zoomSteps",
            ),
            (r#""panes": [{}],"#, "/panes"),
        ] {
            let mixed = TOPICMAP.replace(
                "\"schemaVersion\": 1,",
                &format!("\"schemaVersion\": 1, {field}"),
            );
            let error = render_json(&mixed, RenderFormat::Svg, &RenderOptions::default())
                .expect_err("a topicmap chart is drawn from topicmap.topics");
            assert_eq!(error.code, "option_not_supported", "{field}");
            assert_eq!(error.path, path, "{field}");
        }
    }

    #[test]
    fn topicmap_html_lists_every_area_with_entries_and_paths() {
        let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(output.content.contains(
            "<th scope=\"col\">Topic</th><th scope=\"col\">Entries</th><th scope=\"col\">Paths</th>"
        ));
        assert!(
            output
                .content
                .contains("<th scope=\"row\">WASM</th><td>6</td><td>1</td>")
        );
    }
}
