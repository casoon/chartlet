mod error;
mod layout;
mod metrics;
mod render;
mod scene;
mod spec;

use std::fmt::Write as _;

pub use error::{ChartError, ChartWarning};
pub use metrics::{BuiltinMetrics, TextMetrics};
use spec::Dataset;
pub use spec::{
    CategoryAxisSpec, ChartSpec, ChartType, DataPoint, Orientation, SeriesSpec, ValueAxisSpec,
    ValueFormat,
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
    let description = spec
        .description
        .clone()
        .unwrap_or_else(|| automatic_description(spec));
    let scene = layout::layout(spec, &mut warnings, metrics);
    let svg = render::svg(&scene, spec, &description, &id_prefix);
    let content = match format {
        RenderFormat::Svg => svg,
        RenderFormat::Html => render::html(&svg, spec, options.table_mode),
    };
    Ok(RenderOutput { content, warnings })
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
    let dataset = spec.dataset();
    let chart = chart_type_name(spec.chart_type);
    let show = |value| layout::format_value(value, spec.value_axis.format);
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
            .filter_map(|series| series.name)
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
            labels_at_value(&dataset, highest_value),
            show(lowest_value),
            labels_at_value(&dataset, lowest_value)
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

fn labels_at_value(dataset: &Dataset<'_>, value: f64) -> String {
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
                .map(move |series| match series.name {
                    Some(name) if grouped => format!("{name} in {category}"),
                    _ => (*category).to_owned(),
                })
        })
        .collect::<Vec<_>>()
        .join(", ")
}

const fn chart_type_name(chart_type: ChartType) -> &'static str {
    match chart_type {
        ChartType::Bar => "Bar chart",
        ChartType::Line => "Line chart",
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderFormat, RenderOptions, render_json};

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
        assert!(output.content.contains("<td>-4</td>"));
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
                .contains("text-anchor=\"end\" class=\"chartlet-value\">-0.5%</text>")
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
}
