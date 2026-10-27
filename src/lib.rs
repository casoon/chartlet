mod advice;
mod architecture;
mod boxplot;
mod calendar;
mod classes;
mod color;
mod contrast;
mod cycle;
mod dendrogram;
mod describe;
mod diagram;
mod diverging;
mod draw;
mod error;
mod fishbone;
mod flow;
mod heat;
mod heatmap;
mod inspect;
mod journey;
mod kanban;
mod layout;
mod metrics;
mod network;
mod ohlc;
mod parliament;
mod parts;
#[cfg(feature = "png")]
mod png;
pub mod qr;
mod rangebar;
mod readability;
mod reference;
mod render;
mod roles;
mod sankey;
mod scatter;
mod scene;
mod sequence;
mod sha256;
mod slope;
mod social;
mod sort;
mod spec;
mod state;
mod storymap;
mod stripes;
mod structure;
mod survival;
mod symbols;
mod text;
mod theme;
mod time;
mod timeline;
mod tree;
mod treemap;
mod typography;
mod valuemap;
mod venn;
mod waffle;
mod waterfall;

pub use advice::{Help, PatchOperation, Remedy, RemedyKind, advise, advise_json};
use describe::automatic_description;
pub use error::{ChartError, ChartWarning};
pub use inspect::{INSPECTIONS, Inspection, Severity, inspect_json};
pub use metrics::{BuiltinMetrics, TextMetrics};
pub use sha256::sha256;
pub use spec::{
    ArchitectureSpec, AxisScale, BoundarySpec, BoxDisplay, BoxSpec, BranchSpec, CalendarDay,
    CalendarLayout, CalendarSpec, CategoryAxisSpec, CategorySpec, ChartSpec, ChartType, ClassKind,
    ClassSpec, ClassesSpec, ComponentKind, ComponentSpec, ConnectionDirection, ConnectionSpec,
    Curve, CycleHubSpec, CycleLabels, CycleSpec, CycleStepSpec, Dash, DataPoint,
    DiagramOrientation, DiagramRole, DiagramSpacing, DistancesSpec, FishboneBoneSpec,
    FishboneCauseSpec, FishboneSpec, FishboneSubcauseSpec, FlowEdgeSpec, FlowNodeSpec, FlowSpec,
    FragmentKind, FragmentSpec, Gaps, GroupSpec, HeatmapCellSpec, HeatmapOrder, HeatmapScale,
    HeatmapSpec, InstanceSpec, InterfaceSpec, JourneyDisplay, JourneySpec, JourneyStageSpec,
    KanbanCardSpec, KanbanColumnSpec, KanbanSpec, LaneSpec, LayerSpec, LegendPlacement, Linkage,
    MainPath, Mark, MarkerSpec, MessageKind, MessageSpec, MobileSpec, NetworkEdgeSpec,
    NetworkLayout, NetworkNodeSpec, NetworkOrder, NetworkSpec, NodeKind, NoteSpec, OhlcPoint,
    Orientation, PackageSpec, PaneSpec, ParliamentSpec, PartSpec, ParticipantKind, ParticipantSpec,
    PartySpec, PerspectiveSpec, PortSide, RangeDisplay, RangeSort, RangeSpec, ReferenceSpec,
    RelationKind, RelationSpec, SankeyLinkSpec, SankeyNodeSpec, SankeyOrder, SankeySpec,
    ScatterAxis, ScatterLineSpec, ScatterPointSpec, ScatterQuadrantLabels, ScatterQuadrantsSpec,
    ScatterSpec, ScatterTickSpec, SequenceSpec, SeriesSpec, Shape, SizePreset, Stack, StateKind,
    StateNodeSpec, StateSpec, StepKind, StepSpec, StorymapActivitySpec, StorymapSpec,
    StorymapStorySpec, StorymapTaskSpec, StripesSpec, Stroke, StrokeStyle, SurvivalGroupSpec,
    SurvivalObservationSpec, SurvivalSpec, Theme, TimeAxisKind, TimeAxisSpec, TimePoint,
    TimePrecision, TimelineItemSpec, TimelineSpec, Tooltips, TransitionSpec, TreeLayout,
    TreeNodeKind, TreeNodeSpec, TreeSpec, TreemapItemSpec, TreemapSpec, ValueAxisSpec, ValueFormat,
    ValuemapComponentSpec, ValuemapDependencySpec, ValuemapMoveSpec, ValuemapSpec, VennRegionSpec,
    VennSpec, ViewSpec, ViewStepSpec, ViolinScale, WafflePartSpec, WaffleSpec, WaterfallSpec,
    ZoomBound, ZoomStep,
};
pub use theme::{apply_theme, bundled_theme};
pub use typography::{FontMetrics, TypographySpec, embed_font};

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
    /// layer draws, the specification ID of every diagram node of a flow chart, state,
    /// architecture or class diagram, and the unformatted values on the data table, or in a
    /// JSON data block in the SVG profile. The print variant carries none.
    pub hooks: bool,
    /// The `key` of the view of an architecture model to render; without one, its default view.
    /// Every variant renders the same view.
    pub view: Option<String>,
    /// The `key` of the perspective of an architecture diagram to show, in place of the one the
    /// model or the rendered view selects. Every variant shows the same perspective.
    pub perspective: Option<String>,
    /// Renders every view of an architecture model on one HTML page instead of one view: a list
    /// of links to the views, the default view first, and each view as a section of its own
    /// with its figure, description and data table. Only for the HTML profile and the desktop
    /// variant, and not together with `view`.
    pub all_views: bool,
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
    /// The `key` of the view of an architecture model that was rendered; `None` for a
    /// specification without views. `spec_hash` is the hash of the whole model.
    pub view: Option<String>,
    /// The keys of the views of an architecture model shown on one page with
    /// [`RenderOptions::all_views`], in the order of the page; empty otherwise.
    pub views: Vec<String>,
    /// The `key` of the perspective of an architecture diagram that was shown; `None` without
    /// one.
    pub perspective: Option<String>,
    /// The theme a host applied to the specification before rendering: the name of a bundled
    /// theme or the path of a theme file. The compiler renders the themed specification and
    /// does not know it; the CLI sets it. `None` without one.
    pub theme: Option<String>,
    pub warnings: Vec<ChartWarning>,
}

impl Manifest {
    /// The manifest as a pretty-printed JSON object, its keys in a fixed order:
    /// `chartlet`, `schemaVersion`, `specHash`, `outputHash`, `format`, `variant`, `idPrefix`,
    /// `view` (only when a view was rendered), `views` (only when all views were rendered on one
    /// page), `perspective` (only when one was shown), `theme` (only when a host applied one),
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
            #[serde(skip_serializing_if = "Option::is_none")]
            view: Option<&'a str>,
            #[serde(skip_serializing_if = "<[String]>::is_empty")]
            views: &'a [String],
            #[serde(skip_serializing_if = "Option::is_none")]
            perspective: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            theme: Option<&'a str>,
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
            view: self.view.as_deref(),
            views: &self.views,
            perspective: self.perspective.as_deref(),
            theme: self.theme.as_deref(),
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

/// Validates a chart specification and returns its text alternative; that of the default view
/// for an architecture model with views (see [`ChartSpec::view`] for another view).
///
/// # Errors
///
/// Returns a structured error when the specification is invalid.
pub fn text_alternative(spec: &ChartSpec) -> Result<TextAlternative, ChartError> {
    spec.validate()?;
    if let Some(view) = spec.resolve_view(None)? {
        return text_alternative(&view.spec);
    }
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

/// The family a font file is used under: `typography.family`.
fn family_of(spec: &ChartSpec) -> Result<&str, ChartError> {
    spec.typography
        .as_ref()
        .and_then(|typography| typography.family.as_deref())
        .ok_or_else(|| {
            ChartError::new(
                "invalid_font",
                "/typography/family",
                "a font file needs the name it is used under: add \"typography\": { \"family\": \"Your Font\" }",
            )
        })
}

/// Renders with the advance widths of `font`, a TrueType or OpenType file the caller supplies:
/// the layout measures with it instead of the built-in metrics, and `font_glyphs_missing` names
/// the characters it has no glyph for. The output still only names the family; see [`embed_font`]
/// to carry the file in an SVG. Nothing is fetched or looked up on the machine.
///
/// # Errors
///
/// As [`render_with_metrics`], and `invalid_font` for a file that is not a plain font or a
/// specification without `typography.family`.
pub fn render_with_font(
    spec: &ChartSpec,
    format: RenderFormat,
    options: &RenderOptions,
    font: &[u8],
) -> Result<RenderOutput, ChartError> {
    family_of(spec)?;
    let metrics = FontMetrics::from_ttf(font)?;
    let mut output = render_with_metrics(spec, format, options, &metrics)?;
    output
        .warnings
        .retain(|warning| warning.code != "font_not_embedded");
    output
        .warnings
        .extend(metrics.missing_glyphs(&output.content));
    Ok(output)
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
    match spec
        .typography
        .as_ref()
        .map(typography::TypographySpec::scale)
    {
        Some(scale) if (scale - 1.0).abs() > f64::EPSILON => {
            render_with_metrics(spec, format, options, &typography::ScaledMetrics(scale))
        }
        _ => render_with_metrics(spec, format, options, &BuiltinMetrics),
    }
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
    // A perspective asked for takes the place of the one the model and its views select; the
    // manifest still hashes the model as given.
    let model = spec;
    let selected;
    let spec = match &options.perspective {
        Some(key) => {
            selected = spec.with_perspective(key)?;
            &selected
        }
        None => spec,
    };
    if options.all_views {
        return render_all_views((model, spec), format, options, metrics);
    }
    // A model with views renders one of them, resolved to a diagram of its own; diagnostics
    // point into the model.
    let view = if spec.has_views() || options.view.is_some() {
        spec.validate()?;
        spec.resolve_view(options.view.as_deref())?
    } else {
        None
    };
    let shown = view.as_ref().map_or(spec, |view| &view.spec);
    let mut output =
        render_content(shown, format, options, metrics).map_err(|error| match &view {
            Some(view) => ChartError::new(error.code, view.model_path(&error.path), error.message),
            None => error,
        })?;
    if let Some(view) = &view {
        for warning in &mut output.warnings {
            warning.path = view.model_path(&warning.path);
        }
    }
    if options.manifest {
        let id_prefix = match &options.id_prefix {
            Some(id_prefix) => id_prefix.clone(),
            None => default_id_prefix(shown)?,
        };
        let perspective = shown
            .architecture
            .as_ref()
            .and_then(|architecture| architecture.perspective.clone());
        output.manifest = Some(Manifest {
            view: view.map(|view| view.key),
            perspective,
            ..manifest(model, format, options, &output, id_prefix)?
        });
    }
    Ok(output)
}

/// The manifest of `output`, rendered from `spec`, without a view or perspective.
fn manifest(
    spec: &ChartSpec,
    format: RenderFormat,
    options: &RenderOptions,
    output: &RenderOutput,
    id_prefix: String,
) -> Result<Manifest, ChartError> {
    Ok(Manifest {
        chartlet: env!("CARGO_PKG_VERSION"),
        schema_version: spec.schema_version,
        spec_hash: format!("sha256:{}", sha256::hex(&sha256(&canonical_json(spec)?))),
        output_hash: format!("sha256:{}", sha256::hex(&sha256(output.content.as_bytes()))),
        format,
        variant: options.variant,
        id_prefix,
        view: None,
        views: Vec::new(),
        perspective: None,
        theme: None,
        warnings: output.warnings.clone(),
    })
}

/// Every view of an architecture model on one HTML page, the default view first. Each view is a
/// section with the figure of its own HTML export, under the ID prefix of the page followed by
/// `-v` and the view's place on the page from 1; diagnostics point into the model. The model is
/// given as it came, whose hash the manifest carries, and as rendered, with the perspective
/// asked for.
fn render_all_views(
    (model, spec): (&ChartSpec, &ChartSpec),
    format: RenderFormat,
    options: &RenderOptions,
    metrics: &impl TextMetrics,
) -> Result<RenderOutput, ChartError> {
    let refuse = |message: &str| {
        Err(ChartError::new(
            "option_not_supported",
            "/render/allViews",
            message,
        ))
    };
    if format == RenderFormat::Svg {
        return refuse(
            "all views share one HTML page; render each view as an SVG of its own by its key",
        );
    }
    if options.view.is_some() {
        return refuse("all views are shown; leave out the view");
    }
    if options.variant != Variant::Desktop {
        return refuse("the page of all views carries the desktop and mobile variants of each");
    }
    spec.validate()?;
    if !spec.has_views() {
        return refuse("the specification has no views");
    }
    let page_prefix = resolve_id_prefix(spec, options)?;
    let mut views = vec![
        spec.resolve_view(None)?
            .expect("a model with views has a default"),
    ];
    for key in spec.view_keys() {
        if key != views[0].key {
            views.push(
                spec.resolve_view(Some(key))?
                    .expect("the key is that of a view"),
            );
        }
    }
    let mut warnings = Vec::new();
    let mut sections = Vec::new();
    for (number, view) in views.iter().enumerate() {
        let id = format!("{page_prefix}-v{}", number + 1);
        let output =
            render_content_as(&view.spec, format, options, &id, metrics).map_err(|error| {
                ChartError::new(error.code, view.model_path(&error.path), error.message)
            })?;
        warnings.extend(output.warnings.into_iter().map(|warning| ChartWarning {
            path: view.model_path(&warning.path),
            ..warning
        }));
        sections.push(render::ViewSection {
            id,
            title: view.spec.title.clone(),
            content: output.content,
        });
    }
    dedupe_warnings(&mut warnings);
    let mut output = RenderOutput {
        content: render::html_views(&sections, spec, &page_prefix),
        warnings,
        manifest: None,
    };
    if options.manifest {
        output.manifest = Some(Manifest {
            views: views.into_iter().map(|view| view.key).collect(),
            perspective: options.perspective.clone(),
            ..manifest(model, format, options, &output, page_prefix)?
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
    let id_prefix = resolve_id_prefix(spec, options)?;
    render_content_as(spec, format, options, &id_prefix, metrics)
}

/// Renders `spec` with its IDs under `id_prefix`.
#[allow(clippy::too_many_lines)]
fn render_content_as(
    spec: &ChartSpec,
    format: RenderFormat,
    options: &RenderOptions,
    id_prefix: &str,
    metrics: &impl TextMetrics,
) -> Result<RenderOutput, ChartError> {
    let mut warnings = spec.validate()?;
    warnings.extend(
        spec.typography
            .as_ref()
            .and_then(typography::TypographySpec::disclosure),
    );
    let id_prefix = id_prefix.to_owned();

    let mobile = spec.mobile_variant();
    let mobile_prefix = |prefix: &str| format!("{prefix}-m");
    let hooks = hook_mode(options.hooks, format);
    let parts = |spec: &ChartSpec, prefix: &str, warnings: &mut Vec<ChartWarning>| {
        render_panel_parts(
            spec,
            format,
            prefix,
            options.styles.into(),
            hooks,
            metrics,
            warnings,
        )
    };
    let panel = |spec: &ChartSpec, prefix: &str, warnings: &mut Vec<ChartWarning>| {
        parts(spec, prefix, warnings).svg
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
                stages: Vec::new(),
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

    let desktop = parts(spec, &id_prefix, &mut warnings);
    let content = match format {
        RenderFormat::Svg => desktop.svg,
        RenderFormat::Html => {
            let mobile = mobile
                .as_ref()
                .map(|mobile| parts(mobile, &mobile_prefix(&id_prefix), &mut mobile_warnings));
            merge_mobile_warnings(&mut warnings, mobile_warnings);
            let stages = stages(&desktop, mobile.as_ref());
            render::html(
                render::Panel {
                    label: String::new(),
                    svg: desktop.svg,
                    mobile: mobile.map(|mobile| mobile.svg),
                    stages,
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
    /// 1200 × 630 canvas, [`Variant::Mobile`] the mobile layout. [`Variant::Desktop`] is taken as
    /// [`Variant::Print`]: a PNG needs the resolved stylesheet.
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
/// the mobile variant is asked for and the specification has none.
#[cfg(feature = "png")]
pub fn render_png(spec: &ChartSpec, options: &PngOptions) -> Result<PngOutput, ChartError> {
    render_png_as(spec, *options, None)
}

/// [`render_png`] with a font file of the caller's: the text is measured with it and rasterized
/// with it, under the family the specification names in `typography.family`. The font is the
/// caller's to license; chartlet embeds nothing it was not given.
///
/// # Errors
///
/// As [`render_png`], and `invalid_font` when the file is not a plain TrueType or OpenType font
/// or the specification names no `typography.family`.
#[cfg(feature = "png")]
pub fn render_png_with_font(
    spec: &ChartSpec,
    options: &PngOptions,
    font: &[u8],
) -> Result<PngOutput, ChartError> {
    render_png_as(spec, *options, Some(font))
}

#[cfg(feature = "png")]
#[allow(clippy::too_many_lines)]
fn render_png_as(
    spec: &ChartSpec,
    options: PngOptions,
    font: Option<&[u8]>,
) -> Result<PngOutput, ChartError> {
    if !(0.25..=png::MAX_SCALE).contains(&options.scale) {
        return Err(ChartError::new(
            "invalid_scale",
            "/render/scale",
            "scale must be between 0.25 and 4",
        ));
    }
    // The mobile variant is rasterized like the chart itself: its layout with the resolved
    // stylesheet of the print variant.
    let mobile;
    let (spec, variant) = match options.variant {
        Variant::Desktop | Variant::Print => (spec, Variant::Print),
        Variant::Social => (spec, Variant::Social),
        Variant::Mobile => {
            spec.validate()?;
            mobile = spec.mobile_variant().ok_or_else(|| {
                ChartError::new(
                    "missing_mobile",
                    "/mobile",
                    "the specification has no mobile variant; add \"mobile\": { \"width\": 360 }",
                )
            })?;
            (&mobile, Variant::Print)
        }
    };
    let render_options = RenderOptions {
        variant,
        ..RenderOptions::default()
    };
    let (output, face) = match font {
        Some(font) => (
            render_with_font(spec, RenderFormat::Svg, &render_options, font)?,
            Some((family_of(spec)?, font)),
        ),
        None => (render(spec, RenderFormat::Svg, &render_options)?, None),
    };
    let mut warnings = output.warnings;
    if let Some(warning) = png::missing_glyphs(&output.content) {
        warnings.push(warning);
    }
    Ok(PngOutput {
        png: png::rasterize(&output.content, options.scale, face)?,
        warnings,
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
    render_panel_parts(spec, format, id_prefix, styles, hooks, metrics, warnings).svg
}

/// The stages of a diagram whose nodes can be focused, desktop first; none for any other chart.
fn stages(desktop: &PanelParts, mobile: Option<&PanelParts>) -> Vec<render::Stage> {
    if desktop.stage.hotspots.is_empty() {
        return Vec::new();
    }
    std::iter::once(desktop)
        .chain(mobile)
        .map(|parts| parts.stage.clone())
        .collect()
}

/// A chart laid out and serialized, with what the HTML profile needs to focus its nodes.
struct PanelParts {
    svg: String,
    stage: render::Stage,
}

/// [`render_panel`], keeping where the nodes of a diagram stand.
fn render_panel_parts(
    spec: &ChartSpec,
    format: RenderFormat,
    id_prefix: &str,
    styles: render::StyleMode,
    hooks: render::Hooks,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> PanelParts {
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
    let hotspots = scene
        .elements
        .iter()
        .filter_map(|element| match element {
            scene::Element::Hotspot(hotspot) => Some(hotspot.clone()),
            _ => None,
        })
        .collect();
    let svg = render::svg(&scene, spec, &description, id_prefix, styles, hooks);
    // The roles a diagram draws are known only once it is drawn.
    warnings.extend(contrast::role_warnings(spec, &svg));
    PanelParts {
        svg,
        stage: render::Stage {
            root: id_prefix.to_owned(),
            width: scene.width,
            height: scene.height,
            hotspots,
        },
    }
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

/// The ID prefix a render uses: the one passed in, checked, or the one derived from the
/// specification.
fn resolve_id_prefix(spec: &ChartSpec, options: &RenderOptions) -> Result<String, ChartError> {
    match &options.id_prefix {
        Some(id_prefix) => {
            validate_id_prefix(id_prefix)?;
            Ok(id_prefix.clone())
        }
        None => default_id_prefix(spec),
    }
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
mod tests;
