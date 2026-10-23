use super::*;

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
