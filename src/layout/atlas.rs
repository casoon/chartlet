use super::{PLOT_MARGIN, axis::format_value, count, topicmap::push_sea};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Circle, Element, Polyline, Scene, Text, TextAnchor, TextStyle},
    spec::{AtlasSpec, ChartSpec, NumberStyle},
};

/// One class per realm. `atlas` charts are capped at eight realms, so this covers every one of
/// them, and a host page can address a whole landscape without counting polygons.
pub(crate) const REALM_CLASSES: [&str; 8] = [
    "chartlet-atlas-realm-0",
    "chartlet-atlas-realm-1",
    "chartlet-atlas-realm-2",
    "chartlet-atlas-realm-3",
    "chartlet-atlas-realm-4",
    "chartlet-atlas-realm-5",
    "chartlet-atlas-realm-6",
    "chartlet-atlas-realm-7",
];

/// The knowledge landscape, built up over the steps in `plan/09-wissenslandschaft-atlas.md`.
///
/// Step 6 of 8: the landscape is complete — realms, regions, coast, contour lines, every declared
/// place, and names set where their area has room for them.
pub(super) fn layout_atlas(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let atlas = spec
        .atlas
        .as_ref()
        .expect("validated atlas charts carry an atlas block");
    let margin = f64::from(PLOT_MARGIN);
    let top = 78.0;
    let plot = (
        margin,
        top,
        f64::from(spec.width) - margin * 2.0,
        f64::from(spec.height) - top - margin,
    );

    let landscape = crate::atlas::landscape(atlas, plot);
    let mut elements = Vec::new();
    push_sea(&mut elements, plot, false);
    push_landscape(&mut elements, atlas, &landscape, spec.number_style());

    push_places(&mut elements, &landscape);
    push_names(&mut elements, atlas, &landscape, metrics, warnings);

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// How large a place is drawn, before its own weight is taken into account.
const PLACE_RADIUS: f64 = 2.2;

/// How far an outline may be pulled straight, as a fraction of a cell. A quarter of a cell is
/// under what the grid could have resolved anyway, and it halves the points a map costs.
const OUTLINE_TOLERANCE: f64 = 0.25;

/// A name already on the map, as the box it takes up.
struct Taken {
    x: f64,
    y: f64,
    half_width: f64,
    half_height: f64,
}

fn clear_of(taken: &[Taken], want: &Taken) -> bool {
    taken.iter().all(|other| {
        (want.x - other.x).abs() >= want.half_width + other.half_width
            || (want.y - other.y).abs() >= want.half_height + other.half_height
    })
}

/// The places, each on the ground of its own region.
fn push_places(elements: &mut Vec<Element>, landscape: &crate::atlas::Landscape) {
    for spot in &landscape.spots {
        let detail = spot
            .place
            .tooltip
            .as_ref()
            .map(|line| format!("{} · {line}", spot.place.label));
        elements.push(Element::Circle(Circle {
            cx: spot.x,
            cy: spot.y,
            radius: PLACE_RADIUS * spot.place.weight.sqrt(),
            class: "chartlet-atlas-place",
            topic: Some(spot.region),
            series_index: None,
            style_index: None,
            tooltip: Some(detail.unwrap_or_else(|| spot.place.label.clone())),
        }));
    }
}

/// Writes one name where its area has the most room and nothing else is written yet.
///
/// The room comes from the distance transform: the point furthest from anything that is not this
/// area is the point where a name covers the least of it. If the first such point is taken, there
/// are three more behind it; if the name will not fit any of them at a readable size, it is left
/// off and said so, because a name spilling over a border is worse than no name.
fn push_name(
    elements: &mut Vec<Element>,
    taken: &mut Vec<Taken>,
    field: &crate::atlas::Field,
    name: (&str, &'static str, Option<usize>),
    room: (f64, f64),
    inside: &dyn Fn(usize, usize) -> bool,
    metrics: &impl TextMetrics,
) -> bool {
    let (text, class, topic) = name;
    let (smallest, largest) = room;
    let depth = crate::contour::depth(field.columns, field.rows, inside);
    let apart = (field.columns / 10).max(3);

    for (column, row, deep) in crate::contour::roomiest(&depth, field.columns, apart, 4) {
        let reach = f64::from(deep) * field.cell;
        let (x, y) = field.centre(column, row);
        let mut size = (reach * 0.7).clamp(smallest, largest);
        while size > smallest && metrics.width(text, size) > reach * 1.8 {
            size -= 1.0;
        }
        if metrics.width(text, size) > reach * 1.8 {
            continue;
        }
        let want = Taken {
            x,
            y,
            half_width: metrics.width(text, size) / 2.0 + 2.0,
            half_height: size * 0.6,
        };
        if !clear_of(taken, &want) {
            continue;
        }
        elements.push(Element::StyledText(
            Text {
                x,
                y: y + size * 0.34,
                class,
                anchor: TextAnchor::Middle,
                content: text.to_owned(),
            },
            TextStyle {
                size: Some(size),
                topic,
            },
        ));
        taken.push(want);
        return true;
    }
    false
}

/// All the names: realms first, because they name the whole landscape, then the regions with the
/// most room to spare, so that the ones with least room are asked last.
fn push_names(
    elements: &mut Vec<Element>,
    atlas: &AtlasSpec,
    landscape: &crate::atlas::Landscape,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let field = &landscape.field;
    let mut taken: Vec<Taken> = Vec::new();

    for (realm, entry) in atlas.realms.iter().enumerate() {
        push_name(
            elements,
            &mut taken,
            field,
            (&entry.label, "chartlet-atlas-realm-label", None),
            (12.0, 32.0),
            &|column, row| {
                field
                    .at(column, row)
                    .is_some_and(|region| landscape.sites[region].realm == realm)
            },
            metrics,
        );
    }

    let mut held = vec![0usize; landscape.sites.len()];
    for owner in field.owner.iter().flatten() {
        held[*owner] += 1;
    }
    let mut order: Vec<usize> = (0..landscape.sites.len()).collect();
    crate::sort::by(&mut order, |a, b| held[*b].cmp(&held[*a]).then(a.cmp(b)));

    let ranges = crate::atlas::realm_ranges(atlas);
    for region in order {
        let site = &landscape.sites[region];
        let written = push_name(
            elements,
            &mut taken,
            field,
            (&site.region.label, "chartlet-atlas-label", Some(region)),
            (9.0, 19.0),
            &|column, row| field.at(column, row) == Some(region),
            metrics,
        );
        if !written {
            let realm = site.realm;
            warnings.push(ChartWarning::new(
                "label_does_not_fit",
                format!(
                    "/atlas/realms/{realm}/regions/{}/label",
                    region - ranges[realm].start
                ),
                format!(
                    "{:?} has no room for its own name at this size; the area keeps its tooltip",
                    site.region.label
                ),
            ));
        }
    }

    let declared: usize = atlas
        .realms
        .iter()
        .flat_map(|realm| &realm.regions)
        .map(|region| region.places.len())
        .sum();
    if landscape.spots.len() < declared {
        warnings.push(ChartWarning::new(
            "places_did_not_fit",
            "/atlas/realms",
            format!(
                "{} of {declared} places {} drawn; a region has more places than it has ground",
                landscape.spots.len(),
                if landscape.spots.len() == 1 {
                    "was"
                } else {
                    "were"
                }
            ),
        ));
    }
}

/// The landscape as outlines: the realms as shapes, the regions as the finer divisions inside
/// them, and the coast around all of it.
///
/// Every shape comes from the same grid, so they meet exactly. Order is what makes them readable:
/// the realms fill, the region borders are drawn over that fill, and the coast goes on top of
/// both — a coastline that a border crosses stops being a coastline.
fn push_landscape(
    elements: &mut Vec<Element>,
    atlas: &AtlasSpec,
    landscape: &crate::atlas::Landscape,
    style: NumberStyle,
) {
    let field = &landscape.field;
    let corner = |(column, row): (usize, usize)| {
        (
            field.origin.0 - field.cell / 2.0 + count(column) * field.cell,
            field.origin.1 - field.cell / 2.0 + count(row) * field.cell,
        )
    };
    let shapes = |inside: &dyn Fn(usize, usize) -> bool| -> Vec<Vec<(f64, f64)>> {
        crate::contour::rings(field.columns, field.rows, inside)
            .iter()
            .map(|ring| {
                let points: Vec<(f64, f64)> = ring.iter().map(|at| corner(*at)).collect();
                let mut drawn = crate::contour::simplify(
                    &crate::contour::smooth(&crate::contour::straighten(&points)),
                    field.cell * OUTLINE_TOLERANCE,
                );
                // A polyline does not close itself, and these are areas.
                if let Some(first) = drawn.first().copied() {
                    drawn.push(first);
                }
                drawn
            })
            .collect()
    };

    for (realm, entry) in atlas.realms.iter().enumerate() {
        for points in shapes(&|column, row| {
            field
                .at(column, row)
                .is_some_and(|region| landscape.sites[region].realm == realm)
        }) {
            elements.push(Element::Polyline(Polyline {
                points,
                class: REALM_CLASSES[realm],
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: entry.tooltip.clone().or_else(|| Some(entry.label.clone())),
            }));
        }
    }

    // Contour lines between the realm fills and the borders: they are texture, and texture that
    // crosses a border reads as a border of its own.
    if atlas.contours {
        for level in &landscape.relief.levels {
            for points in shapes(&|column, row| {
                let cell = row * field.columns + column;
                field.owner[cell].is_some() && landscape.relief.height[cell] >= *level
            }) {
                elements.push(Element::Polyline(Polyline {
                    points,
                    class: "chartlet-atlas-contour",
                    topic: None,
                    series_index: None,
                    style_index: None,
                    tooltip: None,
                }));
            }
        }
    }

    for (region, site) in landscape.sites.iter().enumerate() {
        for points in shapes(&|column, row| field.at(column, row) == Some(region)) {
            elements.push(Element::Polyline(Polyline {
                points,
                class: "chartlet-atlas-region",
                topic: Some(region),
                series_index: None,
                style_index: None,
                tooltip: Some(site.region.tooltip.clone().unwrap_or_else(|| {
                    format!(
                        "{}: {}",
                        site.region.label,
                        format_value(site.region.value, style)
                    )
                })),
            }));
        }
    }

    for points in shapes(&|column, row| field.at(column, row).is_some()) {
        elements.push(Element::Polyline(Polyline {
            points,
            class: "chartlet-atlas-coast",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}
