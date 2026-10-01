use chartlet::{RenderFormat, RenderOptions, render_json};

/// Every example specification as name and JSON text.
fn examples() -> Vec<(String, String)> {
    let mut examples: Vec<(String, String)> = std::fs::read_dir("examples")
        .expect("the examples directory should exist")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            (
                path.file_stem()
                    .expect("a file name")
                    .to_string_lossy()
                    .into_owned(),
                std::fs::read_to_string(&path).expect("readable example"),
            )
        })
        .collect();
    examples.sort();
    examples
}

fn render(specification: &str, format: RenderFormat, hooks: bool) -> String {
    render_json(
        specification,
        format,
        &RenderOptions {
            hooks,
            ..RenderOptions::default()
        },
    )
    .expect("every example renders")
    .content
}

/// Removes the attribute `name="…"`, with its leading space, wherever it occurs.
fn remove_attribute(content: &str, name: &str) -> String {
    let needle = format!(" {name}=\"");
    let mut output = String::new();
    let mut rest = content;
    while let Some(start) = rest.find(&needle) {
        output.push_str(&rest[..start]);
        let value = &rest[start + needle.len()..];
        rest = &value[value.find('"').expect("a closed attribute") + 1..];
    }
    output.push_str(rest);
    output
}

/// Removes every element from `open` up to and including `close`.
fn remove_between(content: &str, open: &str, close: &str) -> String {
    let mut output = String::new();
    let mut rest = content;
    while let Some(start) = rest.find(open) {
        output.push_str(&rest[..start]);
        let after = &rest[start..];
        rest = &after[after.find(close).expect("a closed element") + close.len()..];
    }
    output.push_str(rest);
    output
}

/// The output with every hook taken out again. Without hooks chartlet writes no `<g>` at all, and
/// a layer group is left as a bare `<g>` once its attributes are gone.
fn without_hooks(content: &str) -> String {
    // The hooks of a table column name the series before the pane; a bar's own `data-series`
    // is never followed by a pane.
    let mut content = content.to_owned();
    while let Some(start) = content.find(" data-pane=\"") {
        let series = content[..start]
            .rfind(" data-series=\"")
            .filter(|series| !content[*series..start].contains('>'));
        let end = start + content[start + 12..].find('"').expect("closed") + 13;
        content.replace_range(series.unwrap_or(start)..end, "");
    }
    for name in [
        "data-chartlet-type",
        "data-chartlet-id",
        "data-chartlet-table",
        "data-part",
        "data-x",
        "data-value",
        "data-name",
        "data-annotation",
    ] {
        content = remove_attribute(&content, name);
    }
    content = remove_between(&content, "<script type=\"application/json\"", "</script>");
    content = remove_between(&content, "<g data-chartlet-plot", "/>");
    content.replace("<g>", "").replace("</g>", "")
}

#[test]
fn hooks_only_add_to_every_example() {
    for (name, specification) in examples() {
        for format in [RenderFormat::Svg, RenderFormat::Html] {
            let plain = render(&specification, format, false);
            let hooked = render(&specification, format, true);
            assert!(
                !plain.contains("data-chartlet-"),
                "{name}: hooks without the option"
            );
            assert!(hooked.contains("data-chartlet-type"), "{name}: no hooks");
            assert_eq!(without_hooks(&hooked), plain, "{name} ({format:?})");
        }
    }
}

/// The JSON data block of an SVG, unescaped.
fn data_block(svg: &str) -> serde_json::Value {
    let start = svg.find("data-chartlet-data=\"\">").expect("a data block") + 22;
    let end = start + svg[start..].find("</script>").expect("a closed block");
    let json = svg[start..end]
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    serde_json::from_str(&json).expect("the data block is JSON")
}

fn numbers(text: &str) -> Vec<f64> {
    text.split_whitespace()
        .map(|number| number.parse().expect("a number"))
        .collect()
}

fn attribute<'a>(tag: &'a str, name: &str) -> &'a str {
    let start = tag.find(&format!(" {name}=\"")).expect("the attribute") + name.len() + 3;
    &tag[start..start + tag[start..].find('"').expect("closed")]
}

#[test]
fn the_svg_profile_carries_the_table_as_a_data_block() {
    let specification = include_str!("../examples/sensor-readings.json");
    let block = data_block(&render(specification, RenderFormat::Svg, true));
    let html = render(specification, RenderFormat::Html, true);
    let columns = block["columns"].as_array().expect("columns");
    let rows = block["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), html.matches("<tr data-x=").count());
    assert_eq!(columns[1]["name"], "Servers");
    assert_eq!(columns[1]["series"], 1);
    assert_eq!(columns[1]["part"], "value");
    assert!(html.contains(
        "<th scope=\"col\" data-series=\"1\" data-pane=\"0\" data-part=\"value\">Servers</th>"
    ));
    assert_eq!(rows[0]["label"], "2026-03-01");
    assert_eq!(rows[0]["value"][1], 22.0);
    assert_eq!(rows[0]["text"][1], "22.0");
    // The HTML profile has its table and carries no data block.
    assert!(!html.contains("<script"));
}

#[test]
fn a_plot_hook_maps_an_observation_onto_its_drawn_point() {
    let svg = render(
        include_str!("../examples/sensor-readings.json"),
        RenderFormat::Svg,
        true,
    );
    let block = data_block(&svg);
    let plot = &svg[svg.find("<g data-chartlet-plot").expect("a plot hook")..];
    let plot = &plot[..plot.find("/>").expect("closed")];
    let (x_domain, x_range) = (
        numbers(attribute(plot, "data-x-domain")),
        numbers(attribute(plot, "data-x-range")),
    );
    let (y_domain, y_range) = (
        numbers(attribute(plot, "data-y-domain")),
        numbers(attribute(plot, "data-y-range")),
    );
    let linear = |domain: &[f64], range: &[f64], value: f64| {
        range[0] + (value - domain[0]) / (domain[1] - domain[0]) * (range[1] - range[0])
    };
    let row = &block["rows"][0];
    let x = linear(&x_domain, &x_range, row["x"].as_f64().expect("x"));
    let y = linear(
        &y_domain,
        &y_range,
        row["value"][1].as_f64().expect("a value"),
    );

    let group = &svg[svg.find("<g data-series=\"1\"").expect("the Servers layer")..];
    let points = attribute(
        &group[group.find("<polyline").expect("its line")..],
        "points",
    );
    let first: Vec<f64> = points
        .split_whitespace()
        .next()
        .expect("a point")
        .split(',')
        .map(|number| number.parse().expect("a number"))
        .collect();
    assert!((first[0] - x).abs() < 0.01, "{first:?} vs {x}");
    assert!((first[1] - y).abs() < 0.01, "{first:?} vs {y}");
}

#[test]
fn collapsed_gaps_list_every_slot() {
    let svg = render(
        include_str!("../examples/share-price.json"),
        RenderFormat::Svg,
        true,
    );
    let plots: Vec<&str> = svg.split("<g data-chartlet-plot").skip(1).collect();
    assert_eq!(plots.len(), 2, "one plot per pane");
    let rows = data_block(&svg)["rows"].as_array().expect("rows").len();
    assert_eq!(numbers(attribute(plots[0], "data-x-domain")).len(), rows);
    assert_eq!(attribute(plots[1], "data-pane"), "1");
}

#[test]
fn the_cli_writes_hooks_with_the_flag() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/monthly-trend.json",
            "--format",
            "svg",
            "--hooks",
        ])
        .output()
        .expect("the CLI runs");
    assert!(output.status.success(), "{output:?}");
    let svg = String::from_utf8(output.stdout).expect("UTF-8");
    assert!(svg.contains("data-chartlet-type=\"line\""));
    assert!(svg.contains("data-x-domain=\"0 5\""));
}
