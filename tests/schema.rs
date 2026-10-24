use std::fs;
use std::path::Path;

use jsonschema::Validator;
use serde_json::{Value, json};

fn validator() -> Validator {
    let schema: Value = serde_json::from_str(include_str!("../schema/chartlet.schema.json"))
        .expect("the schema is JSON");
    jsonschema::draft202012::new(&schema).expect("the schema compiles")
}

/// The first error as `instance path: message`, or `None` for a valid specification.
fn first_error(validator: &Validator, spec: &Value) -> Option<String> {
    validator
        .validate(spec)
        .err()
        .map(|error| format!("{}: {error}", error.instance_path()))
}

#[test]
fn every_example_matches_the_schema() {
    let validator = validator();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut paths: Vec<_> = fs::read_dir(&dir)
        .expect("examples/ is readable")
        .map(|entry| entry.expect("examples/ entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no examples in {}", dir.display());

    let failures: Vec<String> = paths
        .iter()
        .filter_map(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            let spec: Value = serde_json::from_str(&fs::read_to_string(path).unwrap())
                .unwrap_or_else(|error| panic!("{name} is not JSON: {error}"));
            first_error(&validator, &spec).map(|error| format!("{name}: {error}"))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} examples do not match the schema:\n{}",
        failures.len(),
        paths.len(),
        failures.join("\n")
    );
}

#[test]
fn invalid_specifications_are_rejected() {
    let validator = validator();
    let valid = json!({
        "schemaVersion": 1,
        "type": "bar",
        "title": "Monthly revenue",
        "data": [{ "label": "January", "value": 120 }]
    });
    assert_eq!(first_error(&validator, &valid), None);

    let mut unknown_type = valid.clone();
    unknown_type["type"] = json!("unknown");
    let mut missing_title = valid.clone();
    missing_title.as_object_mut().unwrap().remove("title");
    let mut unknown_property = valid.clone();
    unknown_property["colour"] = json!("red");
    let mut wrong_version = valid.clone();
    wrong_version["schemaVersion"] = json!(2);

    for (case, spec) in [
        ("unknown type", unknown_type),
        ("missing title", missing_title),
        ("unknown property", unknown_property),
        ("wrong schema version", wrong_version),
    ] {
        assert!(
            first_error(&validator, &spec).is_some(),
            "{case} passes the schema"
        );
    }
}

#[test]
fn tokens_and_theme_files_match_their_schemas() {
    let validator = validator();
    let mut spec = json!({
        "schemaVersion": 1, "type": "bar", "title": "Revenue",
        "data": [{ "label": "A", "value": 1 }],
        "tokens": { "accent": "#ff5a5f", "color-2": "rgba(10, 20, 30, .5)" }
    });
    assert_eq!(first_error(&validator, &spec), None);
    for tokens in [
        json!({ "nope": "#fff" }),
        json!({ "accent": "red" }),
        json!({ "color-5": "#fff" }),
    ] {
        spec["tokens"] = tokens;
        assert!(
            first_error(&validator, &spec).is_some(),
            "{} passes",
            spec["tokens"]
        );
    }

    let theme_schema: Value =
        serde_json::from_str(include_str!("../schema/chartlet-theme.schema.json")).expect("JSON");
    let theme_validator = jsonschema::draft202012::new(&theme_schema).expect("compiles");
    for name in ["light", "dark", "blueprint"] {
        let theme: Value = serde_json::from_str(
            &fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("themes/{name}.json")),
            )
            .expect("theme file"),
        )
        .expect("JSON");
        assert_eq!(first_error(&theme_validator, &theme), None, "{name}");
    }
    assert!(first_error(&theme_validator, &json!({ "title": "x" })).is_some());
}
