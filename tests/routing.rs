//! Measures the routing and the geometry of every diagram example: how often edges cross, how
//! long they are altogether, how many run closer than 12 px side by side, whether one passes
//! behind a step it does not join, and whether the steps sit on the 4 px grid. A change to the
//! layout may lower the numbers; it must not raise them. Where it lowers them, run
//! `cargo test --test routing -- --ignored --nocapture` and put the printed rows in [`BASELINE`].

use std::fs;

use chartlet::{RenderFormat, RenderOptions, Variant, render_json};

/// The measured example, as name and variant; crossings, edge length in px (rounded up), and
/// pairs of runs closer than 12 px.
const BASELINE: &[(&str, &str, usize, u32, usize)] = &[
    ("api-environments", "desktop", 0, 464, 0),
    ("billing-migration", "desktop", 0, 1513, 0),
    ("booking-containers", "desktop", 0, 1244, 0),
    ("booking-context", "desktop", 0, 1092, 0),
    ("booking-deployment", "desktop", 1, 4382, 0),
    ("brand-typography", "desktop", 0, 444, 0),
    ("checkout-calls", "desktop", 0, 908, 0),
    ("claim-handling", "desktop", 0, 1000, 0),
    ("contract-review", "desktop", 0, 808, 0),
    ("contract-review", "mobile", 0, 676, 0),
    ("court-instances", "desktop", 0, 600, 0),
    ("court-instances", "mobile", 0, 556, 0),
    ("data-tier", "desktop", 0, 2048, 0),
    ("data-tier", "mobile", 0, 1732, 0),
    ("invoice-model", "desktop", 0, 904, 0),
    ("order-flow", "desktop", 0, 1791, 0),
    ("order-fulfilment", "desktop", 2, 2237, 0),
    ("order-model", "desktop", 0, 740, 0),
    ("orders-delta", "desktop", 0, 1180, 0),
    ("payment-interfaces", "desktop", 0, 1104, 0),
    ("payment-interfaces", "mobile", 0, 1220, 0),
    ("payment-notes", "desktop", 0, 1692, 0),
    ("payment-notes", "mobile", 0, 1642, 0),
    ("payment-redesign", "desktop", 0, 960, 0),
    ("release-flow", "desktop", 0, 3585, 0),
    ("release-focus", "desktop", 0, 3585, 0),
    ("release-focus", "mobile", 0, 3277, 0),
    ("request-volume", "desktop", 0, 1132, 0),
    ("service-fan-out", "desktop", 0, 3056, 0),
    ("service-perspectives", "desktop", 0, 1566, 0),
    ("shop-architecture", "desktop", 0, 3638, 0),
    ("shop-deployment", "desktop", 0, 700, 0),
    ("shop-focus", "desktop", 0, 2410, 0),
    ("shop-focus", "mobile", 0, 2212, 0),
    ("shop-modules", "desktop", 0, 1068, 0),
    ("shop-schema", "desktop", 0, 296, 0),
    ("shop-symbols", "desktop", 0, 3706, 0),
    ("shop-views", "desktop", 0, 2702, 0),
    ("sketch-pipeline", "desktop", 0, 464, 0),
    ("study-selection", "desktop", 0, 1316, 0),
    ("study-selection", "mobile", 0, 836, 0),
    ("subscription-model", "desktop", 0, 744, 0),
    ("tenant-architecture", "desktop", 0, 2410, 0),
    ("tenant-architecture", "mobile", 0, 2212, 0),
    ("ticket-states", "desktop", 0, 2183, 0),
];

#[derive(Debug, Default)]
struct Measure {
    crossings: usize,
    length: f64,
    close: usize,
    behind: usize,
    off_grid: usize,
}

type Point = (f64, f64);

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let end = tag[start..].find('"')?;
    Some(&tag[start..start + end])
}

fn number(tag: &str, name: &str) -> f64 {
    attribute(tag, name)
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("{name} of {tag}"))
}

fn points(tag: &str) -> Vec<Point> {
    attribute(tag, "points")
        .expect("points")
        .split(' ')
        .map(|pair| {
            let (x, y) = pair.split_once(',').expect("a pair");
            (x.parse().expect("x"), y.parse().expect("y"))
        })
        .collect()
}

/// The edges as polylines, and the steps as boxes (left, top, right, bottom).
fn parse(svg: &str) -> (Vec<Vec<Point>>, Vec<[f64; 4]>) {
    let body = &svg[svg.find("</style>").expect("a style")..];
    // The samples of a legend strip are no steps and no edges of the diagram.
    let body = body
        .split("class=\"chartlet-legend-strip\"")
        .next()
        .expect("a split has a first part");
    let (mut edges, mut steps) = (Vec::new(), Vec::new());
    for tag in body.split('<').skip(1) {
        let class = attribute(tag, "class").unwrap_or("");
        let classes: Vec<&str> = class.split(' ').collect();
        if tag.starts_with("polyline ") && classes.contains(&"chartlet-flow-edge") {
            edges.push(points(tag));
        } else if classes.contains(&"chartlet-flow-node") {
            if tag.starts_with("rect ") {
                let (x, y) = (number(tag, "x"), number(tag, "y"));
                steps.push([x, y, x + number(tag, "width"), y + number(tag, "height")]);
            } else if tag.starts_with("polygon ") {
                let corners = points(tag);
                let side = |pick: fn(&Point) -> f64, least: bool| {
                    corners.iter().map(pick).fold(
                        if least {
                            f64::INFINITY
                        } else {
                            f64::NEG_INFINITY
                        },
                        |a, b| if least { a.min(b) } else { a.max(b) },
                    )
                };
                steps.push([
                    side(|p| p.0, true),
                    side(|p| p.1, true),
                    side(|p| p.0, false),
                    side(|p| p.1, false),
                ]);
            }
        }
    }
    (edges, steps)
}

/// How far an edge reaches into the steps it joins.
const INSET: f64 = 6.0;

fn runs(line: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    line.windows(2).map(|pair| (pair[0], pair[1]))
}

#[allow(clippy::many_single_char_names)]
fn crosses((a, b): (Point, Point), (c, d): (Point, Point)) -> bool {
    let cross = (b.0 - a.0) * (d.1 - c.1) - (b.1 - a.1) * (d.0 - c.0);
    if cross.abs() < 1e-9 {
        return false;
    }
    let t = ((c.0 - a.0) * (d.1 - c.1) - (c.1 - a.1) * (d.0 - c.0)) / cross;
    let u = ((c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)) / cross;
    (0.001..0.999).contains(&t) && (0.001..0.999).contains(&u)
}

/// Whether two straight runs lie side by side closer than 12 px, and further than 0.
fn close((a, b): (Point, Point), (c, d): (Point, Point)) -> bool {
    for (axis, along) in [(0, 1), (1, 0)] {
        let pick = |p: Point, axis: usize| if axis == 0 { p.0 } else { p.1 };
        if (pick(a, axis) - pick(b, axis)).abs() < 1e-6
            && (pick(c, axis) - pick(d, axis)).abs() < 1e-6
        {
            let distance = (pick(a, axis) - pick(c, axis)).abs();
            let overlap = pick(a, along)
                .max(pick(b, along))
                .min(pick(c, along).max(pick(d, along)))
                - pick(a, along)
                    .min(pick(b, along))
                    .max(pick(c, along).min(pick(d, along)));
            if distance > 0.01 && distance < 12.0 && overlap > 2.0 {
                return true;
            }
        }
    }
    false
}

fn measure(svg: &str) -> Measure {
    let (edges, steps) = parse(svg);
    let mut measure = Measure::default();
    for (index, edge) in edges.iter().enumerate() {
        measure.length += runs(edge)
            .map(|(a, b)| (b.0 - a.0).hypot(b.1 - a.1))
            .sum::<f64>();
        for other in &edges[index + 1..] {
            for first in runs(edge) {
                for second in runs(other) {
                    measure.crossings += usize::from(crosses(first, second));
                    measure.close += usize::from(close(first, second));
                }
            }
        }
        // An edge starts and ends a few pixels inside the steps it joins; only a run through a
        // step further in counts as passing behind it.
        for (a, b) in runs(edge) {
            measure.behind += steps
                .iter()
                .filter(|step| {
                    a.0.max(b.0) > step[0] + INSET
                        && a.0.min(b.0) < step[2] - INSET
                        && a.1.max(b.1) > step[1] + INSET
                        && a.1.min(b.1) < step[3] - INSET
                })
                .count();
        }
    }
    measure.off_grid = steps
        .iter()
        .filter(|step| {
            step.iter()
                .any(|v| (v / 4.0 - (v / 4.0).round()).abs() > 0.01)
        })
        .count();
    measure
}

/// Every diagram example with the variants it has a reference for, as name, variant and SVG.
fn examples() -> Vec<(String, &'static str, String)> {
    let mut found = Vec::new();
    let mut names: Vec<String> = fs::read_dir("examples")
        .expect("the examples")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".json").map(str::to_owned)
        })
        .collect();
    names.sort();
    for name in names {
        let json = fs::read_to_string(format!("examples/{name}.json")).expect("the example");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        if !matches!(
            value["type"].as_str(),
            Some("flow" | "architecture" | "state" | "classes")
        ) {
            continue;
        }
        for (variant, label, suffix) in [
            (Variant::Desktop, "desktop", ""),
            (Variant::Mobile, "mobile", ".mobile"),
        ] {
            if suffix.is_empty() || fs::metadata(format!("examples/{name}{suffix}.svg")).is_ok() {
                let options = RenderOptions {
                    variant,
                    ..RenderOptions::default()
                };
                let svg = render_json(&json, RenderFormat::Svg, &options)
                    .unwrap_or_else(|error| panic!("{name} renders: {error}"))
                    .content;
                found.push((name.clone(), label, svg));
            }
        }
    }
    found
}

#[test]
#[ignore = "prints the rows of BASELINE"]
fn print_baseline() {
    for (name, variant, svg) in examples() {
        let m = measure(&svg);
        println!(
            "    (\"{name}\", \"{variant}\", {}, {}, {}),",
            m.crossings,
            m.length.ceil(),
            m.close
        );
    }
}

/// Edges cross no more often, run no longer and keep no closer to each other than before.
#[test]
fn routing_does_not_get_worse() {
    let examples = examples();
    assert!(examples.len() > 30, "the diagram examples are found");
    for (name, variant, svg) in &examples {
        let row = BASELINE
            .iter()
            .find(|row| row.0 == name && row.1 == *variant)
            .unwrap_or_else(|| panic!("{name} ({variant}) has no row in BASELINE"));
        let m = measure(svg);
        assert!(
            m.crossings <= row.2,
            "{name} ({variant}): {} crossings, up from {}",
            m.crossings,
            row.2
        );
        assert!(
            m.length <= f64::from(row.3),
            "{name} ({variant}): edges {:.0} px long, up from {}",
            m.length,
            row.3
        );
        assert!(
            m.close <= row.4,
            "{name} ({variant}): {} runs closer than 12 px, up from {}",
            m.close,
            row.4
        );
    }
}

/// No edge passes behind a step it does not join.
#[test]
fn no_edge_passes_behind_a_step() {
    for (name, variant, svg) in examples() {
        let m = measure(&svg);
        assert_eq!(m.behind, 0, "{name} ({variant}) has an edge behind a step");
    }
}

/// Every step's box lies on the 4 px grid; the sizes of fonts are exempt.
#[test]
fn every_step_lies_on_the_grid() {
    for (name, variant, svg) in examples() {
        let m = measure(&svg);
        assert_eq!(
            m.off_grid, 0,
            "{name} ({variant}) has steps off the 4 px grid"
        );
    }
}
