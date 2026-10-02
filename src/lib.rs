mod atlas;
mod calendar;
mod color;
mod contour;
mod describe;
mod diverging;
mod error;
mod layout;
mod metrics;
mod noise;
mod ohlc;
#[cfg(feature = "png")]
mod png;
pub mod qr;
mod rangebar;
mod reference;
mod render;
mod scene;
mod sha256;
mod social;
mod spec;
mod stripes;
mod text;
mod time;

use describe::automatic_description;
pub use error::{ChartError, ChartWarning};
pub use metrics::{BuiltinMetrics, TextMetrics};
pub use sha256::sha256;
pub use spec::{
    AxisScale, CalendarDay, CalendarLayout, CalendarSpec, CartoucheSpec, CategoryAxisSpec,
    ChartSpec, ChartType, Corner, Curve, Dash, DataPoint, Gaps, LayerSpec, LegendPlacement, Mark,
    MobileSpec, OhlcPoint, Orientation, PaneSpec, RangeSpec, ReferenceSpec, SeriesSpec, Shape,
    Stack, StripesSpec, Stroke, Theme, TimeAxisKind, TimeAxisSpec, TimePoint, TimePrecision,
    Tooltips, TopicLinkSpec, TopicMapSpec, TopicSpec, ValueAxisSpec, ValueFormat, ZoomBound,
    ZoomStep,
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
    pub variant: Variant,
    /// Also returns a [`Manifest`] of the render in [`RenderOutput::manifest`].
    pub manifest: bool,
    /// Where the chart's styles live. The print variant always carries its own.
    pub styles: Styles,
    /// Adds the `data-*` hooks that the optional interactive module of the npm package reads:
    /// chart type and ID on the root, the plot geometry of every pane, a group around what each
    /// layer draws, and the unformatted values on the data table, or in a JSON data block in
    /// the SVG profile. The print variant carries none.
    pub hooks: bool,
}

/// Where a chart's styles live.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Styles {
    /// Every chart carries its whole stylesheet, scoped to its root: an SVG file stands alone.
    #[default]
    Inline,
    /// The page loads the shared [`stylesheet`] once; a chart carries only what is its own, such
    /// as declared layer colors, and often no `<style>` at all.
    External,
}

/// The stylesheet every chart rendered with [`Styles::External`] relies on. It is the same for
/// every chart, so a site serves it once as a cacheable file.
#[must_use]
pub fn stylesheet() -> String {
    render::shared_stylesheet(&ChartType::ALL, true)
}

/// The shared stylesheet with only the rules of `chart_types`: enough for a site that renders no
/// other types. It is [`stylesheet_common`] followed by [`stylesheet_types`], and the part of
/// each type comes in the order of `chart_types`.
#[must_use]
pub fn stylesheet_for(chart_types: &[ChartType]) -> String {
    render::shared_stylesheet(chart_types, true)
}

/// The part of the shared stylesheet that every chart relies on, whatever its type: colors,
/// text, grid and axes. A page that loads type parts separately loads this part first.
#[must_use]
pub fn stylesheet_common() -> String {
    render::shared_stylesheet(&[], true)
}

/// The parts of the shared stylesheet that only charts of `chart_types` use, without the common
/// part. Each type's rules style only charts of that type, so the parts of separate calls can be
/// loaded as separate files, in any order, after [`stylesheet_common`].
#[must_use]
pub fn stylesheet_types(chart_types: &[ChartType]) -> String {
    render::shared_stylesheet(chart_types, false)
}

/// Which layout the SVG profile renders. The HTML profile always carries the chart and, when the
/// specification has one, its mobile variant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Variant {
    /// The chart at `width` × `height`.
    #[default]
    Desktop,
    /// The chart at the size in `mobile`; its IDs end in `-m`.
    Mobile,
    /// The chart at `width` × `height` for print and PDF pipelines and renderers outside the
    /// browser: its stylesheet carries the theme's colors as literal values instead of CSS
    /// custom properties, and no rule that needs a browser. Its IDs end in `-p`.
    Print,
    /// The chart on a 1200 × 630 canvas for link previews such as Open Graph images: the title
    /// drawn large, the source below, no tooltips, and the resolved stylesheet of
    /// [`Variant::Print`]. Its IDs end in `-s`.
    Social,
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
    /// The provenance of `content`, when [`RenderOptions::manifest`] asks for it.
    pub manifest: Option<Manifest>,
}

/// The provenance of one render: which version of chartlet produced which output from which
/// specification, with which options and warnings. It carries no timestamp, so the same render
/// always yields the same manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The version of the chartlet crate that rendered.
    pub chartlet: &'static str,
    pub schema_version: u8,
    /// `sha256:` and the hexadecimal SHA-256 of the canonical specification: the parsed
    /// specification serialized again, so key order and whitespace of the input do not matter.
    pub spec_hash: String,
    /// `sha256:` and the hexadecimal SHA-256 of [`RenderOutput::content`] as UTF-8.
    pub output_hash: String,
    pub format: RenderFormat,
    pub variant: Variant,
    /// The ID prefix the chart was rendered with: the one passed in, or the one derived from the
    /// specification. The mobile variant appends `-m` to it, the print variant `-p`, the social
    /// variant `-s`.
    pub id_prefix: String,
    pub warnings: Vec<ChartWarning>,
}

impl Manifest {
    /// The manifest as a pretty-printed JSON object, its keys in a fixed order:
    /// `chartlet`, `schemaVersion`, `specHash`, `outputHash`, `format`, `variant`, `idPrefix`,
    /// `warnings` (each with `code`, `path`, `message`).
    ///
    /// # Panics
    ///
    /// Never: every field is a string, a number or a list of strings.
    #[must_use]
    pub fn to_json(&self) -> String {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Json<'a> {
            chartlet: &'a str,
            schema_version: u8,
            spec_hash: &'a str,
            output_hash: &'a str,
            format: &'a str,
            variant: &'a str,
            id_prefix: &'a str,
            warnings: Vec<Warning<'a>>,
        }
        #[derive(serde::Serialize)]
        struct Warning<'a> {
            code: &'a str,
            path: &'a str,
            message: &'a str,
        }
        let json = Json {
            chartlet: self.chartlet,
            schema_version: self.schema_version,
            spec_hash: &self.spec_hash,
            output_hash: &self.output_hash,
            format: match self.format {
                RenderFormat::Svg => "svg",
                RenderFormat::Html => "html",
            },
            variant: match self.variant {
                Variant::Desktop => "desktop",
                Variant::Mobile => "mobile",
                Variant::Print => "print",
                Variant::Social => "social",
            },
            id_prefix: &self.id_prefix,
            warnings: self
                .warnings
                .iter()
                .map(|warning| Warning {
                    code: warning.code,
                    path: &warning.path,
                    message: &warning.message,
                })
                .collect(),
        };
        serde_json::to_string_pretty(&json).expect("a manifest always serializes")
    }
}

/// What a chart says without its graphic: the description its SVG carries and the data table of
/// its HTML profile, as text, for hosts that build their own accessible wrapper around the SVG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAlternative {
    pub description: String,
    pub table: DataTable,
}

/// A chart's data table as text, values written as the chart writes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTable {
    pub caption: String,
    /// The column heads; the first names the categories.
    pub columns: Vec<String>,
    /// One row per category, starting with the category.
    pub rows: Vec<Vec<String>>,
}

/// Validates a chart specification and returns its text alternative.
///
/// # Errors
///
/// Returns a structured error when the specification is invalid.
pub fn text_alternative(spec: &ChartSpec) -> Result<TextAlternative, ChartError> {
    spec.validate()?;
    Ok(TextAlternative {
        description: spec
            .description
            .clone()
            .unwrap_or_else(|| automatic_description(spec)),
        table: render::data_table(spec),
    })
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
    let mut output = render_content(spec, format, options, metrics)?;
    if options.manifest {
        let id_prefix = match &options.id_prefix {
            Some(id_prefix) => id_prefix.clone(),
            None => default_id_prefix(spec)?,
        };
        output.manifest = Some(Manifest {
            chartlet: env!("CARGO_PKG_VERSION"),
            schema_version: spec.schema_version,
            spec_hash: format!("sha256:{}", sha256::hex(&sha256(&canonical_json(spec)?))),
            output_hash: format!("sha256:{}", sha256::hex(&sha256(output.content.as_bytes()))),
            format,
            variant: options.variant,
            id_prefix,
            warnings: output.warnings.clone(),
        });
    }
    Ok(output)
}

fn render_content(
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

    let mobile = spec.mobile_variant();
    let mobile_prefix = |prefix: &str| format!("{prefix}-m");
    let hooks = hook_mode(options.hooks, format);
    let panel = |spec: &ChartSpec, prefix: &str, warnings: &mut Vec<ChartWarning>| {
        render_panel(
            spec,
            format,
            prefix,
            options.styles.into(),
            hooks,
            metrics,
            warnings,
        )
    };

    match options.variant {
        Variant::Desktop => {}
        Variant::Mobile => {
            return render_mobile_alone(
                mobile,
                format,
                &mobile_prefix(&id_prefix),
                options.styles,
                hooks,
                metrics,
            );
        }
        Variant::Print => return render_print(spec, format, &id_prefix, warnings, metrics),
        Variant::Social => return render_social(spec, format, &id_prefix, metrics),
    }

    // The HTML profile lays the chart out a second time at the mobile size; its warnings are
    // reported where they add to those of the chart itself.
    let mut mobile_warnings = match (&mobile, format) {
        (Some(mobile), RenderFormat::Html) => mobile.validate_mobile_variant()?,
        _ => Vec::new(),
    };

    // Zoom steps render as pre-computed variants switched by radio buttons, which only the
    // HTML profile can carry. The pure SVG profile stays a single static chart.
    if format == RenderFormat::Html && spec.zoom_steps.len() > 1 {
        let mut panels = Vec::new();
        for (index, step) in spec.zoom_steps.iter().enumerate() {
            let sliced = zoom_variant(spec, step);
            let panel_prefix = format!("{id_prefix}-z{index}");
            let svg = panel(&sliced, &panel_prefix, &mut warnings);
            let mobile = mobile.as_ref().map(|mobile| {
                panel(
                    &zoom_variant(mobile, step),
                    &mobile_prefix(&panel_prefix),
                    &mut mobile_warnings,
                )
            });
            panels.push(render::Panel {
                label: step.label.clone(),
                svg,
                mobile,
            });
        }
        dedupe_warnings(&mut warnings);
        merge_mobile_warnings(&mut warnings, mobile_warnings);
        return Ok(RenderOutput {
            content: render::html_zoom(
                &panels,
                spec,
                options.table_mode,
                &id_prefix,
                options.hooks,
            ),
            warnings,
            manifest: None,
        });
    }

    let svg = panel(spec, &id_prefix, &mut warnings);
    let content = match format {
        RenderFormat::Svg => svg,
        RenderFormat::Html => {
            let mobile = mobile
                .as_ref()
                .map(|mobile| panel(mobile, &mobile_prefix(&id_prefix), &mut mobile_warnings));
            merge_mobile_warnings(&mut warnings, mobile_warnings);
            render::html(
                render::Panel {
                    label: String::new(),
                    svg,
                    mobile,
                },
                spec,
                options.table_mode,
                &id_prefix,
                options.hooks,
            )
        }
    };
    Ok(RenderOutput {
        content,
        warnings,
        manifest: None,
    })
}

/// The chart one zoom step shows: a window of time on a time chart, a range of categories on
/// any other.
fn zoom_variant(spec: &ChartSpec, step: &ZoomStep) -> ChartSpec {
    if spec.chart_type == ChartType::Time {
        let zone = spec.time_zone().unwrap_or_default();
        let bound = |bound: &ZoomBound| {
            bound
                .resolve(zone)
                .expect("validated zoom steps resolve to timestamps")
        };
        spec.windowed(bound(&step.from), bound(&step.to))
    } else {
        let index = |bound: &ZoomBound| {
            bound
                .index()
                .expect("validated zoom steps of a category chart are indices")
        };
        spec.sliced(index(&step.from), index(&step.to))
    }
}

/// The mobile variant alone, as SVG, with its IDs under `prefix`.
fn render_mobile_alone(
    mobile: Option<ChartSpec>,
    format: RenderFormat,
    prefix: &str,
    styles: Styles,
    hooks: render::Hooks,
    metrics: &impl TextMetrics,
) -> Result<RenderOutput, ChartError> {
    if format == RenderFormat::Html {
        return Err(ChartError::new(
            "option_not_supported",
            "/render/variant",
            "the HTML profile carries both variants; render the mobile variant alone as SVG",
        ));
    }
    let Some(mobile) = mobile else {
        return Err(ChartError::new(
            "missing_mobile",
            "/mobile",
            "the specification has no mobile variant; add \"mobile\": { \"width\": 360 }",
        ));
    };
    let mut warnings = mobile.validate_mobile_variant()?;
    let content = render_panel(
        &mobile,
        format,
        prefix,
        styles.into(),
        hooks,
        metrics,
        &mut warnings,
    );
    Ok(RenderOutput {
        content,
        warnings,
        manifest: None,
    })
}

/// The print variant, as SVG, with its IDs ending in `-p`.
fn render_print(
    spec: &ChartSpec,
    format: RenderFormat,
    id_prefix: &str,
    mut warnings: Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Result<RenderOutput, ChartError> {
    if format == RenderFormat::Html {
        return Err(ChartError::new(
            "option_not_supported",
            "/render/variant",
            "the print variant is a standalone SVG; render it with the SVG format",
        ));
    }
    let content = render_panel(
        spec,
        format,
        &format!("{id_prefix}-p"),
        render::StyleMode::Print,
        render::Hooks::Off,
        metrics,
        &mut warnings,
    );
    warn_unresolved_colors(spec, "print", &mut warnings);
    Ok(RenderOutput {
        content,
        warnings,
        manifest: None,
    })
}

/// The social variant, as SVG, with its IDs ending in `-s`. The chart is laid out without its
/// title at the size the canvas leaves it, and validated at that size like a mobile variant.
fn render_social(
    spec: &ChartSpec,
    format: RenderFormat,
    id_prefix: &str,
    metrics: &impl TextMetrics,
) -> Result<RenderOutput, ChartError> {
    if format == RenderFormat::Html {
        return Err(ChartError::new(
            "option_not_supported",
            "/render/variant",
            "the social variant is a standalone SVG; render it with the SVG format",
        ));
    }
    let mut frame_warnings = Vec::new();
    let frame = social::Frame::new(spec, metrics, &mut frame_warnings);
    let chart = ChartSpec {
        width: frame.chart_width,
        height: frame.chart_height,
        show_title: false,
        mobile: None,
        ..spec.clone()
    };
    let mut warnings = chart.validate_mobile_variant()?;
    warnings.append(&mut frame_warnings);
    let content = render_panel(
        &chart,
        format,
        &format!("{id_prefix}-s"),
        render::StyleMode::Print,
        render::Hooks::Off,
        metrics,
        &mut warnings,
    );
    warn_unresolved_colors(spec, "social", &mut warnings);
    Ok(RenderOutput {
        content: frame.compose(&content),
        warnings,
        manifest: None,
    })
}

/// A variant with a resolved stylesheet cannot see the page that would define a `var()` color, so
/// such a layer is drawn in the text color; several of them would become indistinguishable.
fn warn_unresolved_colors(spec: &ChartSpec, variant: &str, warnings: &mut Vec<ChartWarning>) {
    for entry in spec.indexed_layers() {
        if entry
            .layer
            .resolved_color()
            .is_some_and(color::lacks_fallback)
        {
            warnings.push(ChartWarning::new(
                "color_not_resolved",
                format!("/panes/{}/layers/{}/color", entry.pane, entry.local),
                format!("the {variant} variant cannot resolve a var() color and draws this layer in the text color; give it a fallback such as var(--name, #2563eb)"),
            ));
        }
    }
}

/// Options of [`render_png`].
#[cfg(feature = "png")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PngOptions {
    /// [`Variant::Print`] rasterizes the chart at `width` × `height`, [`Variant::Social`] the
    /// 1200 × 630 canvas. [`Variant::Desktop`] is taken as [`Variant::Print`]: a PNG needs the
    /// resolved stylesheet. [`Variant::Mobile`] is not supported.
    pub variant: Variant,
    /// Image pixels per SVG pixel, 0.25–4; 2 for a high-density screen.
    pub scale: f32,
}

#[cfg(feature = "png")]
impl Default for PngOptions {
    fn default() -> Self {
        Self {
            variant: Variant::Print,
            scale: 1.0,
        }
    }
}

/// A rendered PNG and the warnings of the SVG it was rasterized from.
#[cfg(feature = "png")]
#[derive(Debug, Clone)]
pub struct PngOutput {
    pub png: Vec<u8>,
    pub warnings: Vec<ChartWarning>,
}

/// Validates a chart specification and renders it as PNG: the print or the social variant,
/// rasterized with the bundled Inter font and no system fonts. The same specification, options
/// and chartlet version yield the same bytes.
///
/// # Errors
///
/// Returns a structured error when the specification is invalid, the scale is out of range, or
/// the variant is the mobile one.
#[cfg(feature = "png")]
pub fn render_png(spec: &ChartSpec, options: &PngOptions) -> Result<PngOutput, ChartError> {
    if !(0.25..=png::MAX_SCALE).contains(&options.scale) {
        return Err(ChartError::new(
            "invalid_scale",
            "/render/scale",
            "scale must be between 0.25 and 4",
        ));
    }
    let variant = match options.variant {
        Variant::Desktop | Variant::Print => Variant::Print,
        Variant::Social => Variant::Social,
        Variant::Mobile => {
            return Err(ChartError::new(
                "option_not_supported",
                "/render/variant",
                "a PNG is rendered from the print or the social variant",
            ));
        }
    };
    let output = render(
        spec,
        RenderFormat::Svg,
        &RenderOptions {
            variant,
            ..RenderOptions::default()
        },
    )?;
    Ok(PngOutput {
        png: png::rasterize(&output.content, options.scale)?,
        warnings: output.warnings,
    })
}

/// Lays out and serializes one chart, using its explicit description or a generated one. The HTML
/// profile captions the chart, so its SVGs leave the drawn title out (`format`); the title stays
/// their accessible name either way. `print` draws it for print (see [`Variant::Print`]).
fn render_panel(
    spec: &ChartSpec,
    format: RenderFormat,
    id_prefix: &str,
    styles: render::StyleMode,
    hooks: render::Hooks,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> String {
    let description = spec
        .description
        .clone()
        .unwrap_or_else(|| automatic_description(spec));
    let scene = if format == RenderFormat::Html && spec.show_title {
        let untitled = ChartSpec {
            show_title: false,
            ..spec.clone()
        };
        layout::layout(&untitled, warnings, metrics)
    } else {
        layout::layout(spec, warnings, metrics)
    };
    render::svg(&scene, spec, &description, id_prefix, styles, hooks)
}

/// Which hooks a chart's SVGs carry: the HTML profile has its data table to read the values
/// from, the SVG profile carries them itself.
const fn hook_mode(hooks: bool, format: RenderFormat) -> render::Hooks {
    match (hooks, format) {
        (false, _) => render::Hooks::Off,
        (true, RenderFormat::Html) => render::Hooks::Attributes,
        (true, RenderFormat::Svg) => render::Hooks::WithData,
    }
}

/// Zoom panels repeat the same data, so identical warnings would otherwise appear once per panel.
fn dedupe_warnings(warnings: &mut Vec<ChartWarning>) {
    let mut seen = std::collections::BTreeSet::new();
    warnings.retain(|warning| seen.insert((warning.code, warning.path.clone())));
}

/// Adds the warnings of the mobile variant that the chart itself does not already report, marked
/// as coming from the mobile variant.
fn merge_mobile_warnings(warnings: &mut Vec<ChartWarning>, mut mobile: Vec<ChartWarning>) {
    dedupe_warnings(&mut mobile);
    let reported = warnings
        .iter()
        .map(|warning| (warning.code, warning.path.clone()))
        .collect::<std::collections::BTreeSet<_>>();
    warnings.extend(
        mobile
            .into_iter()
            .filter(|warning| !reported.contains(&(warning.code, warning.path.clone())))
            .map(|warning| ChartWarning {
                message: format!("mobile variant: {}", warning.message),
                ..warning
            }),
    );
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

/// The canonical form of a specification: the parsed specification serialized again, so that two
/// inputs that differ only in key order or whitespace share it.
fn canonical_json(spec: &ChartSpec) -> Result<Vec<u8>, ChartError> {
    serde_json::to_vec(spec).map_err(|error| {
        ChartError::new(
            "serialization_failed",
            "/",
            format!("could not canonicalize the chart specification: {error}"),
        )
    })
}

fn default_id_prefix(spec: &ChartSpec) -> Result<String, ChartError> {
    let hash = canonical_json(spec)?
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    Ok(format!("chartlet-{hash:016x}"))
}

#[cfg(test)]
mod tests {
    use super::{
        BuiltinMetrics, ChartSpec, RenderFormat, RenderOptions, TextMetrics, Variant, render_json,
        render_with_metrics, text_alternative,
    };

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
    fn html_captions_the_title_instead_of_drawing_it() {
        let svg = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        let html = render_json(SPEC, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            svg.content
                .contains("class=\"chartlet-title\">Profit &amp; loss")
        );
        assert!(!html.content.contains("class=\"chartlet-title\""));
        assert!(
            html.content
                .contains("<figcaption>Profit &amp; loss</figcaption>")
        );
        assert!(html.content.contains("-title\">Profit &amp; loss</title>"));
        let ids = |content: &str| {
            content
                .split("id=\"")
                .skip(1)
                .filter_map(|rest| rest.split('"').next())
                .filter(|id| id.starts_with("chartlet-"))
                .map(str::to_owned)
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert!(ids(&svg.content).is_subset(&ids(&html.content)));
    }

    #[test]
    fn show_title_false_leaves_the_title_out_of_any_svg() {
        let spec = SPEC.replacen(
            "\"type\": \"bar\",",
            "\"type\": \"bar\", \"showTitle\": false,",
            1,
        );
        let output = render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(!output.content.contains("class=\"chartlet-title\""));
        assert!(
            output
                .content
                .contains("-title\">Profit &amp; loss</title>")
        );
    }

    #[test]
    fn html_does_not_shorten_the_title() {
        let title = "A very long title ".repeat(10);
        let spec = SPEC.replacen("Profit & loss", title.trim(), 1);
        let truncated = |format| {
            render_json(&spec, format, &RenderOptions::default())
                .unwrap()
                .warnings
                .iter()
                .any(|warning| warning.code == "text_truncated" && warning.path == "/title")
        };
        assert!(truncated(RenderFormat::Svg));
        assert!(!truncated(RenderFormat::Html));
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
        assert!(
            output
                .content
                .contains("<div class=\"chartlet-data\" data-viz-text>")
        );
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
    fn line_charts_draw_several_series_apart_by_color_and_pattern() {
        let line = GROUPED.replace("\"bar\"", "\"line\"");
        let svg = render_ok(&line).content;
        assert!(svg.contains("chartlet-line chartlet-line-series-1\""));
        assert!(svg.contains("chartlet-line chartlet-line-series-2 chartlet-line-dashed"));
        assert!(svg.contains("class=\"chartlet-legend\""));
        assert!(!svg.contains("class=\"chartlet-value\""));
    }

    #[test]
    fn patterns_outline_every_other_series() {
        let spec = GROUPED.replacen('{', "{\"patterns\": true,", 1);
        let svg = render_ok(&spec).content;
        assert!(svg.contains("chartlet-bar chartlet-series-2 chartlet-outline"));
        assert!(svg.contains(".chartlet-outline{fill:var(--chartlet-background)"));
        let line = spec.replace("\"bar\"", "\"line\"");
        assert_eq!(
            render_err(&line),
            ("option_not_supported", "/patterns".to_owned())
        );
    }

    #[test]
    fn stacks_add_up_and_a_percent_stack_takes_no_negative_values() {
        let stacked = GROUPED.replacen('{', "{\"stack\": \"normal\",", 1);
        let svg = render_ok(&stacked).content;
        assert!(
            svg.contains("The series are stacked. Highest total:"),
            "{svg}"
        );
        let percent = GROUPED.replacen('{', "{\"stack\": \"percent\",", 1);
        assert!(render_ok(&percent).content.contains("shares of the series"));
        assert_eq!(
            render_err(&percent.replace("120", "-120")),
            ("invalid_value", "/series/0/values/0".to_owned())
        );
        let single = SPEC.replacen('{', "{\"stack\": \"normal\",", 1);
        assert_eq!(
            render_err(&single),
            ("option_not_supported", "/stack".to_owned())
        );
    }

    #[test]
    fn stretched_stripes_fill_the_canvas_and_stretch() {
        let spec = r#"{"schemaVersion": 1, "type": "stripes", "title": "Band",
            "stripes": {"firstYear": 2000, "values": [0.1, -0.2, 0.3, 0.5], "stretch": true}}"#;
        let svg = render_ok(spec).content;
        assert!(svg.contains(" preserveAspectRatio=\"none\""));
        assert!(svg.contains("<rect x=\"0\" y=\"0\""), "{svg}");
        assert!(!svg.contains("class=\"chartlet-title\""));
        assert!(!svg.contains("class=\"chartlet-tick\""));
    }

    #[test]
    fn tooltips_markers_leaves_a_dense_line_without_targets() {
        let points: Vec<String> = (0..120)
            .map(|minute| {
                format!(
                    r#"{{"time": "2026-03-01T{:02}:{:02}:00Z", "value": {}}}"#,
                    minute / 60,
                    minute % 60,
                    minute % 7
                )
            })
            .collect();
        let spec = format!(
            r#"{{"schemaVersion": 1, "type": "time", "title": "Visits", "panes": [{{"layers":
            [{{"mark": "line", "name": "Visits", "points": [{}]}}]}}]}}"#,
            points.join(",")
        );
        let every = render_ok(&spec).content;
        assert_eq!(every.matches("class=\"chartlet-hit\"").count(), 120);
        let markers = spec.replacen('{', "{\"tooltips\": \"markers\",", 1);
        let svg = render_ok(&markers).content;
        assert!(!svg.contains("class=\"chartlet-hit\""));
        assert!(svg.len() * 2 < every.len());
        let bar = SPEC.replacen('{', "{\"tooltips\": \"markers\",", 1);
        assert_eq!(
            render_err(&bar),
            ("option_not_supported", "/tooltips".to_owned())
        );
    }

    #[test]
    fn a_point_layer_draws_dots_without_a_line() {
        let spec = include_str!("../examples/resting-heart-rate.json");
        let svg = render_ok(spec).content;
        // Only the weekly mean is a line; the readings are dots, one in the legend as well.
        assert_eq!(svg.matches("<polyline").count(), 2, "{svg}");
        assert_eq!(
            svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
                .count(),
            28
        );
        let dashed = spec.replacen(
            "\"mark\": \"point\",",
            "\"mark\": \"point\", \"dash\": \"dotted\",",
            1,
        );
        assert_eq!(
            render_err(&dashed),
            ("option_not_supported", "/panes/0/layers/0/dash".to_owned())
        );
    }

    #[test]
    fn a_horizontal_log_axis_writes_only_labels_that_fit_and_keeps_its_end_label() {
        let spec = r#"{"schemaVersion":1,"type":"rangebar","orientation":"horizontal","title":"t",
            "width":480,"height":240,"valueAxis":{"scale":"log"},"ranges":[
            {"label":"A","low":1000000,"high":10000000},{"label":"B","low":10,"high":300}]}"#;
        let svg = render_ok(spec).content;
        // Seven gridlines from 10 to 10,000,000, but only every other tick labelled.
        assert_eq!(svg.matches("class=\"chartlet-grid\"").count(), 7, "{svg}");
        assert_eq!(svg.matches("class=\"chartlet-tick\"").count(), 4, "{svg}");
        let end = svg.find(">1000000 to 10000000<").expect("the end label");
        let x: f64 = svg[..end]
            .rsplit("<text x=\"")
            .next()
            .and_then(|tail| tail.split('"').next())
            .and_then(|x| x.parse().ok())
            .expect("the label's x");
        assert!(
            x + crate::metrics::TextMetrics::width(
                &crate::metrics::BuiltinMetrics,
                "1000000 to 10000000",
                12.0
            ) <= 480.0,
            "{svg}"
        );
    }

    #[test]
    fn every_layer_writes_its_times_as_finely_as_it_needs() {
        // Monthly values beside yearly means placed mid-year: the means say only the year.
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "CO2", "panes": [{"layers": [
            {"mark": "line", "name": "Monthly", "points": [{"time": "2020-01", "value": 1},
                {"time": "2020-02", "value": 2}, {"time": "2020-03", "value": 3}]},
            {"mark": "line", "name": "Yearly", "precision": "year", "points": [
                {"time": "2019-07-01", "value": 1}, {"time": "2020-07-01", "value": 2}]}]}]}"#;
        let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
            .expect("renders")
            .content;
        assert!(html.contains("<title>2020 – Yearly: 2</title>"), "{html}");
        assert!(
            html.contains("<title>2020-02 – Monthly: 2</title>"),
            "{html}"
        );
        assert!(html.contains("<th scope=\"row\">2019</th>"), "{html}");

        // On a numeric axis, a finely spaced helper line does not add decimals to the points.
        let numeric = r#"{"schemaVersion": 1, "type": "time", "title": "Quakes",
            "timeAxis": {"kind": "number"}, "panes": [{"layers": [
            {"mark": "point", "name": "Counted", "points": [{"time": 4.5, "value": 3},
                {"time": 5, "value": 2}]},
            {"mark": "line", "name": "Fit", "points": [{"time": 4.45, "value": 3},
                {"time": 5.05, "value": 2}]}]}]}"#;
        let svg = render_ok(numeric).content;
        assert!(svg.contains("<title>4.5 – Counted: 3</title>"), "{svg}");
        assert!(svg.contains("<title>4.45 – Fit: 3</title>"), "{svg}");
        let on_numeric = numeric.replacen(
            "\"name\": \"Fit\",",
            "\"name\": \"Fit\", \"precision\": \"year\",",
            1,
        );
        assert_eq!(
            render_err(&on_numeric),
            (
                "option_not_supported",
                "/panes/0/layers/1/precision".to_owned()
            )
        );
    }

    #[test]
    fn a_line_without_markers_keeps_its_tooltips() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Fit", "showValues": false,
            "panes": [{"layers": [{"mark": "line", "name": "Fit", "markers": false, "points": [
            {"time": "2020", "value": 1}, {"time": "2021", "value": 2}]}]}]}"#;
        let svg = render_ok(spec).content;
        assert!(!svg.contains("class=\"chartlet-point"), "{svg}");
        assert_eq!(svg.matches("class=\"chartlet-hit\"").count(), 2, "{svg}");
        let point = spec.replace("\"mark\": \"line\"", "\"mark\": \"point\"");
        assert_eq!(
            render_err(&point),
            (
                "option_not_supported",
                "/panes/0/layers/0/markers".to_owned()
            )
        );
    }

    #[test]
    fn a_declared_precision_names_times_but_keeps_the_ticks() {
        let points: Vec<String> = (1989..=2022)
            .map(|year| format!(r#"{{"time": "{year}-07-01", "value": {}}}"#, year % 7))
            .collect();
        let spec = format!(
            r#"{{"schemaVersion": 1, "type": "time", "title": "pH", "width": 720, "height": 340,
            "panes": [{{"layers": [{{"mark": "line", "name": "pH", "points": [{}]}}]}}]}}"#,
            points.join(",")
        );
        let declared = spec.replacen('{', "{\"timeAxis\": {\"precision\": \"year\"},", 1);
        let ticks = |svg: &str| svg.matches("class=\"chartlet-tick\"").count();
        assert_eq!(
            ticks(&render_ok(&declared).content),
            ticks(&render_ok(&spec).content)
        );
    }

    #[test]
    fn the_last_step_holds_until_its_end_without_an_observation_there() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Annual means",
            "showValues": false, "panes": [{"layers": [{"mark": "line", "name": "Mean",
            "curve": "step", "stepEnd": "2026", "points": [{"time": "2024", "value": 1},
            {"time": "2025", "value": 2}]}]}]}"#;
        let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
            .expect("renders")
            .content;
        // Two observations, two tooltips and two rows; the line runs on to the end of 2025.
        assert_eq!(html.matches("<title>20").count(), 2, "{html}");
        assert!(!html.contains("<th scope=\"row\">2026</th>"), "{html}");
        let line = html
            .split("<polyline points=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the line");
        assert_eq!(line.split(' ').count(), 4, "{line}");

        let linear = spec.replace("\"curve\": \"step\", ", "");
        assert_eq!(
            render_err(&linear),
            (
                "option_not_supported",
                "/panes/0/layers/0/stepEnd".to_owned()
            )
        );
        let early = spec.replace("\"stepEnd\": \"2026\"", "\"stepEnd\": \"2025\"");
        assert_eq!(
            render_err(&early),
            ("unordered_time", "/panes/0/layers/0/stepEnd".to_owned())
        );
    }

    #[test]
    fn a_compact_chart_keeps_room_for_its_plot() {
        let spec = SPEC.replacen('{', "{\"width\": 240,", 1);
        let svg = render_ok(&spec).content;
        // The plot starts after the compact gutter of 56 pixels.
        assert!(svg.contains("<line x1=\"56\""), "{svg}");
        let too_narrow = SPEC.replacen('{', "{\"width\": 199,", 1);
        assert_eq!(
            render_err(&too_narrow),
            ("invalid_dimension", "/width".to_owned())
        );
    }

    #[test]
    fn range_groups_take_palette_colors_and_a_legend() {
        let spec = include_str!("../examples/soil-animals.json");
        let svg = render_ok(spec).content;
        // Three groups: a span in each color, and a swatch per group in the legend.
        for (series, spans) in [(1, 2), (2, 3), (3, 3)] {
            assert_eq!(
                svg.matches(&format!(
                    "class=\"chartlet-range chartlet-range-series-{series}\""
                ))
                .count(),
                spans + 1,
                "{svg}"
            );
        }
        assert!(svg.contains(">Mesofauna</text>"));
        let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
            .expect("renders")
            .content;
        assert!(html.contains("Nematodes (Microfauna)"), "{html}");
        let partial = spec.replacen(", \"group\": \"Microfauna\"", "", 1);
        assert_eq!(
            render_err(&partial),
            ("missing_group", "/ranges/0/group".to_owned())
        );
        let five =
            spec.replacen("\"Microfauna\"", "\"A\"", 1)
                .replacen("\"Mesofauna\"", "\"B\"", 1);
        assert_eq!(
            render_err(&five),
            ("too_many_series", "/ranges/5/group".to_owned())
        );
    }

    #[test]
    fn stacked_areas_rest_on_each_other() {
        let spec = include_str!("../examples/generation-mix.json");
        let options = RenderOptions {
            hooks: true,
            ..RenderOptions::default()
        };
        let output = render_json(spec, RenderFormat::Html, &options).expect("renders");
        // The scale reaches the total of the highest stack, 2,920 TWh in 2016.
        assert!(
            output.content.contains(">3,000</text>"),
            "{}",
            output.content
        );
        assert_eq!(output.content.matches(" data-stacked=\"\"").count(), 4);
        assert!(
            output
                .content
                .contains("Stacked areas, from the bottom up: Fossil, Nuclear")
        );
        // A tooltip tells the layer's own value, not the top of the stack.
        assert!(
            output.content.contains("Wind and solar: 760"),
            "{}",
            output.content
        );

        // A stacked pane takes eight areas; beyond the palette they bring their own colors.
        let mut extra = String::new();
        for (index, color) in ["#0a0", "#a0a", "#aa0", "#0aa"].iter().enumerate() {
            let points = [2000, 2004, 2008, 2012, 2016, 2020, 2024]
                .map(|year| format!(r#"{{"time": "{year}", "value": 10}}"#))
                .join(",");
            std::fmt::Write::write_fmt(
                &mut extra,
                format_args!(r#",{{"mark": "area", "name": "Extra {index}", "color": "{color}", "points": [{points}]}}"#),
            )
            .expect("writing to String cannot fail");
        }
        let eight = spec.replacen(
            "\n      ]\n    }\n  ]",
            &format!("{extra}\n      ]\n    }}\n  ]"),
            1,
        );
        assert_eq!(
            render_ok(&eight)
                .content
                .matches("class=\"chartlet-legend\"")
                .count(),
            8
        );
        let nine = eight.replacen(",{\"mark\": \"area\", \"name\": \"Extra 0\"", ",{\"mark\": \"area\", \"name\": \"Extra 9\", \"color\": \"#123\", \"points\": [{\"time\": \"2000\", \"value\": 1}, {\"time\": \"2024\", \"value\": 1}]},{\"mark\": \"area\", \"name\": \"Extra 0\"", 1);
        assert_eq!(
            render_err(&nine),
            ("too_many_layers", "/panes/0/layers".to_owned())
        );

        let percent = spec.replace("\"stack\": \"normal\"", "\"stack\": \"percent\"");
        assert_eq!(
            render_err(&percent),
            ("option_not_supported", "/panes/0/stack".to_owned())
        );
        let gap = spec.replacen("\"value\": 1520", "\"value\": null", 1);
        assert_eq!(
            render_err(&gap),
            (
                "unaligned_stack",
                "/panes/0/layers/0/points/1/value".to_owned()
            )
        );
        let negative = spec.replacen("\"value\": 1520", "\"value\": -1", 1);
        assert_eq!(
            render_err(&negative),
            (
                "negative_in_stack",
                "/panes/0/layers/0/points/1/value".to_owned()
            )
        );
        let shifted = spec.replacen(
            "\"time\": \"2024\", \"value\": 620",
            "\"time\": \"2023\", \"value\": 620",
            1,
        );
        assert_eq!(
            render_err(&shifted),
            ("unaligned_stack", "/panes/0/layers/1/points".to_owned())
        );
    }

    #[test]
    fn a_sparkline_draws_only_its_line_at_a_small_size() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Visitors", "sparkline": true,
            "width": 120, "height": 32, "panes": [{"layers": [{"mark": "line", "name": "Visitors",
            "points": [{"time": "2026-03-01", "value": 3}, {"time": "2026-03-02", "value": 5},
                       {"time": "2026-03-03", "value": 4}]}]}]}"#;
        let svg = render_ok(spec).content;
        assert!(svg.contains("width=\"120\" height=\"32\""));
        assert!(svg.contains("<title id="));
        assert!(!svg.contains("class=\"chartlet-tick\""));
        assert!(!svg.contains("class=\"chartlet-grid\""));
        // One visible dot where the line ends; every observation keeps an invisible tooltip.
        assert_eq!(
            svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
                .count(),
            1,
            "{svg}"
        );
        assert_eq!(svg.matches("class=\"chartlet-hit\"").count(), 3, "{svg}");
        let wide = spec.replace("\"width\": 120", "\"width\": 800");
        assert_eq!(
            render_err(&wide),
            ("invalid_dimension", "/width".to_owned())
        );
        let bar = SPEC.replacen('{', "{\"sparkline\": true,", 1);
        assert_eq!(render_err(&bar).0, "option_not_supported");
    }

    #[test]
    fn names_at_the_end_of_the_lines_replace_the_legend() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Two lines", "legend": "end",
            "panes": [{"layers": [
            {"mark": "line", "name": "North", "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 3}]},
            {"mark": "line", "name": "South", "points": [{"time": "2020", "value": 2}, {"time": "2021", "value": 3}]}]}]}"#;
        let svg = render_ok(spec).content;
        let names: Vec<&str> = svg
            .split("<text ")
            .filter(|text| text.contains("class=\"chartlet-legend\""))
            .collect();
        assert_eq!(names.len(), 2, "{svg}");
        // Both end at 3, so the names move apart.
        let ys: Vec<f64> = names
            .iter()
            .filter_map(|text| text.split("y=\"").nth(1)?.split('"').next()?.parse().ok())
            .collect();
        assert!((ys[0] - ys[1]).abs() >= 14.0, "{ys:?}");
        let bar = SPEC.replacen('{', "{\"legend\": \"end\",", 1);
        assert_eq!(
            render_err(&bar),
            ("option_not_supported", "/legend".to_owned())
        );
    }

    #[test]
    fn an_exact_axis_ends_where_it_says_and_ticks_at_multiples_of_its_step() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Forcing", "panes": [{
            "valueAxis": {"min": -2.5, "max": 3.5, "exact": true, "step": 1},
            "layers": [{"mark": "line", "name": "V", "points": [
                {"time": "2020", "value": -1}, {"time": "2021", "value": 2.2}]}]}]}"#;
        let svg = render_ok(spec).content;
        let ticks: Vec<&str> = svg
            .split("class=\"chartlet-tick\">")
            .skip(1)
            .filter_map(|rest| rest.split('<').next())
            // The time axis writes years; the value ticks are the short ones.
            .filter(|tick| tick.chars().count() <= 2)
            .collect();
        // The ends stay at −2.5 and 3.5; the ticks sit on multiples of the step, zero included.
        assert_eq!(ticks, ["\u{2212}2", "\u{2212}1", "0", "1", "2", "3"]);
        assert!(svg.contains("class=\"chartlet-zero\""));
        let outside = spec.replace("\"value\": 2.2", "\"value\": 4");
        assert_eq!(
            render_err(&outside),
            (
                "value_outside_axis",
                "/panes/0/layers/0/points/1/value".to_owned()
            )
        );
        let half = spec.replace("\"max\": 3.5, ", "");
        assert_eq!(render_err(&half).0, "invalid_axis_range");
        let bars = SPEC.replacen(
            '{',
            "{\"valueAxis\": {\"min\": 5, \"max\": 20, \"exact\": true},",
            1,
        );
        assert_eq!(render_err(&bars).0, "invalid_axis_range");
    }

    #[test]
    fn a_dense_axis_labels_every_few_categories() {
        let data: Vec<String> = (0..43)
            .map(|index| format!("{{\"label\": \"Country {index}\", \"value\": {index}}}"))
            .collect();
        let spec = format!(
            "{{\"schemaVersion\": 1, \"type\": \"bar\", \"title\": \"Dense\", \"width\": 720, \"data\": [{}]}}",
            data.join(",")
        );
        let output = render_ok(&spec);
        let labels = output.content.matches("class=\"chartlet-label\"").count();
        assert!(labels < 43 && labels > 5, "{labels} labels");
        assert!(
            output
                .warnings
                .iter()
                .any(|warning| warning.code == "labels_thinned")
        );
        assert!(
            !output
                .warnings
                .iter()
                .any(|warning| warning.code == "text_truncated")
        );
    }

    #[test]
    fn months_read_as_months_and_the_precision_can_be_set() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Monthly", "panes": [{"layers": [
            {"mark": "line", "name": "V", "points": [{"time": "2026-01", "value": 1},
             {"time": "2026-02", "value": 2}, {"time": "2026-03", "value": 3}]}]}]}"#;
        let svg = render_ok(spec).content;
        assert!(svg.contains("<title>2026-02 – V: 2</title>"), "{svg}");
        let german = render_ok(&spec.replace(
            "\"type\": \"time\",",
            "\"type\": \"time\", \"locale\": \"de\",",
        ));
        assert!(
            german.content.contains("<title>02.2026 – V: 2</title>"),
            "{}",
            german.content
        );
        let mid_year = r#"{"schemaVersion": 1, "type": "time", "title": "Annual",
            "timeAxis": {"precision": "year"}, "panes": [{"layers": [
            {"mark": "line", "name": "V", "points": [{"time": "1950-07-01", "value": 1},
             {"time": "1951-07-01", "value": 2}]}]}]}"#;
        assert!(
            render_ok(mid_year)
                .content
                .contains("<title>1950 – V: 1</title>")
        );
    }

    #[test]
    fn a_step_line_holds_each_value_until_the_next() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Steps", "panes": [{"layers": [
            {"mark": "line", "name": "Mean", "curve": "step",
             "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 3},
                        {"time": "2022", "value": 2}]}]}]}"#;
        let svg = render_ok(spec).content;
        let points = svg
            .split("<polyline points=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_default();
        // Three observations and a corner before each of the two jumps.
        assert_eq!(points.split(' ').count(), 5, "{points}");
        let annotation = spec.replace(
            r#"{"mark": "line", "name": "Mean", "curve": "step","#,
            r#"{"mark": "annotation", "label": "A", "value": 1, "curve": "step"}, {"mark": "line", "name": "Mean","#,
        );
        assert_eq!(render_err(&annotation).0, "option_not_supported");
    }

    #[test]
    fn a_numeric_axis_reads_plain_numbers_and_can_run_backwards() {
        let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Profile",
            "timeAxis": {"kind": "number", "reverse": true, "title": "Distance (km)"},
            "panes": [{"layers": [{"mark": "line", "name": "Height",
                "points": [{"time": 0, "value": 10}, {"time": 2.5, "value": 30},
                           {"time": 10, "value": 20}]}]}]}"#;
        let html = html_ok(spec);
        assert!(
            html.contains("Line chart with 3 points from 0.0 to 10.0."),
            "{html}"
        );
        assert!(html.contains("<th scope=\"col\">Distance (km)</th>"));
        let svg = render_ok(spec).content;
        // Reversed: the first position, 0, sits at the right end of the plot.
        let first = svg.find("<title>0.0 – ").map(|_| ()).is_some();
        assert!(first, "{svg}");
        assert_eq!(
            render_err(&spec.replace(
                r#""kind": "number""#,
                r#""kind": "number", "timezone": "+02:00""#
            )),
            ("option_not_supported", "/timeAxis/timezone".to_owned())
        );
        assert_eq!(
            render_err(&spec.replace(r#""time": 2.5"#, r#""time": "2026-01-01""#)).0,
            "invalid_time"
        );
    }

    #[test]
    fn the_mobile_variant_of_small_multiples_takes_its_own_columns() {
        let spec = r#"{"schemaVersion": 1, "type": "multiples", "title": "Panels", "columns": 2,
            "mobile": {"width": 360, "height": 600, "columns": 1}, "panes": [
            {"title": "A", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 2}]}]},
            {"title": "B", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 3}, {"time": "2021", "value": 4}]}]}]}"#;
        let mobile = render_json(
            spec,
            RenderFormat::Svg,
            &RenderOptions {
                variant: Variant::Mobile,
                ..RenderOptions::default()
            },
        )
        .unwrap()
        .content;
        // One column: both panel titles start at the same x.
        let xs: Vec<&str> = mobile
            .split("<text ")
            .filter(|text| text.contains("class=\"chartlet-panel-title\""))
            .filter_map(|text| text.split('"').nth(1))
            .collect();
        assert_eq!(xs.len(), 2, "{mobile}");
        assert_eq!(xs[0], xs[1]);
        let bar = SPEC.replacen('{', "{\"mobile\": {\"width\": 360, \"columns\": 1},", 1);
        assert_eq!(
            render_err(&bar),
            ("option_not_supported", "/mobile/columns".to_owned())
        );
    }

    #[test]
    fn panels_on_axes_of_their_own_are_not_compared() {
        let spec = r#"{"schemaVersion": 1, "type": "multiples", "title": "Two units",
            "independentAxes": true, "panes": [
            {"title": "Small", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 2}]}]},
            {"title": "Large", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 1000}, {"time": "2021", "value": 2000}]}]}]}"#;
        let svg = render_ok(spec).content;
        assert!(svg.contains("with a value axis of their own"), "{svg}");
        assert!(!svg.contains("Highest:"));
        assert!(
            svg.contains(">2.0</text>") || svg.contains(">2</text>"),
            "{svg}"
        );
    }

    #[test]
    fn a_logarithmic_axis_takes_positive_values_only() {
        let log = SPEC.replacen('{', "{\"valueAxis\": {\"scale\": \"log\"},", 1);
        // SPEC has a negative value.
        assert_eq!(render_err(&log).0, "invalid_value");
        let positive = log.replace("-4", "4");
        let hooked = render_json(
            &positive.replace("\"type\": \"bar\"", "\"type\": \"line\""),
            RenderFormat::Svg,
            &RenderOptions {
                hooks: true,
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(hooked.content.contains(" data-y-scale=\"log\""));
        let svg = render_ok(&positive).content;
        assert!(
            svg.contains(">10</text>") || svg.contains(">1</text>"),
            "{svg}"
        );
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
            "Zeitreihe mit 3 Punkten von 01.03.2026 bis 03.03.2026. Höchster Wert: 20,0 (03.03.2026). Niedrigster Wert: \u{2212}1,5 (02.03.2026)."
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

    /// Adds `"locale": "de"` to a specification of the given type.
    fn german(spec: &str, chart_type: &str) -> String {
        spec.replace(
            &format!("\"type\": \"{chart_type}\","),
            &format!("\"type\": \"{chart_type}\", \"locale\": \"de\","),
        )
    }

    fn html_ok(spec: &str) -> String {
        render_json(spec, RenderFormat::Html, &RenderOptions::default())
            .unwrap()
            .content
    }

    #[test]
    fn german_bar_chart_writes_description_table_and_numbers_in_german() {
        let html = html_ok(&german(SPEC, "bar").replace("\"value\": -4}", "\"value\": -4.5}"));
        assert!(html.contains(
            "Balkendiagramm mit 2 Kategorien. Höchster Wert: 12 (North &lt;East&gt;). Niedrigster Wert: \u{2212}4,5 (South)."
        ));
        assert!(html.contains("<th scope=\"col\">Kategorie</th><th scope=\"col\">Wert</th>"));
        assert!(html.contains("<td>\u{2212}4,5</td>"));
    }

    #[test]
    fn german_stripes_write_description_tooltips_and_table_in_german() {
        let html = html_ok(&german(STRIPES, "stripes"));
        assert!(html.contains("<title>1851: \u{2212}0,5</title>"));
        assert!(html.contains(
            "Wärmestreifen von 1850 bis 1855, ein Streifen pro Jahr, auf einer divergierenden Farbskala um 0 mit den äußersten Stufen bei \u{2212}1 und 1. Niedrigster Wert: \u{2212}1 (1850). Höchster Wert: 2 (1855). 1 Jahr hat keinen Wert."
        ));
        assert!(html.contains("<th scope=\"col\">Jahr</th>"));
        assert!(html.contains("<th scope=\"row\">1852</th><td>fehlt</td>"));
    }

    #[test]
    fn german_calendar_names_months_and_weekdays_in_german() {
        let days = DAYS.replace("\"value\": -2}", "\"value\": -2.5}");
        let months = html_ok(&german(&calendar("months", &days), "calendar"));
        for month in ["Jan", "Mär", "Mai", "Okt", "Dez"] {
            assert!(months.contains(&format!(">{month}</text>")), "{month}");
        }
        assert!(!months.contains(">Mar</text>"));
        assert!(months.contains("<title>2024-01-01: \u{2212}2,5</title>"));
        assert!(months.contains("<title>2024-12-31: kein Wert</title>"));
        assert!(months.contains(
            "Kalender 2024 mit einer Zeile pro Monat, auf einer divergierenden Farbskala um "
        ));
        assert!(months.contains(
            "2 von 366 Tagen haben einen Wert. Niedrigster Wert: \u{2212}2,5 (2024-01-01). Höchster Wert: 4 (2024-02-29)."
        ));
        assert!(months.contains("<th scope=\"col\">Datum</th>"));
        let weeks = render_ok(&german(&calendar("weeks", DAYS), "calendar")).content;
        for weekday in ["Mo", "Mi", "Fr", "So"] {
            assert!(weeks.contains(&format!(">{weekday}</text>")), "{weekday}");
        }
        assert!(weeks.contains(">Mär</text>"));
        assert!(weeks.contains("einer Zeile pro Wochentag und einer Spalte pro Woche"));
    }

    #[test]
    fn german_range_chart_writes_spans_legend_and_table_in_german() {
        let html = html_ok(&german(RANGES, "rangebar"));
        assert!(html.contains("<title>Observed: 1,05 (0,9 bis 1,2)</title>"));
        assert!(html.contains("<title>Natural: \u{2212}0,1 bis 0,1, modelliert</title>"));
        assert!(html.contains(">Schraffiert: modelliert<"));
        assert!(html.contains(
            "Spannweitendiagramm mit 2 Kategorien, jeweils eine Spanne vom unteren zum oberen Wert mit einem mittleren Wert. Niedrigster unterer Wert: \u{2212}0,1 (Natural). Höchster oberer Wert: 1,2 (Observed). Modelliert, schraffiert gezeichnet: Natural."
        ));
        assert!(html.contains(
            "<th scope=\"col\">Kategorie</th><th scope=\"col\">Unterer Wert</th><th scope=\"col\">Mittlerer Wert</th><th scope=\"col\">Oberer Wert</th>"
        ));
        assert!(html.contains("<th scope=\"row\">Natural (modelliert)</th>"));
    }

    #[test]
    fn german_maps_write_description_picker_and_table_in_german() {
        let topicmap = html_ok(&german(TOPICMAP, "topicmap"));
        assert!(topicmap.contains(
            "Themenkarte mit 5 Gebieten und einer Insel. Größtes: AI in practice mit 291 Einträgen, kleinstes: Management mit 69. Engste Nachbarn: AI in practice und Engineering."
        ));
        assert!(topicmap.contains("<legend>Gebiet</legend>"));
        assert!(topicmap.contains(
            "<th scope=\"col\">Thema</th><th scope=\"col\">Einträge</th><th scope=\"col\">Pfade</th><th scope=\"col\">Anteil</th>"
        ));
        let atlas = html_ok(&german(ATLAS, "atlas"));
        assert!(atlas.contains(
            "Wissenslandschaft aus 2 Bereichen und 4 Regionen; die größte ist Models mit 40. 1 Ort ist markiert."
        ));
        assert!(atlas.contains(
            "<th scope=\"col\">Region</th><th scope=\"col\">Einträge</th><th scope=\"col\">Orte</th>"
        ));
    }

    #[test]
    fn the_text_alternative_matches_the_description_and_the_table() {
        let spec = ChartSpec::from_json(&german(TIME, "time")).unwrap();
        let alternative = text_alternative(&spec).unwrap();
        let html = html_ok(&german(TIME, "time"));
        assert!(html.contains(&crate::render::escape(&alternative.description)));
        assert!(html.contains(&format!(
            "<caption>{}</caption>",
            crate::render::escape(&alternative.table.caption)
        )));
        assert_eq!(alternative.table.columns[0], "Zeit");
        assert_eq!(alternative.table.rows.len(), 3);
        assert!(alternative.table.rows.iter().all(|row| row.len() == 2));
    }

    #[test]
    fn a_reference_line_crosses_the_bars_and_is_described() {
        let spec = SPEC.replacen(
            '{',
            "{\"references\": [{\"value\": 20, \"label\": \"Target\"}],",
            1,
        );
        let output = render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert!(output.content.contains("class=\"chartlet-rule\""));
        assert!(output.content.contains(">Target</text>"));
        assert!(output.content.contains("Reference lines: Target at 20."));
        let line = SPEC
            .replace("\"type\": \"bar\"", "\"type\": \"line\"")
            .replacen(
                '{',
                "{\"references\": [{\"value\": 1, \"label\": \"T\"}],",
                1,
            );
        assert_eq!(
            render_err(&line),
            ("option_not_supported", "/references".to_owned())
        );
    }

    #[test]
    fn a_value_axis_range_must_be_ordered_and_in_its_place() {
        let reversed = TIME.replace(
            "\"valueAxis\": {\"title\": \"Orders\"}",
            "\"valueAxis\": {\"title\": \"Orders\", \"min\": 10, \"max\": 5}",
        );
        assert_eq!(
            render_err(&reversed),
            ("invalid_axis_range", "/panes/0/valueAxis/max".to_owned())
        );
        let top_level = TIME.replacen('{', "{\"valueAxis\": {\"min\": 0},", 1);
        assert_eq!(
            render_err(&top_level),
            ("option_not_supported", "/valueAxis".to_owned())
        );
    }

    #[test]
    fn too_many_decimals_are_refused() {
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
        assert_eq!(
            svg.content
                .matches("class=\"chartlet-point chartlet-point-series-1\"")
                .count(),
            3
        );
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
                "expected an ISO 8601 date such as 2026-03-01 or 2026-03-01T12:00:00Z, a month such as 2026-03, or a year such as 1850",
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
            sparse
                .content
                .matches("class=\"chartlet-point chartlet-point-series-1\"")
                .count(),
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
        assert_eq!(
            dense
                .content
                .matches("class=\"chartlet-point chartlet-point-series-1\"")
                .count(),
            0
        );
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
    fn the_shared_stylesheet_is_the_common_part_and_one_part_per_type() {
        let common = crate::stylesheet_common();
        assert!(!common.contains(".chartlet-type-"));
        let (bar, time) = (
            crate::stylesheet_types(&[crate::ChartType::Bar]),
            crate::stylesheet_types(&[crate::ChartType::Time]),
        );
        assert_eq!(
            crate::stylesheet_for(&[crate::ChartType::Time, crate::ChartType::Bar]),
            format!("{common}{time}{bar}")
        );
        let every_type: String = crate::ChartType::ALL
            .iter()
            .map(|chart_type| crate::stylesheet_types(&[*chart_type]))
            .collect();
        assert_eq!(crate::stylesheet(), format!("{common}{every_type}"));
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
    fn declared_layer_colors_stay_inside_their_chart() {
        let specification = include_str!("../examples/revenue-vs-forecast.json");
        let svg = render_json(
            specification,
            RenderFormat::Svg,
            &RenderOptions {
                id_prefix: Some("first".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap();
        // Inline SVG stylesheets apply to the whole page; an unscoped rule would recolor the
        // layers of every other chart on it.
        assert!(svg.content.contains("id=\"first\" role=\"img\""));
        assert!(
            svg.content
                .contains("#first .chartlet-style-0{stroke:#7ea6ff}")
        );
        assert!(!svg.content.contains("}.chartlet-style-0{"));
        // Every other rule is scoped too, so a later chart's `.chartlet-line` cannot restyle
        // this one; the dark palette outranks a later light chart's defaults.
        assert!(svg.content.contains("#first .chartlet-title{"));
        assert!(
            svg.content
                .contains(".chartlet-root.chartlet-theme-dark{--chartlet-text:")
        );
        assert!(
            svg.content
                .contains("<style>.chartlet-root{--chartlet-text:")
        );
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
    fn layer_names_are_required_and_unique_within_a_pane() {
        let second = "{\"mark\": \"line\", \"name\": \"Orders\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 1}, {\"time\": \"2026-03-02\", \"value\": 2}]}";
        let duplicated = TIME.replace("\"layers\": [", &format!("\"layers\": [{second},"));
        let error = render_json(&duplicated, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("two layers with one name cannot be told apart");
        assert_eq!(error.code, "duplicate_series");
        assert_eq!(error.path, "/panes/0/layers/1/name");

        let anonymous = TIME.replace("\"name\": \"Orders\", ", "");
        let error = render_json(&anonymous, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        assert_eq!(error.warnings, []);
    }

    // --- Uncertainty bands, reference lines, stripes, calendars, range bars, small multiples ---

    #[test]
    fn a_manifest_is_returned_only_on_request_and_hashes_the_parsed_specification() {
        assert!(render_ok(SPEC).manifest.is_none());

        let options = RenderOptions {
            manifest: true,
            variant: Variant::Desktop,
            ..RenderOptions::default()
        };
        let output = render_json(SPEC, RenderFormat::Html, &options).unwrap();
        let manifest = output.manifest.expect("the manifest was asked for");
        assert_eq!(manifest.chartlet, env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.format, RenderFormat::Html);
        assert_eq!(
            manifest.output_hash,
            format!(
                "sha256:{}",
                super::sha256::hex(&super::sha256(output.content.as_bytes()))
            )
        );
        assert!(manifest.id_prefix.starts_with("chartlet-"));

        // Whitespace and key order of the input do not change the specification hash.
        let value: serde_json::Value = serde_json::from_str(SPEC).unwrap();
        let compact = render_json(&value.to_string(), RenderFormat::Html, &options).unwrap();
        assert_eq!(compact.manifest, Some(manifest.clone()));

        let json = manifest.to_json();
        let keys = [
            "chartlet",
            "schemaVersion",
            "specHash",
            "outputHash",
            "format",
            "variant",
            "idPrefix",
            "warnings",
        ];
        let positions: Vec<usize> = keys
            .iter()
            .map(|key| json.find(&format!("\"{key}\"")).unwrap())
            .collect();
        assert!(positions.is_sorted(), "{json}");
        assert!(json.contains("\"format\": \"html\""), "{json}");
    }

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
        let error = render_json(
            &with_rules(
                r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1,
                    "points": [{"time": "2026-03-01", "value": 1}]}"#,
            ),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .expect_err("a zone has no points");
        assert_eq!(error.code, "option_not_supported");
        assert_eq!(error.path, "/panes/0/layers/0/points");
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

    #[test]
    fn zones_are_drawn_behind_the_data_labelled_and_described() {
        let spec = with_rules(
            r##"{"mark": "band", "label": "Quiet", "from": "2026-03-01", "to": "2026-03-02"},
               {"mark": "band", "label": "Target", "bottom": 25, "top": 40, "color": "#16a34a"}"##,
        );
        let output = render_ok(&spec);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert_eq!(svg.matches("class=\"chartlet-zone").count(), 2);
        assert!(svg.contains("class=\"chartlet-zone chartlet-style-1-swatch\""));
        assert!(svg.contains(".chartlet-style-1-swatch{fill:#16a34a}"));
        // Behind the data: every zone comes before the line.
        assert!(svg.rfind("chartlet-zone\"").unwrap() < svg.find("class=\"chartlet-line").unwrap());
        assert!(svg.contains(">Quiet</text>"));
        assert!(svg.contains(">Target</text>"));
        assert!(svg.contains("<title>Quiet: 2026-03-01 to 2026-03-02</title>"));
        assert!(svg.contains("<title>Target: values 25 to 40</title>"));
        // 40 lies above every observation, so the axis reaches it.
        assert!(svg.contains(">40<"));
        assert!(svg.contains("Zones: Quiet, 2026-03-01 to 2026-03-02; Target, values 25 to 40."));
        // Zones are neither series in the legend nor columns in the table.
        assert!(!svg.contains("class=\"chartlet-legend\""));
        let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
        assert!(
            html.content
                .contains("<th scope=\"col\">Time</th><th scope=\"col\">Orders</th></tr>")
        );
    }

    #[test]
    fn a_zone_with_one_open_end_reaches_the_plot_edge_and_says_so() {
        let spec = with_rules(
            r#"{"mark": "band", "label": "Later", "from": "2026-03-02", "bottom": 0, "top": 5}"#,
        );
        let svg = render_ok(&spec).content;
        assert!(svg.contains("<title>Later: from 2026-03-02, values 0 to 5</title>"));
    }

    #[test]
    fn zones_are_validated_by_name() {
        for (zone, code, path) in [
            (
                r#"{"mark": "band", "bottom": 0, "top": 1}"#,
                "missing_label",
                "/panes/0/layers/0/label",
            ),
            (
                r#"{"mark": "band", "label": "Z"}"#,
                "missing_position",
                "/panes/0/layers/0",
            ),
            (
                r#"{"mark": "band", "label": "Z", "bottom": 2, "top": 2}"#,
                "invalid_band",
                "/panes/0/layers/0/top",
            ),
            (
                r#"{"mark": "band", "label": "Z", "from": "2026-03-02", "to": "2026-03-01"}"#,
                "invalid_band",
                "/panes/0/layers/0/to",
            ),
            (
                r#"{"mark": "band", "label": "Z", "from": "1600-01-01", "to": "2026-03-01"}"#,
                "invalid_time",
                "/panes/0/layers/0/from",
            ),
            (
                r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "name": "N"}"#,
                "option_not_supported",
                "/panes/0/layers/0/name",
            ),
            (
                r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "value": 1}"#,
                "option_not_supported",
                "/panes/0/layers/0/value",
            ),
            (
                r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "shape": "circle"}"#,
                "option_not_supported",
                "/panes/0/layers/0/shape",
            ),
            (
                r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "dash": "dotted"}"#,
                "option_not_supported",
                "/panes/0/layers/0/dash",
            ),
        ] {
            assert_eq!(
                render_err(&with_rules(zone)),
                (code, path.to_owned()),
                "{zone}"
            );
        }
        // Zones, reference lines and point markers share one limit.
        let seven = (0..7)
            .map(|index| match index % 3 {
                0 => format!(r#"{{"mark": "band", "bottom": {index}, "top": 50, "label": "Z{index}"}}"#),
                1 => format!(r#"{{"mark": "annotation", "value": {index}, "label": "R{index}"}}"#),
                _ => format!(
                    r#"{{"mark": "annotation", "time": "2026-03-02", "value": {index}, "label": "M{index}"}}"#
                ),
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            render_err(&with_rules(&seven)),
            ("too_many_annotations", "/panes/0/layers".to_owned())
        );
    }

    #[test]
    fn point_markers_are_drawn_in_their_shape_labelled_and_described() {
        let spec = with_rules(
            r##"{"mark": "annotation", "time": "2026-03-02", "value": 50, "label": "Launch", "shape": "triangle-up"},
               {"mark": "annotation", "time": "2026-03-01", "value": 12, "label": "Start"},
               {"mark": "annotation", "time": "2026-03-02", "value": 5, "label": "Dip", "shape": "diamond", "color": "#16a34a"}"##,
        );
        let output = render_ok(&spec);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // Shapes other than the circle are closed outlines; the default is a circle.
        assert_eq!(svg.matches("class=\"chartlet-marker").count(), 3);
        assert!(svg.contains("<circle cx="));
        assert!(svg.contains("class=\"chartlet-marker\"><title>Start: 2026-03-01, 12</title>"));
        assert!(svg.contains("<title>Launch: 2026-03-02, 50</title>"));
        assert!(svg.contains(
            "class=\"chartlet-marker chartlet-style-2\"><title>Dip: 2026-03-02, 5</title>"
        ));
        assert!(svg.contains(
            ".chartlet-marker.chartlet-style-2{fill:#16a34a;stroke:var(--chartlet-background)}"
        ));
        assert!(svg.contains(">Launch</text>"));
        // 50 lies above every observation, so the axis reaches it.
        assert!(svg.contains(">50<"));
        assert!(svg.contains(
            "Markers: Launch at 2026-03-02, 50; Start at 2026-03-01, 12; Dip at 2026-03-02, 5."
        ));
        assert!(!svg.contains("Reference lines"));
        assert!(!svg.contains("class=\"chartlet-legend\""));
    }

    #[test]
    fn a_point_marker_needs_a_label_and_only_it_takes_a_shape() {
        assert_eq!(
            render_err(&with_rules(
                r#"{"mark": "annotation", "time": "2026-03-02", "value": 5}"#
            )),
            ("missing_label", "/panes/0/layers/0/label".to_owned())
        );
        let error = render_json(
            &with_rules(
                r#"{"mark": "annotation", "time": "2026-03-02", "label": "L", "shape": "square"}"#,
            ),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .expect_err("a reference line has no shape");
        assert_eq!(error.code, "option_not_supported");
        assert_eq!(error.path, "/panes/0/layers/0/shape");
        assert!(error.message.contains("point marker"), "{}", error.message);
    }

    #[test]
    fn a_marker_label_moves_to_the_left_at_the_right_edge() {
        let spec = with_rules(
            r#"{"mark": "annotation", "time": "2026-03-03", "value": 20, "label": "Peak"}"#,
        );
        let output = render_ok(&spec);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(
            output
                .content
                .contains("text-anchor=\"end\" class=\"chartlet-rule-label\">Peak</text>")
        );
    }

    #[test]
    fn overlapping_annotation_labels_are_reported_not_dropped() {
        let spec = with_rules(
            r#"{"mark": "annotation", "value": 12, "label": "First"},
               {"mark": "annotation", "value": 12, "label": "Second"}"#,
        );
        let output = render_ok(&spec);
        assert!(output.content.contains(">First</text>"));
        assert!(output.content.contains(">Second</text>"));
        assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
        assert_eq!(output.warnings[0].code, "label_overlap");
        assert_eq!(output.warnings[0].path, "/panes/0/layers/1");
    }

    #[test]
    fn a_reference_line_label_across_a_data_line_is_reported() {
        let spec = TIME
            .replace("\"value\": 10}", "\"value\": 10.4}")
            .replace("\"value\": 15}", "\"value\": 10.4}")
            .replace("\"value\": 20}", "\"value\": 0}")
            .replace(
                "\"layers\": [",
                r#""layers": [{"mark": "annotation", "value": 10, "label": "Limit"},"#,
            );
        let output = render_ok(&spec);
        assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
        assert_eq!(output.warnings[0].code, "label_overlap");
        assert_eq!(output.warnings[0].path, "/panes/0/layers/0");
        assert!(output.warnings[0].message.contains("data line"));
    }

    #[test]
    fn a_label_outside_the_plot_is_reported() {
        // An area starts the axis at zero without padding, so 20 is the top edge of the plot and
        // the label of a reference line there sits above it.
        let spec = TIME
            .replace("\"mark\": \"line\"", "\"mark\": \"area\"")
            .replace(
                "\"layers\": [",
                r#""layers": [{"mark": "annotation", "value": 20, "label": "Ceiling"},"#,
            );
        let output = render_ok(&spec);
        assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
        assert_eq!(output.warnings[0].code, "label_overlap");
        assert_eq!(output.warnings[0].path, "/panes/0/layers/0");
        assert!(output.warnings[0].message.contains("outside"));
    }

    #[test]
    fn label_collisions_are_measured_with_the_injected_metrics() {
        // Wide metrics stretch the label of the horizontal line into that of the vertical one.
        struct Wide;
        impl TextMetrics for Wide {
            fn width(&self, text: &str, font_size: f64) -> f64 {
                BuiltinMetrics.width(text, font_size) * 4.0
            }
        }
        let spec = ChartSpec::from_json(&with_rules(
            r#"{"mark": "annotation", "value": 24, "label": "Ceiling"},
               {"mark": "annotation", "time": "2026-03-01T06:00:00Z", "label": "Deploy"}"#,
        ))
        .unwrap();
        let narrow = render_ok(&serde_json::to_string(&spec).unwrap());
        assert!(narrow.warnings.is_empty(), "{:?}", narrow.warnings);
        let wide = render_with_metrics(&spec, RenderFormat::Svg, &RenderOptions::default(), &Wide)
            .unwrap();
        assert!(
            wide.warnings
                .iter()
                .any(|warning| warning.code == "label_overlap"),
            "{:?}",
            wide.warnings
        );
    }

    #[test]
    fn zones_and_markers_are_described_in_german() {
        let spec = with_rules(
            r#"{"mark": "band", "label": "Ruhe", "from": "2026-03-01", "to": "2026-03-02"},
               {"mark": "band", "label": "Ziel", "bottom": 2.5},
               {"mark": "annotation", "time": "2026-03-02", "value": 12.5, "label": "Start"}"#,
        )
        .replace(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1, \"locale\": \"de\",",
        );
        // A zone with one edge covers everything above it, up to the edge of the plot.
        let open = render_ok(&spec).content;
        assert!(open.contains("Ziel, Werte über 2,5."), "{open}");
        let bare = spec.replace(r#", "bottom": 2.5}"#, "}");
        assert_eq!(
            render_err(&bare),
            ("missing_position", "/panes/0/layers/1".to_owned())
        );
        let spec = spec.replace(
            r#""bottom": 2.5}"#,
            r#""bottom": 2.5, "from": "2026-03-02", "to": "2026-03-03"}"#,
        );
        let svg = render_ok(&spec).content;
        assert!(svg.contains(
            "Bereiche: Ruhe, 01.03.2026 bis 02.03.2026; Ziel, 02.03.2026 bis 03.03.2026, Werte über 2,5."
        ), "{svg}");
        assert!(svg.contains("Markierungen: Start bei 02.03.2026, 12,5."));
        assert!(svg.contains("<title>Start: 02.03.2026, 12,5</title>"));
    }

    #[test]
    fn a_zoom_window_cuts_zones_and_leaves_out_those_it_misses() {
        let spec = with_rules(
            r#"{"mark": "band", "label": "Early", "from": "2026-03-01", "to": "2026-03-02"},
               {"mark": "band", "label": "Wide", "from": "2026-02-20", "to": "2026-03-10"}"#,
        )
        .replace(
            "\"timeAxis\"",
            r#""zoomSteps": [{"label": "All", "from": "2026-03-01", "to": "2026-03-03"},
                             {"label": "Late", "from": "2026-03-02", "to": "2026-03-03"}],
               "timeAxis""#,
        );
        let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default())
            .unwrap()
            .content;
        // Both zones in the first window, only the wide one in the second.
        assert_eq!(html.matches("class=\"chartlet-zone\"").count(), 3);
        assert!(html.contains("<title>Wide: 2026-03-02 to 2026-03-03</title>"));
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
            // The legend text needs its own color; without it a dark chart draws it black.
            assert!(svg.contains(".chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}"));
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
        // Every area names the center it was placed around, inside the viewBox.
        let centers = output
            .content
            .split("class=\"chartlet-topic-area")
            .skip(1)
            .map(|rest| {
                let attribute = |name: &str| -> f64 {
                    let start = rest.find(name).expect("center attribute") + name.len();
                    let end = start + rest[start..].find('"').expect("closing quote");
                    rest[start..end].parse().expect("a number")
                };
                (attribute("data-cx=\""), attribute("data-cy=\""))
            })
            .collect::<Vec<_>>();
        assert_eq!(centers.len(), 6);
        for (cx, cy) in centers {
            assert!((0.0..=800.0).contains(&cx) && (0.0..=450.0).contains(&cy));
        }

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
        // The rule names the map's root ID: the chart's own rules are scoped to it and would
        // outrank a rule that selects by class alone.
        let root = output.content.split(" id=\"").nth(1).unwrap();
        let root = &root[..root.find('"').unwrap()];
        assert!(output.content.contains(&format!(
            ".chartlet-wrapper:has(.chartlet-topic-picker input.topic-2:checked) #{root} .chartlet-topic-area.chartlet-topic-2{{fill:var(--chartlet-accent)}}"
        )));
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
        assert!(svg.content.contains("1 place is marked"));
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

    // --- Gaps, line patterns, areas, time zoom and more layers ---

    /// A time chart over five days whose pane holds `layers`; `extra` goes into the top level.
    fn week(layers: &str, extra: &str) -> String {
        format!(
            r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Sensors",
                "showValues": false{extra},
                "panes": [{{"layers": [{layers}]}}]
            }}"#
        )
    }

    /// A line layer over the five days of [`week`]; `fields` goes into the layer object and
    /// `values` are the five values, `null` included.
    fn sensor(name: &str, fields: &str, values: [&str; 5]) -> String {
        let points = values
            .iter()
            .enumerate()
            .map(|(day, value)| format!(r#"{{"time": "2026-03-0{}", "value": {value}}}"#, day + 1))
            .collect::<Vec<_>>()
            .join(", ");
        format!(r#"{{"mark": "line", "name": "{name}"{fields}, "points": [{points}]}}"#)
    }

    const GAPPED: [&str; 5] = ["10", "12", "null", "14", "15"];

    #[test]
    fn a_null_value_breaks_the_line_and_counts_as_missing() {
        let spec = week(&sensor("Load", "", GAPPED), "");
        let output = render_ok(&spec);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // Two pieces of line around the gap, and a marker for each of the four values.
        assert_eq!(svg.matches("<polyline").count(), 2);
        assert_eq!(
            svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
                .count(),
            4
        );
        assert!(svg.contains("1 value is missing."), "{svg}");

        let html = html_ok(&spec);
        assert!(html.contains("<th scope=\"row\">2026-03-03</th><td>Missing</td>"));

        let german = html_ok(&german(&spec, "time"));
        assert!(german.contains("1 Wert fehlt."));
        assert!(german.contains("<th scope=\"row\">03.03.2026</th><td>fehlt</td>"));

        // A lone value between two gaps keeps its marker but draws no line.
        let lone = week(&sensor("Load", "", ["10", "null", "12", "null", "15"]), "");
        let svg = render_ok(&lone).content;
        assert!(!svg.contains("<polyline"), "{svg}");
        assert_eq!(
            svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
                .count(),
            3
        );
        assert!(svg.contains("2 values are missing."));

        // Small multiples break their lines the same way.
        let gapped = panel("B", 1.0).replacen(
            r#"{"time": "2010", "value": 12}"#,
            r#"{"time": "2005", "value": null}, {"time": "2010", "value": 12}"#,
            1,
        );
        let spec = multiples(&[panel("A", 1.0), gapped].join(","));
        assert!(render_ok(&spec).content.contains("1 value is missing."));
    }

    #[test]
    fn a_band_breaks_at_a_missing_value_and_a_missing_value_has_no_band() {
        let banded = |gap: &str| {
            let points = [
                r#"{"time": "2026-03-01", "value": 1, "lower": 0, "upper": 2}"#,
                r#"{"time": "2026-03-02", "value": 2, "lower": 1, "upper": 3}"#,
                gap,
                r#"{"time": "2026-03-04", "value": 2, "lower": 1, "upper": 3}"#,
                r#"{"time": "2026-03-05", "value": 3, "lower": 2, "upper": 4}"#,
            ]
            .join(", ");
            week(
                &format!(r#"{{"mark": "line", "name": "Load", "points": [{points}]}}"#),
                "",
            )
        };
        let svg = render_ok(&banded(r#"{"time": "2026-03-03", "value": null}"#)).content;
        assert_eq!(
            svg.matches("class=\"chartlet-band chartlet-band-series-1\"")
                .count(),
            2
        );
        assert_eq!(
            render_err(&banded(
                r#"{"time": "2026-03-03", "value": null, "lower": 1, "upper": 3}"#
            )),
            (
                "invalid_band",
                "/panes/0/layers/0/points/2/lower".to_owned()
            )
        );
    }

    #[test]
    fn a_line_takes_its_declared_pattern_and_weight_in_plot_and_legend() {
        let spec = week(
            &[
                sensor("Dotted", r#", "dash": "dotted""#, GAPPED),
                sensor("Bold", r#", "stroke": "bold""#, GAPPED),
                sensor("Dashed", r#", "dash": "dashed""#, GAPPED),
                sensor(
                    "Solid model",
                    r#", "modeled": true, "dash": "solid""#,
                    GAPPED,
                ),
            ]
            .join(", "),
            "",
        );
        let svg = render_ok(&spec).content;
        assert!(svg.contains(".chartlet-line-dotted{stroke-dasharray:0 7}"));
        assert!(svg.contains(".chartlet-line-bold{stroke-width:4.5}"));
        // Two pieces of line around the gap, and the legend sample.
        for class in [
            "chartlet-line chartlet-line-dotted chartlet-line-series-1",
            "chartlet-line chartlet-line-bold chartlet-line-series-2",
            "chartlet-line chartlet-line-dashed chartlet-line-series-3",
            "chartlet-line chartlet-line-series-4",
        ] {
            assert_eq!(
                svg.matches(&format!("class=\"{class}\"")).count(),
                3,
                "{class}"
            );
        }
        // An explicit pattern replaces the modeled dash, but the legend still says modeled.
        assert!(svg.contains(">Solid model (modeled)<"));
        assert!(!svg.contains("class=\"chartlet-line chartlet-line-modeled"));

        // Without the new options a chart carries none of their rules.
        let plain = render_ok(&week(&sensor("Load", "", GAPPED), "")).content;
        assert!(!plain.contains("chartlet-line-dotted"));

        let rule = week(
            &format!(
                r#"{}, {{"mark": "annotation", "value": 1, "label": "Limit", "dash": "dotted"}}"#,
                sensor("Load", "", GAPPED)
            ),
            "",
        );
        assert_eq!(
            render_err(&rule),
            ("option_not_supported", "/panes/0/layers/1/dash".to_owned())
        );
    }

    #[test]
    fn an_area_is_filled_down_to_zero_and_described() {
        let area = sensor("Load", "", ["10", "12", "null", "14", "15"])
            .replace("\"mark\": \"line\"", "\"mark\": \"area\"");
        let spec = week(
            &[
                area.clone(),
                sensor("Base", "", ["8", "9", "9", "10", "11"]),
            ]
            .join(", "),
            "",
        );
        let output = render_ok(&spec);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(svg.contains(".chartlet-area{stroke:none;fill-opacity:.18}"));
        // One filled outline on either side of the gap, and a swatch under the legend sample.
        assert_eq!(
            svg.matches("<polyline points=\"").count(),
            // Two area outlines, two line pieces, one line for the second layer, two samples.
            7
        );
        assert_eq!(
            svg.matches("class=\"chartlet-area chartlet-band-series-1\"")
                .count(),
            3
        );
        // The value axis reaches zero although every value lies well above it.
        assert!(svg.contains("class=\"chartlet-tick\">0</text>"), "{svg}");
        assert!(svg.contains("The area below Load is filled down to zero."));
        assert!(
            html_ok(&german(&spec, "time")).contains("Die Fläche unter Load ist bis null gefüllt.")
        );

        let colored = week(
            &area.replace(
                "\"name\": \"Load\"",
                "\"name\": \"Load\", \"color\": \"#0f766e\"",
            ),
            "",
        );
        let svg = render_ok(&colored).content;
        assert!(svg.contains(".chartlet-area.chartlet-style-0{fill:#0f766e;stroke:none}"));
        assert!(svg.contains("The area below Load is filled down to zero."));
    }

    #[test]
    fn a_time_chart_zooms_into_windows_of_time() {
        let zoom = r#", "zoomSteps": [
            {"label": "All", "from": "2026-03-01", "to": "2026-03-05"},
            {"label": "End", "from": "2026-03-04", "to": 1772668800}
        ]"#;
        let spec = week(&sensor("Load", "", GAPPED), zoom);
        let html = html_ok(&spec);
        assert!(html.contains("<fieldset class=\"chartlet-zoom\">"));
        assert_eq!(html.matches("<svg").count(), 2);
        let end = html
            .split("chartlet-panel chartlet-panel-1")
            .nth(1)
            .and_then(|panel| panel.split("</div>").next())
            .expect("the second window has a panel");
        assert!(end.contains("Time chart with 2 points from 2026-03-04 to 2026-03-05."));
        assert!(!end.contains("2026-03-01"));
        // The table keeps every observation.
        assert!(html.contains("<th scope=\"row\">2026-03-01</th>"));
        // The SVG profile stays one static chart.
        assert_eq!(render_ok(&spec).content.matches("<svg").count(), 1);

        for (steps, code, path) in [
            (
                r#"[{"label": "A", "from": "2026-03-04", "to": "2026-03-04"}, {"label": "B", "from": "2026-03-01", "to": "2026-03-05"}]"#,
                "invalid_zoom_step",
                "/zoomSteps/0/from",
            ),
            (
                r#"[{"label": "A", "from": "2026-03-01", "to": "2026-03-05"}, {"label": "B", "from": "2026-03-02", "to": "2026-03-03"}]"#,
                "zoom_out_of_range",
                "/zoomSteps/1",
            ),
            (
                r#"[{"label": "A", "from": "March", "to": "2026-03-05"}, {"label": "B", "from": "2026-03-01", "to": "2026-03-05"}]"#,
                "invalid_time",
                "/zoomSteps/0/from",
            ),
            (
                r#"[{"label": "A", "from": "2026-03-01", "to": "2026-03-05"}]"#,
                "not_enough_zoom_steps",
                "/zoomSteps",
            ),
        ] {
            let spec = week(
                &sensor("Load", "", GAPPED),
                &format!(r#", "zoomSteps": {steps}"#),
            );
            assert_eq!(render_err(&spec), (code, path.to_owned()), "{steps}");
        }

        // A category chart still counts categories, and a timestamp is no category.
        let dated = SPEC.replace(
            "\"schemaVersion\": 1,",
            r#""schemaVersion": 1, "zoomSteps": [{"label": "A", "from": "2026-03-01", "to": 1}, {"label": "B", "from": 0, "to": 1}],"#,
        );
        assert_eq!(
            render_err(&dated),
            ("invalid_spec", "/zoomSteps/0/from".to_owned())
        );
        let panels = [panel("A", 1.0), panel("B", 1.0)].join(",");
        let zoomed = multiples(&panels).replace(
            "\"schemaVersion\": 1,",
            r#""schemaVersion": 1, "zoomSteps": [{"label": "A", "from": "2000", "to": "2010"}, {"label": "B", "from": "2000", "to": "2050"}],"#,
        );
        assert_eq!(
            render_err(&zoomed),
            ("option_not_supported", "/zoomSteps".to_owned())
        );
    }

    #[test]
    fn a_pane_takes_six_layers_when_two_bring_their_own_color() {
        let colored = r##", "color": "#0f766e""##;
        let layers = |names: &[(&str, &str)]| {
            names
                .iter()
                .map(|(name, fields)| sensor(name, fields, GAPPED))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let six = [
            ("Own", colored),
            ("A", ""),
            ("B", ""),
            ("C", ""),
            ("Also own", colored),
            ("D", ""),
        ];
        let output = render_ok(&week(&layers(&six), ""));
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // The fifth data layer takes the first palette color no layer before it holds.
        assert!(svg.contains("class=\"chartlet-line chartlet-line-series-1\""));
        for series in 2..=4 {
            assert!(svg.contains(&format!("chartlet-line-series-{series}")));
        }
        assert_eq!(svg.matches("class=\"chartlet-legend\"").count(), 6);

        let seven = [six.as_slice(), &[("E", colored)]].concat();
        assert_eq!(
            render_err(&week(&layers(&seven), "")),
            ("too_many_layers", "/panes/0/layers".to_owned())
        );
        let five_in_palette = [("A", ""), ("B", ""), ("C", ""), ("D", ""), ("E", "")];
        assert_eq!(
            render_err(&week(&layers(&five_in_palette), "")),
            ("too_many_layers", "/panes/0/layers/4/color".to_owned())
        );
    }

    /// Position and content of every text of `class`, in document order.
    fn placed_texts(svg: &str, class: &str) -> Vec<(f64, f64, String)> {
        svg.split("<text ")
            .skip(1)
            .filter(|chunk| chunk.contains(&format!("class=\"{class}\"")))
            .map(|chunk| {
                let read = |name: &str| -> f64 {
                    chunk
                        .split(&format!(" {name}=\""))
                        .nth(1)
                        .or_else(|| chunk.strip_prefix(&format!("{name}=\"")))
                        .and_then(|rest| rest.split('"').next())
                        .expect("a coordinate")
                        .parse()
                        .expect("a number")
                };
                let content = chunk
                    .split_once('>')
                    .and_then(|(_, rest)| rest.split_once("</text>"))
                    .map(|(content, _)| content.to_owned())
                    .expect("a text element has content");
                (read("x"), read("y"), content)
            })
            .collect()
    }

    #[test]
    fn a_long_legend_wraps_into_rows_without_overlap() {
        let names = [
            "Server hall north",
            "Server hall south",
            "Cooling plant",
            "Office floors",
            "Lighting",
            "Charging",
        ];
        let layers = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let fields = if index < 4 {
                    String::new()
                } else {
                    format!(r##", "color": "#12345{index}""##)
                };
                sensor(name, &fields, GAPPED)
            })
            .collect::<Vec<_>>()
            .join(", ");
        let svg = render_ok(&week(&layers, r#", "width": 480"#)).content;
        let legend = placed_texts(&svg, "chartlet-legend");
        let written: Vec<&str> = legend
            .iter()
            .map(|(_, _, content)| content.as_str())
            .collect();
        assert_eq!(written, names, "every name is written in full");
        let rows: std::collections::BTreeSet<u64> =
            legend.iter().map(|(_, y, _)| y.to_bits()).collect();
        assert!(rows.len() > 1, "{legend:?}");
        let width = |content: &str| BuiltinMetrics.width(content, 12.0);
        for (x, y, content) in &legend {
            assert!(x + width(content) <= 480.0, "{content} leaves the chart");
            for (other_x, other_y, other) in &legend {
                if other_y.to_bits() == y.to_bits() && other_x > x {
                    assert!(
                        x + width(content) < *other_x - 24.0,
                        "{content} runs into {other}"
                    );
                }
            }
        }
        // The plot starts below the last legend row.
        let lowest = legend.iter().map(|(_, y, _)| *y).fold(0.0, f64::max);
        let ticks = placed_texts(&svg, "chartlet-tick");
        assert!(ticks.iter().all(|(_, y, _)| *y > lowest), "{ticks:?}");
    }

    // --- Candlesticks and stacked panes ---

    /// Three candles: rising, falling, and one that opens and closes alike.
    const CANDLES: &str = r#"[
        {"time": "2026-03-02", "open": 10, "high": 12, "low": 9, "close": 11},
        {"time": "2026-03-03", "open": 11, "high": 11.5, "low": 8, "close": 9},
        {"time": "2026-03-04", "open": 9, "high": 10, "low": 8.5, "close": 9}
    ]"#;

    /// A time chart whose panes are given as JSON; `extra` goes into the top level.
    fn stacked(panes: &str, extra: &str) -> String {
        format!(
            r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Share price",
                "timeAxis": {{"title": "Day"}}{extra},
                "panes": [{panes}]
            }}"#
        )
    }

    fn candle_pane(extra_layers: &str) -> String {
        format!(
            r#"{{"heightRatio": 3, "valueAxis": {{"title": "Price", "decimals": 2}}, "layers": [{{"mark": "ohlc", "name": "Price", "data": {CANDLES}}}{extra_layers}]}}"#
        )
    }

    const VOLUME_PANE: &str = r#"{"valueAxis": {"title": "Volume", "format": "percent"}, "layers": [{"mark": "area", "name": "Volume", "points": [{"time": "2026-03-02", "value": 0.12}, {"time": "2026-03-03", "value": 0.2}, {"time": "2026-03-04", "value": 0.08}]}]}"#;

    #[test]
    fn candles_differ_in_shape_and_color_by_direction() {
        let output = render_ok(&stacked(&candle_pane(""), ""));
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let svg = output.content;
        // A rising candle is hollow, a falling one filled; the legend sample shows both.
        assert_eq!(
            svg.matches("class=\"chartlet-candle chartlet-candle-rise\"")
                .count(),
            3,
            "two rising or level candles and the legend sample"
        );
        assert_eq!(
            svg.matches("class=\"chartlet-candle chartlet-candle-fall\"")
                .count(),
            2
        );
        assert!(svg.contains(
            ".chartlet-candle-rise{fill:var(--chartlet-background);stroke:var(--chartlet-rise)}"
        ));
        assert!(svg.contains(".chartlet-theme-dark{--chartlet-rise:"));
        assert!(svg.contains("Price (hollow: rising, filled: falling)</text>"));
        assert!(svg.contains(
            "<title>2026-03-02 – Price: open 10.00, high 12.00, low 9.00, close 11.00</title>"
        ));
        // A level candle keeps a body one pixel high.
        assert!(svg.contains(" height=\"1\" class=\"chartlet-candle chartlet-candle-rise\""));
        assert!(svg.contains(
            "opened at 10.00 on 2026-03-02 and closed at 9.00 on 2026-03-04, a change of \u{2212}1.00 (\u{2212}10.0%); high 12.00 on 2026-03-02, low 8.00 on 2026-03-03."
        ), "{svg}");
        let html = html_ok(&stacked(&candle_pane(""), ""));
        assert!(html.contains("<th scope=\"col\">Price (open)</th><th scope=\"col\">Price (high)</th><th scope=\"col\">Price (low)</th><th scope=\"col\">Price (close)</th>"));
        assert!(html.contains("<td>11.00</td><td>11.50</td><td>8.00</td><td>9.00</td>"));
        // Charts without candles carry none of their styles.
        assert!(!render_ok(TIME).content.contains("chartlet-rise"));
    }

    #[test]
    fn dense_candles_are_drawn_as_wicks_only() {
        // 704 plot pixels at the default width leave room for 234 candles.
        let data = (0..240)
            .map(|index| {
                format!(
                    r#"{{"time": {}, "open": 10, "high": 12, "low": 9, "close": 11}}"#,
                    1_770_000_000 + index * 86_400
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let output = render_ok(&stacked(
            &format!(r#"{{"layers": [{{"mark": "ohlc", "data": [{data}]}}]}}"#),
            "",
        ));
        assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
        assert_eq!(output.warnings[0].code, "dense_chart");
        assert_eq!(output.warnings[0].path, "/panes/0/layers/0/data");
        let svg = output.content;
        assert_eq!(
            svg.matches("class=\"chartlet-wick chartlet-wick-rise\"")
                .count(),
            241
        );
        // Only the legend sample keeps its bodies.
        assert_eq!(
            svg.matches("class=\"chartlet-candle chartlet-candle-rise\"")
                .count(),
            1
        );
    }

    #[test]
    fn invalid_candles_are_refused_with_their_path() {
        let refuse = |data: &str, layer: &str| {
            render_err(&stacked(
                &format!(r#"{{"layers": [{{"mark": "ohlc", "data": {data}{layer}}}]}}"#),
                "",
            ))
        };
        let valid = CANDLES;
        assert_eq!(
            refuse(&valid.replace("\"low\": 8,", "\"low\": 9.5,"), ""),
            ("invalid_candle", "/panes/0/layers/0/data/1/low".to_owned())
        );
        assert_eq!(
            refuse(&valid.replace("\"high\": 12,", "\"high\": 10.5,"), ""),
            ("invalid_candle", "/panes/0/layers/0/data/0/high".to_owned())
        );
        assert_eq!(
            refuse(&valid.replace("2026-03-03", "2026-03-02"), ""),
            ("unordered_time", "/panes/0/layers/0/data/1/time".to_owned())
        );
        assert_eq!(
            refuse(&valid.replace("\"open\": 10,", "\"open\": 1e101,"), ""),
            (
                "unsupported_numeric_range",
                "/panes/0/layers/0/data/0/open".to_owned()
            )
        );
        assert_eq!(
            refuse(
                r#"[{"time": "2026-03-02", "open": 1, "high": 2, "low": 0, "close": 1}]"#,
                ""
            ),
            ("empty_series", "/panes/0/layers/0/data".to_owned())
        );
        assert_eq!(
            refuse(valid, r##", "color": "#123""##),
            ("option_not_supported", "/panes/0/layers/0/color".to_owned())
        );
        assert_eq!(
            refuse(valid, r#", "points": [{"time": "2026-03-02", "value": 1}]"#),
            (
                "option_not_supported",
                "/panes/0/layers/0/points".to_owned()
            )
        );
        assert_eq!(
            render_err(&multiples(&format!(
                r#"{{"title": "A", "layers": [{{"mark": "ohlc", "data": {valid}}}]}}, {}"#,
                panel("B", 1.0)
            ))),
            ("option_not_supported", "/panes/0/layers/0/mark".to_owned())
        );
    }

    #[test]
    fn panes_stack_by_their_ratio_and_share_one_time_axis_at_the_bottom() {
        let svg = render_ok(&stacked(&format!("{}, {VOLUME_PANE}", candle_pane("")), "")).content;
        // 450 high: the plot runs from 102 (title and one legend row) to 394 (time axis title).
        // Less the gap of 36, the ratio 3:1 splits 256 pixels into 192 and 64.
        let titles = placed_texts(&svg, "chartlet-axis-title");
        let at = |name: &str| {
            titles
                .iter()
                .find(|(_, _, content)| content == name)
                .map(|(_, y, _)| *y)
                .expect("the axis title is drawn")
        };
        assert!((at("Price") - 82.0).abs() < 1e-9);
        assert!((at("Volume") - 310.0).abs() < 1e-9);
        // Time tick labels sit only below the bottom pane; value ticks at the left of both.
        let ticks = placed_texts(&svg, "chartlet-tick");
        let time_ticks: Vec<_> = ticks.iter().filter(|(x, _, _)| *x > 72.0).collect();
        assert_ne!(time_ticks.len(), 0);
        assert!(
            time_ticks.iter().all(|(_, y, _)| (*y - 418.0).abs() < 1e-9),
            "{time_ticks:?}"
        );
        assert!(
            ticks
                .iter()
                .any(|(x, _, content)| *x <= 72.0 && !content.ends_with('%'))
        );
        assert!(ticks.iter().any(|(_, _, content)| content.ends_with('%')));
        // Vertical grid lines run through both panes, each within its own.
        assert!(svg.contains("y1=\"102\" x2="));
        assert!(svg.contains("y1=\"330\" x2="));
        assert!(svg.contains("y2=\"394\" class=\"chartlet-grid\""));
        // The panes share one legend, and the area of the second pane takes the next palette
        // color, since candles take none.
        assert!(svg.contains("Volume</text>"));
        assert!(svg.contains("chartlet-area chartlet-band-series-1"));
        assert!(svg.contains(" 2 stacked panes on one time axis: Price, Volume."));
        assert!(
            svg.contains(
                " Volume – Highest: 20% (Volume in 2026-03-03). Lowest: 8% (Volume in 2026-03-04)."
            ),
            "{svg}"
        );
    }

    #[test]
    fn every_pane_writes_its_values_by_its_own_axis() {
        let html = html_ok(&stacked(
            &format!(
                "{}, {VOLUME_PANE}",
                candle_pane(
                    r#", {"mark": "line", "name": "Average", "points": [{"time": "2026-03-02", "value": 10.5}, {"time": "2026-03-04", "value": 10}]}, {"mark": "annotation", "label": "Target", "value": 11}"#
                )
            ),
            "",
        ));
        assert!(html.contains("<td>10.50</td>"), "{html}");
        assert!(html.contains("<td>12%</td>"));
        assert!(html.contains("Reference lines: Target at 11.00."));
        assert!(
            html.contains("chartlet-line-series-1"),
            "the line takes the first palette color"
        );
    }

    #[test]
    fn pane_limits_are_refused_by_name() {
        let line = |name: &str| {
            format!(
                r#"{{"layers": [{{"mark": "line", "name": "{name}", "points": [{{"time": "2026-03-02", "value": 1}}, {{"time": "2026-03-03", "value": 2}}]}}]}}"#
            )
        };
        let five = (0..5)
            .map(|index| line(&format!("L{index}")))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            render_err(&stacked(&five, "")),
            ("too_many_panes", "/panes".to_owned())
        );
        let four = (0..4)
            .map(|index| line(&format!("L{index}")))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(render_ok(&stacked(&four, "")).warnings, []);
        let tall = format!(
            "{}, {}",
            line("A"),
            line("B").replacen('{', r#"{"heightRatio": 11, "#, 1)
        );
        assert_eq!(
            render_err(&stacked(&tall, "")),
            ("invalid_height_ratio", "/panes/1/heightRatio".to_owned())
        );
        assert_eq!(
            render_err(&stacked(&format!("{}, {}", line("A"), line("A")), "")),
            ("duplicate_series", "/panes/1/layers/0/name".to_owned())
        );
        let anonymous = line("A").replace(r#""name": "A", "#, "");
        assert_eq!(
            render_err(&stacked(&format!("{anonymous}, {}", line("B")), "")),
            ("missing_name", "/panes/0/layers/0/name".to_owned())
        );
        let two = r#"{"layers": [{"mark": "line", "name": "X", "points": [{"time": "2026-03-02", "value": 1}, {"time": "2026-03-03", "value": 2}]}, {"mark": "line", "name": "Y", "points": [{"time": "2026-03-02", "value": 1}, {"time": "2026-03-03", "value": 2}]}]}"#;
        assert_eq!(
            render_err(&stacked(
                &format!("{four}, {two}")
                    .replacen(&line("L3"), "", 1)
                    .replace(",,", ","),
                ""
            )),
            ("too_many_layers", "/panes/3/layers/1/color".to_owned())
        );
        let late = line("Late")
            .replace("2026-03-02", "2026-03-10")
            .replace("2026-03-03", "2026-03-11");
        let zoomed = stacked(
            &format!("{}, {late}", line("Early")),
            r#", "zoomSteps": [{"label": "All", "from": "2026-03-01", "to": "2026-03-12"}, {"label": "Early", "from": "2026-03-01", "to": "2026-03-04"}]"#,
        );
        assert_eq!(
            render_err(&zoomed),
            ("zoom_out_of_range", "/zoomSteps/1".to_owned())
        );
        let squeezed = stacked(&tall.replace("11", "10"), r#", "height": 240"#);
        let output = render_ok(&squeezed);
        assert!(
            output
                .warnings
                .iter()
                .any(|warning| warning.code == "dense_chart"
                    && warning.path == "/panes/0/heightRatio"),
            "{:?}",
            output.warnings
        );
    }

    #[test]
    fn german_candles_and_panes_write_german_texts() {
        let spec = stacked(
            &format!("{}, {VOLUME_PANE}", candle_pane("")),
            r#", "locale": "de""#,
        );
        let html = html_ok(&spec);
        assert!(html.contains("Price (hohl: steigend, gefüllt: fallend)</text>"));
        assert!(html.contains(
            "<title>02.03.2026 – Price: Eröffnung 10,00; Hoch 12,00; Tief 9,00; Schluss 11,00</title>"
        ));
        assert!(
            html.contains(
                " 2 übereinanderliegende Teildiagramme auf einer Zeitachse: Price, Volume."
            )
        );
        assert!(html.contains(
            "Price: Eröffnung 10,00 am 02.03.2026, Schluss 9,00 am 04.03.2026, Veränderung \u{2212}1,00 (\u{2212}10,0\u{202f}%); Hoch 12,00 am 02.03.2026, Tief 8,00 am 03.03.2026."
        ), "{html}");
        assert!(html.contains("<th scope=\"col\">Price (Eröffnung)</th><th scope=\"col\">Price (Hoch)</th><th scope=\"col\">Price (Tief)</th><th scope=\"col\">Price (Schluss)</th>"));
        assert!(html.contains("<td>12\u{202f}%</td>"));
    }

    /// The specification with a `mobile` field added at the top level.
    fn with_mobile(specification: &str, mobile: &str) -> String {
        specification.replacen('{', &format!("{{\"mobile\": {mobile},"), 1)
    }

    fn mobile_svg(specification: &str) -> Result<super::RenderOutput, super::ChartError> {
        render_json(
            specification,
            RenderFormat::Svg,
            &RenderOptions {
                variant: Variant::Mobile,
                ..RenderOptions::default()
            },
        )
    }

    #[test]
    fn mobile_sizes_are_validated_with_their_paths() {
        for (mobile, path) in [
            (r#"{"width": 199}"#, "/mobile/width"),
            (r#"{"width": 601}"#, "/mobile/width"),
            (r#"{"width": 360, "height": 239}"#, "/mobile/height"),
            (r#"{"width": 360, "height": 1601}"#, "/mobile/height"),
            (r#"{"width": 360, "breakpoint": 319}"#, "/mobile/breakpoint"),
            (
                r#"{"width": 360, "breakpoint": 1601}"#,
                "/mobile/breakpoint",
            ),
        ] {
            let error = render_json(
                &with_mobile(GROUPED, mobile),
                RenderFormat::Html,
                &RenderOptions::default(),
            )
            .unwrap_err();
            assert_eq!(
                (error.code, error.path.as_str()),
                ("invalid_dimension", path)
            );
        }
        let missing = render_json(
            &with_mobile(GROUPED, r#"{"height": 360}"#),
            RenderFormat::Html,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!(
            (missing.code, missing.path.as_str()),
            ("invalid_spec", "/mobile")
        );
    }

    #[test]
    fn a_missing_mobile_field_keeps_the_default_id() {
        let plain = render_json(GROUPED, RenderFormat::Svg, &RenderOptions::default()).unwrap();
        let spec = ChartSpec::from_json(GROUPED).unwrap();
        assert!(spec.mobile.is_none());
        assert!(!serde_json::to_string(&spec).unwrap().contains("mobile"));
        assert!(plain.content.contains("id=\"chartlet-"));
    }

    #[test]
    fn html_carries_both_variants_behind_a_container_query() {
        let specification = with_mobile(GROUPED, r#"{"width": 360, "breakpoint": 700}"#);
        let html = render_json(
            &specification,
            RenderFormat::Html,
            &RenderOptions {
                id_prefix: Some("chart".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap()
        .content;
        assert!(html.starts_with(
            "<div class=\"chartlet-wrapper chartlet-responsive chartlet-bp-700\"><style>"
        ));
        assert!(html.contains("container-type:inline-size"));
        assert!(html.contains("@container (max-width:699px){.chartlet-bp-700 .chartlet-variant-desktop{display:none}.chartlet-bp-700 .chartlet-variant-mobile{display:block}}"));
        assert!(html.contains(".chartlet-variant-mobile{display:none}"));
        assert!(html.contains("<div class=\"chartlet-variant-desktop\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"450\""));
        assert!(html.contains("<div class=\"chartlet-variant-mobile\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"360\" height=\"360\""));
        assert!(html.contains("id=\"chart\""));
        assert!(html.contains("id=\"chart-m\""));
        assert!(html.contains("aria-labelledby=\"chart-m-title chart-m-description\""));
        // Every ID is unique across both variants.
        let ids = html
            .split(" id=\"")
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap())
            .collect::<Vec<_>>();
        let unique = ids.iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), unique.len(), "{ids:?}");
        // Caption, source, filter and data table appear once.
        for once in [
            "<figcaption>",
            "<table>",
            "<fieldset class=\"chartlet-filter\">",
        ] {
            assert_eq!(html.matches(once).count(), 1, "{once}");
        }
        // The series filter reaches the bars of both variants.
        assert!(html.contains(".chartlet-wrapper:has(.chartlet-filter input.series-1:not(:checked)) .chartlet-root [data-series=\"1\"]{display:none}"));
        assert_eq!(html.matches("<svg").count(), 2);
    }

    #[test]
    fn the_svg_profile_is_unchanged_by_a_mobile_variant() {
        let options = RenderOptions {
            id_prefix: Some("chart".to_owned()),
            ..RenderOptions::default()
        };
        let plain = render_json(GROUPED, RenderFormat::Svg, &options).unwrap();
        let responsive = render_json(
            &with_mobile(GROUPED, r#"{"width": 360}"#),
            RenderFormat::Svg,
            &options,
        )
        .unwrap();
        assert_eq!(plain.content, responsive.content);
    }

    #[test]
    fn the_mobile_variant_renders_alone_as_svg() {
        let output = mobile_svg(&with_mobile(GROUPED, r#"{"width": 320, "height": 400}"#))
            .unwrap()
            .content;
        assert!(output.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"320\" height=\"400\" viewBox=\"0 0 320 400\" id=\"chartlet-"));
        assert!(output.contains("-m-title\""));

        let error = mobile_svg(GROUPED).unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("missing_mobile", "/mobile")
        );

        let error = render_json(
            &with_mobile(GROUPED, r#"{"width": 360}"#),
            RenderFormat::Html,
            &RenderOptions {
                variant: Variant::Mobile,
                ..RenderOptions::default()
            },
        )
        .unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("option_not_supported", "/render/variant")
        );
    }

    #[test]
    fn the_print_variant_draws_the_chart_with_literal_colors() {
        let print = |specification: &str, format| {
            render_json(
                specification,
                format,
                &RenderOptions {
                    id_prefix: Some("costs".to_owned()),
                    variant: Variant::Print,
                    ..RenderOptions::default()
                },
            )
        };
        let desktop = render_json(
            GROUPED,
            RenderFormat::Svg,
            &RenderOptions {
                id_prefix: Some("costs".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap()
        .content;
        let light = print(GROUPED, RenderFormat::Svg).unwrap().content;
        let dark = print(
            &GROUPED.replacen('{', r#"{"theme": "dark","#, 1),
            RenderFormat::Svg,
        )
        .unwrap()
        .content;

        for svg in [&light, &dark] {
            assert!(svg.contains(" id=\"costs-p\""));
            assert!(!svg.contains("var("), "{svg}");
            assert!(!svg.contains(":has("), "{svg}");
            assert!(!svg.contains("currentColor"), "{svg}");
            assert!(!svg.contains("--chartlet"), "{svg}");
            assert!(svg.contains("font-family:Inter,ui-sans-serif,system-ui,sans-serif;"));
        }
        // The same scene as the chart on screen, only the stylesheet differs.
        let body = |svg: &str| svg[svg.find("</style>").unwrap()..].replace("costs-p", "costs");
        assert_eq!(body(&light), body(&desktop));
        assert!(light.contains("<style>#costs-p.chartlet-root{max-width:100%;height:auto;font-family:Inter,ui-sans-serif,system-ui,sans-serif;color:#172033}"));
        assert!(light.contains("#costs-p .chartlet-series-2{fill:#c2410c}"));
        assert!(dark.contains("#costs-p .chartlet-series-2{fill:#ff9c72}"));
        assert!(dark.contains("#costs-p .chartlet-background{fill:#0e131c}"));

        let error = print(GROUPED, RenderFormat::Html).unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("option_not_supported", "/render/variant")
        );
    }

    #[test]
    fn the_print_variant_resolves_a_declared_variable_color_to_the_text_color() {
        let svg = render_json(
            &TIME.replacen(
                r#""name": "Orders""#,
                r#""name": "Orders", "color": "var(--brand)""#,
                1,
            ),
            RenderFormat::Svg,
            &RenderOptions {
                variant: Variant::Print,
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(
            svg.content.contains(".chartlet-style-0{stroke:#172033}"),
            "{}",
            svg.content
        );
        assert!(!svg.content.contains("var("), "{}", svg.content);
        assert_eq!(
            svg.warnings
                .iter()
                .map(|warning| (warning.code, warning.path.as_str()))
                .collect::<Vec<_>>(),
            [("color_not_resolved", "/panes/0/layers/0/color")]
        );
    }

    #[test]
    fn zoom_panels_carry_both_variants_under_one_radio_group() {
        let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Quarterly revenue",
            "mobile": {"width": 360},
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
        let html = render_json(
            specification,
            RenderFormat::Html,
            &RenderOptions {
                id_prefix: Some("q".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap()
        .content;
        assert_eq!(
            html.matches("<fieldset class=\"chartlet-zoom\">").count(),
            1
        );
        assert_eq!(html.matches("type=\"radio\"").count(), 2);
        for id in ["q-z0", "q-z0-m", "q-z1", "q-z1-m"] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(html.contains("<div class=\"chartlet-panel chartlet-panel-1\"><div class=\"chartlet-variant-desktop\"><svg"));
        assert_eq!(html.matches("<table>").count(), 1);
    }

    #[test]
    fn warnings_of_the_mobile_variant_are_marked_and_deduplicated() {
        // A label that is shortened at both sizes is reported once; one that only the mobile
        // variant shortens is reported as coming from it.
        let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Revenue",
            "mobile": {"width": 280},
            "categories": ["An exceptionally long category label that never fits", "B"],
            "series": [
                {"name": "Direct", "values": [1, 2]},
                {"name": "A partner network with a long name", "values": [2, 1]}
            ]
        }"#;
        let desktop = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
            .unwrap()
            .warnings;
        let html = render_json(specification, RenderFormat::Html, &RenderOptions::default())
            .unwrap()
            .warnings;
        assert!(html.starts_with(&desktop));
        let added = &html[desktop.len()..];
        assert_ne!(added, []);
        for warning in added {
            assert!(
                warning.message.starts_with("mobile variant: "),
                "{warning:?}"
            );
            assert!(
                !desktop
                    .iter()
                    .any(|seen| (seen.code, &seen.path) == (warning.code, &warning.path)),
                "{warning:?}"
            );
        }
    }

    #[test]
    fn the_mobile_variant_is_checked_for_density_at_its_own_width() {
        let points = (0..400)
            .map(|index| {
                format!(
                    "{{\"time\": {}, \"value\": {index}}}",
                    1_772_323_200 + index * 3600
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let specification = format!(
            r#"{{"schemaVersion": 1, "type": "time", "title": "Load", "mobile": {{"width": 280}},
            "panes": [{{"layers": [{{"mark": "line", "points": [{points}]}}]}}]}}"#
        );
        let desktop = render_json(&specification, RenderFormat::Svg, &RenderOptions::default())
            .unwrap()
            .warnings;
        assert!(!desktop.iter().any(|warning| warning.code == "dense_chart"));
        let html = render_json(
            &specification,
            RenderFormat::Html,
            &RenderOptions::default(),
        )
        .unwrap()
        .warnings;
        assert!(html.iter().any(|warning| warning.code == "dense_chart"
            && warning.message.starts_with("mobile variant: ")));
    }

    #[test]
    fn the_topic_picker_switches_both_variants() {
        let specification = with_mobile(
            include_str!("../examples/topicmap-sample.json"),
            r#"{"width": 360}"#,
        );
        let html = render_json(
            &specification,
            RenderFormat::Html,
            &RenderOptions::default(),
        )
        .unwrap()
        .content;
        assert_eq!(
            html.matches("<fieldset class=\"chartlet-topic-picker\">")
                .count(),
            1
        );
        assert_eq!(html.matches("<svg").count(), 2);
        // The picker's rules name both variants' root IDs.
        let rule = ":checked) #{root} .chartlet-topic-area.chartlet-topic-0{";
        let desktop = html.split(" id=\"").nth(1).unwrap();
        let desktop = &desktop[..desktop.find('"').unwrap()];
        assert!(html.contains(&rule.replace("{root}", desktop)));
        assert!(html.contains(&rule.replace("{root}", &format!("{desktop}-m"))));
        assert!(html.starts_with("<div class=\"chartlet-wrapper chartlet-responsive"));
    }

    // --- Collapsed gaps ---

    /// Thursday 2026-03-05 to Wednesday 2026-03-11 without the weekend, as Unix seconds.
    const TRADING_DAYS: [i64; 5] = [
        1_772_668_800,
        1_772_755_200,
        1_773_014_400,
        1_773_100_800,
        1_773_187_200,
    ];

    /// A time chart with gaps collapsed: one line over `days`, then `extra` layers, then `tail`
    /// at the top level.
    fn collapsed(days: &[i64], extra: &str, tail: &str) -> String {
        let points = days
            .iter()
            .enumerate()
            .map(|(index, day)| format!(r#"{{"time": {day}, "value": {}}}"#, index + 1))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Trading days",
                "showValues": false,
                "timeAxis": {{"gaps": "collapse"}},
                "panes": [{{"layers": [{{"mark": "line", "points": [{points}]}}{extra}]}}]{tail}
            }}"#
        )
    }

    /// The horizontal centers of the line's point markers, in order.
    fn point_xs(svg: &str) -> Vec<f64> {
        svg.split("<circle cx=\"")
            .skip(1)
            .map(|rest| rest[..rest.find('"').unwrap()].parse().unwrap())
            .collect()
    }

    /// The labels and positions of the time ticks, in order.
    fn time_ticks(svg: &str) -> Vec<(String, f64)> {
        svg.split("<text x=\"")
            .skip(1)
            .filter(|rest| rest.contains("class=\"chartlet-tick\">2026"))
            .map(|rest| {
                let x = rest[..rest.find('"').unwrap()].parse().unwrap();
                let label = rest.split("class=\"chartlet-tick\">").nth(1).unwrap();
                (label[..label.find('<').unwrap()].to_owned(), x)
            })
            .collect()
    }

    #[test]
    fn collapsed_gaps_place_observations_at_equal_distances() {
        let svg = render_ok(&collapsed(&TRADING_DAYS, "", "")).content;
        let xs = point_xs(&svg);
        assert_eq!(xs.len(), 5);
        let step = xs[1] - xs[0];
        assert!(step > 0.0);
        for pair in xs.windows(2) {
            assert!((pair[1] - pair[0] - step).abs() < 0.01, "{xs:?}");
        }
        // The weekend takes no space: Friday and Monday are neighbours.
        assert!(svg.contains("<title>2026-03-06: 2</title>"));
        assert!(svg.contains("<title>2026-03-09: 3</title>"));
        // Daily ticks: the weekend's boundaries fall on Monday, which is labelled once.
        let labels: Vec<String> = time_ticks(&svg)
            .into_iter()
            .map(|(label, _)| label)
            .collect();
        assert_eq!(
            labels,
            [
                "2026-03-05",
                "2026-03-06",
                "2026-03-09",
                "2026-03-10",
                "2026-03-11"
            ]
        );

        // The same data with gaps shown leaves room for the weekend.
        let shown =
            render_ok(&collapsed(&TRADING_DAYS, "", "").replace("collapse", "show")).content;
        let xs = point_xs(&shown);
        assert!(xs[2] - xs[1] > 2.0 * (xs[1] - xs[0]));
    }

    #[test]
    fn collapsed_ticks_sit_on_the_first_observation_after_their_boundary() {
        // Five weeks of weekdays from Monday 2026-03-02, without Monday 2026-03-16.
        let monday: i64 = 1_772_409_600;
        let days: Vec<i64> = (0..33)
            .map(|day| monday + day * 86_400)
            .filter(|epoch| (epoch / 86_400 + 3).rem_euclid(7) < 5)
            .filter(|epoch| *epoch != monday + 14 * 86_400)
            .collect();
        let svg = render_ok(&collapsed(&days, "", "")).content;
        let ticks = time_ticks(&svg);
        let labels: Vec<&str> = ticks.iter().map(|(label, _)| label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "2026-03-02",
                "2026-03-09",
                "2026-03-17",
                "2026-03-23",
                "2026-03-30"
            ]
        );
        // The tick of the week without its Monday sits on the Tuesday's observation.
        let xs = point_xs(&svg);
        let tuesday = days
            .iter()
            .position(|epoch| *epoch == monday + 15 * 86_400)
            .unwrap();
        assert!((ticks[2].1 - xs[tuesday]).abs() < 0.01, "{ticks:?}");
    }

    #[test]
    fn collapsed_gaps_move_annotations_onto_observations() {
        let extra = r#", {"mark": "annotation", "label": "Rule", "time": "2026-03-07"},
            {"mark": "annotation", "label": "Event", "time": "2026-03-08", "value": 2, "shape": "square"},
            {"mark": "band", "label": "Zone", "from": "2026-03-07", "to": "2026-03-10T12:00"}"#;
        let output = render_ok(&collapsed(&TRADING_DAYS, extra, ""));
        let svg = output.content;
        let xs = point_xs(&svg);
        let (monday, tuesday) = (xs[2], xs[3]);
        // A reference line on Saturday stands on Monday, the next observation.
        assert!(svg.contains(&format!(
            "<polyline points=\"{monday},78 {monday},414\" class=\"chartlet-rule\">"
        )));
        // So does a point marker on Sunday.
        let marker = svg
            .split("class=\"chartlet-marker\"")
            .next()
            .and_then(|head| head.rsplit("<polyline points=\"").next())
            .unwrap();
        let corners: Vec<f64> = marker
            .split([' ', '"'])
            .filter_map(|pair| pair.split(',').next()?.parse().ok())
            .collect();
        let center = f64::midpoint(corners[0], corners[1]);
        assert!((center - monday).abs() < 0.01, "{marker}");
        // A zone covers the observations between its edges: from Monday to Tuesday.
        assert!(svg.contains(&format!(
            "<rect x=\"{monday}\" y=\"78\" width=\"{}\" height=\"336\" class=\"chartlet-zone\">",
            tuesday - monday
        )));
        // Tooltips and description keep the real times.
        assert!(svg.contains("<title>Rule: 2026-03-07</title>"));
        assert!(svg.contains("Reference lines: Rule at 2026-03-07."));
    }

    #[test]
    fn collapsed_gaps_refuse_times_without_an_observation() {
        for (extra, path) in [
            (
                r#", {"mark": "annotation", "label": "Rule", "time": "2026-03-04"}"#,
                "/panes/0/layers/1/time",
            ),
            (
                r#", {"mark": "annotation", "label": "Event", "time": "2026-03-12", "value": 2}"#,
                "/panes/0/layers/1/time",
            ),
            (
                r#", {"mark": "band", "label": "Weekend", "from": "2026-03-07", "to": "2026-03-08"}"#,
                "/panes/0/layers/1",
            ),
            (
                r#", {"mark": "band", "label": "Before", "from": "2026-03-01", "to": "2026-03-04"}"#,
                "/panes/0/layers/1",
            ),
        ] {
            assert_eq!(
                render_err(&collapsed(&TRADING_DAYS, extra, "")),
                ("time_out_of_range", path.to_owned()),
                "{extra}"
            );
        }
        // A zone reaching beyond the observations is cut at the first and the last of them.
        let wide =
            r#", {"mark": "band", "label": "All", "from": "2026-03-01", "to": "2026-03-20"}"#;
        let svg = render_ok(&collapsed(&TRADING_DAYS, wide, "")).content;
        let xs = point_xs(&svg);
        assert!(svg.contains(&format!(
            "<rect x=\"{}\" y=\"78\" width=\"{}\"",
            xs[0],
            xs[4] - xs[0]
        )));
        // With the distances kept, the same times are fine.
        let rule = r#", {"mark": "annotation", "label": "Rule", "time": "2026-03-04"}"#;
        render_ok(&collapsed(&TRADING_DAYS, rule, "").replace("collapse", "show"));
    }

    #[test]
    fn collapsed_zoom_windows_close_their_gaps_too() {
        let zones = r#", {"mark": "band", "label": "Early", "from": "2026-03-05", "to": "2026-03-07T12:00"}"#;
        let zoom = r#", "zoomSteps": [
            {"label": "All", "from": "2026-03-05", "to": "2026-03-11"},
            {"label": "Weekend on", "from": "2026-03-07", "to": "2026-03-11"}
        ]"#;
        let html = html_ok(&collapsed(&TRADING_DAYS, zones, zoom));
        let panel = |index: usize| {
            html.split(&format!("chartlet-panel chartlet-panel-{index}"))
                .nth(1)
                .and_then(|panel| panel.split("</div>").next())
                .unwrap()
                .to_owned()
        };
        let (all, late) = (panel(0), panel(1));
        assert_eq!(point_xs(&all).len(), 5);
        let xs = point_xs(&late);
        assert_eq!(xs.len(), 3);
        assert!(((xs[2] - xs[1]) - (xs[1] - xs[0])).abs() < 0.01, "{xs:?}");
        // The zone keeps Thursday and Friday in the full window; the late window starts on the
        // Saturday, so its share of the zone holds no observation and is left out.
        assert!(all.contains("class=\"chartlet-zone\""));
        assert!(!late.contains("class=\"chartlet-zone\""));
    }

    #[test]
    fn collapsed_gaps_are_named_in_the_description() {
        let spec = collapsed(&TRADING_DAYS, "", "");
        assert!(
            render_ok(&spec)
                .content
                .contains("Lowest: 1 (2026-03-05). Gaps in time are closed up.</desc>")
        );
        let html = html_ok(&german(&spec, "time"));
        assert!(
            html.contains("Zeiträume ohne Beobachtung sind ausgelassen."),
            "{html}"
        );
        let shown = render_ok(&spec.replace("collapse", "show")).content;
        assert!(!shown.contains("Gaps in time"));
    }

    #[test]
    fn collapsed_small_multiples_share_their_slots() {
        let points = |days: &[i64]| {
            days.iter()
                .map(|day| format!(r#"{{"time": {day}, "value": 1}}"#))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let spec = format!(
            r#"{{
                "schemaVersion": 1,
                "type": "multiples",
                "title": "Desks",
                "timeAxis": {{"gaps": "collapse"}},
                "panes": [
                    {{"title": "A", "layers": [{{"mark": "line", "points": [{}]}}]}},
                    {{"title": "B", "layers": [{{"mark": "line", "points": [{}]}}]}}
                ]
            }}"#,
            points(&TRADING_DAYS[..3]),
            points(&TRADING_DAYS[2..]),
        );
        let svg = render_ok(&spec).content;
        let xs = point_xs(&svg);
        assert_eq!(xs.len(), 6);
        // Panel B, one grid cell of (800 - 2 × 16) / 2 pixels to the right, starts on the third
        // of the five shared slots, where panel A ends, and keeps the same distances.
        let cell = 384.0;
        let step = xs[1] - xs[0];
        assert!((xs[3] - cell - xs[2]).abs() < 0.01, "{xs:?}");
        assert!((xs[5] - xs[4] - step).abs() < 0.01, "{xs:?}");
    }
}
