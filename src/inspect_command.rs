//! `chartlet inspect`: the inspections of an architecture model, without rendering it. Part of
//! the command-line program, not of the library.

use std::path::PathBuf;

use chartlet::{INSPECTIONS, Inspection, Severity, inspect_json};
use serde_json::{Value, json};

use crate::{Failure, read_input, spec_failure, usage, write_stdout};

/// Writes the findings to stdout, as text lines or one JSON document. Findings are no failure;
/// with `--strict`, a gap is.
pub fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), Failure> {
    let input = match arguments.next() {
        Some(help) if help == "-h" || help == "--help" => return Ok(write_stdout(&usage())?),
        Some(input) => PathBuf::from(input),
        None => return Err(usage().into()),
    };
    let mut disabled = Vec::new();
    let mut strict = false;
    let mut json_output = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--disable" => {
                let code = arguments
                    .next()
                    .ok_or_else(|| "--disable requires an inspection code".to_owned())?;
                if !INSPECTIONS.contains(&code.as_str()) {
                    return Err(format!(
                        "unknown inspection {code:?}; the inspections are {}",
                        INSPECTIONS.join(", ")
                    )
                    .into());
                }
                disabled.push(code);
            }
            "--strict" => strict = true,
            "--format" => {
                json_output = match arguments.next().as_deref() {
                    Some("text") => false,
                    Some("json") => true,
                    _ => return Err("--format must be text or json".to_owned().into()),
                };
            }
            "--diagnostics" => match arguments.next().as_deref() {
                Some("json" | "text") => {}
                _ => return Err("--diagnostics must be text or json".to_owned().into()),
            },
            "--diagnostics=json" | "--diagnostics=text" => {}
            "-h" | "--help" => return Ok(write_stdout(&usage())?),
            unknown => return Err(format!("unknown argument {unknown:?}\n\n{}", usage()).into()),
        }
    }

    let disabled: Vec<&str> = disabled.iter().map(String::as_str).collect();
    let findings = inspect_json(&read_input(&input)?, &disabled).map_err(spec_failure)?;
    let report = if json_output {
        json!({ "findings": findings.iter().map(finding_json).collect::<Vec<Value>>() }).to_string()
    } else {
        findings
            .iter()
            .map(|finding| {
                format!(
                    "{}[{}] at {}: {}; {}",
                    finding.severity.name(),
                    finding.code,
                    finding.path,
                    finding.message,
                    finding.hint
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    if !report.is_empty() {
        write_stdout(&report)?;
    }
    let gaps = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Gap)
        .count();
    if strict && gaps > 0 {
        return Err(Failure {
            code: "strict_inspections",
            path: None,
            message: format!("strict mode rejected {gaps} gap(s)"),
        });
    }
    Ok(())
}

fn finding_json(finding: &Inspection) -> Value {
    json!({
        "code": finding.code,
        "severity": finding.severity.name(),
        "path": finding.path,
        "message": finding.message,
        "hint": finding.hint,
    })
}
