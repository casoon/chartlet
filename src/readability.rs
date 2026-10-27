//! Readability budgets of diagrams: above a size a diagram is hard to read and is better split
//! into an overview and details. The numbers are editorial guidance for a canvas of the default
//! size, not a limit of correctness: the budget grows with the area of the canvas, a tree has a
//! larger one because it reads top-down, and the count of nodes and edges is only a heuristic
//! (a crowded small flow below the budget is not caught, a large clear one above it is
//! reported though it may be fine). Nothing here stops a render.

use crate::{
    error::ChartWarning,
    spec::{ChartSpec, ChartType},
};

/// The code of the warning and of the inspection.
pub(crate) const DIAGRAM_OVER_BUDGET: &str = "diagram_over_budget";

/// The area of the default canvas, 800 by 450.
const DEFAULT_AREA: f64 = 800.0 * 450.0;

/// A kind of thing a diagram counts: its name, how many there are and how many read well on the
/// default canvas.
type Count = (&'static str, usize, usize);

/// What one type of diagram counts.
struct Budget {
    /// JSON Pointer of the block that holds the counted things.
    path: &'static str,
    items: Count,
    relations: Option<Count>,
}

/// A diagram over its budget: where, and what was counted against what.
pub(crate) struct Finding {
    pub path: &'static str,
    pub message: String,
    pub hint: String,
}

fn budget(spec: &ChartSpec) -> Option<Budget> {
    // A model with views is drawn one view at a time; each view is checked on its own.
    if spec.has_views() {
        return None;
    }
    Some(match spec.chart_type {
        ChartType::Flow => {
            let flow = spec.flow.as_ref()?;
            Budget {
                path: "/flow",
                items: ("steps", flow.nodes.len(), 9),
                relations: Some(("edges", flow.edges.len(), 12)),
            }
        }
        ChartType::State => {
            let state = spec.state.as_ref()?;
            Budget {
                path: "/state",
                items: ("states", state.states.len(), 9),
                relations: Some(("transitions", state.transitions.len(), 12)),
            }
        }
        ChartType::Architecture => {
            let architecture = spec.architecture.as_ref()?;
            Budget {
                path: "/architecture",
                items: ("components", architecture.components.len(), 9),
                relations: Some(("connections", architecture.connections.len(), 12)),
            }
        }
        ChartType::Classes => {
            let classes = spec.classes.as_ref()?;
            Budget {
                path: "/classes",
                items: ("classes", classes.classes.len(), 9),
                relations: Some(("relations", classes.relations.len(), 12)),
            }
        }
        ChartType::Sequence => {
            let sequence = spec.sequence.as_ref()?;
            Budget {
                path: "/sequence",
                items: ("participants", sequence.participants.len(), 6),
                relations: Some(("messages", sequence.messages.len(), 16)),
            }
        }
        ChartType::Tree => {
            let tree = spec.tree.as_ref()?;
            Budget {
                path: "/tree",
                items: ("nodes", tree.nodes.len(), 24),
                relations: None,
            }
        }
        _ => return None,
    })
}

/// The budget at the size of the canvas: the area relative to the default canvas, as a square
/// root so that doubling both sides doubles the budget. A canvas smaller than the default keeps
/// the budget.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "the factor is at least 1 and the budget small"
)]
fn scaled(limit: usize, spec: &ChartSpec) -> usize {
    let area = f64::from(spec.width) * f64::from(spec.height);
    let factor = (area / DEFAULT_AREA).sqrt().max(1.0);
    (limit as f64 * factor).floor() as usize
}

/// The finding for a diagram that has more than its budget of items or relations.
pub(crate) fn finding(spec: &ChartSpec) -> Option<Finding> {
    let budget = budget(spec)?;
    let mut over = Vec::new();
    for (name, count, limit) in std::iter::once(budget.items).chain(budget.relations) {
        let limit = scaled(limit, spec);
        if count > limit {
            over.push(format!("{count} {name} (budget {limit} at this size)"));
        }
    }
    if over.is_empty() {
        return None;
    }
    let split = if spec.chart_type == ChartType::Architecture {
        "split it into views (an overview and details)"
    } else {
        "split it into an overview and detail diagrams"
    };
    Some(Finding {
        path: budget.path,
        message: format!(
            "the diagram has {}, more than reads well at {}x{}",
            over.join(" and "),
            spec.width,
            spec.height
        ),
        hint: format!(
            "{split}, try the other orientation or a larger size, or keep it if it reads well; no content is removed for you"
        ),
    })
}

/// The warning `diagram_over_budget` for a diagram over its budget, an empty list otherwise.
pub(crate) fn warnings(spec: &ChartSpec) -> Vec<ChartWarning> {
    finding(spec)
        .map(|finding| {
            ChartWarning::new(
                DIAGRAM_OVER_BUDGET,
                finding.path,
                format!("{}: {}", finding.message, finding.hint),
            )
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn assert_quiet(spec: &ChartSpec) {
        assert_eq!(warnings(spec), Vec::<ChartWarning>::new());
    }

    fn flow(nodes: usize, edges: usize, size: (u32, u32)) -> ChartSpec {
        let steps: Vec<Value> = (0..nodes)
            .map(|index| json!({ "id": format!("s{index}"), "label": format!("Step {index}") }))
            .collect();
        let joins: Vec<Value> = (0..edges)
            .map(|index| {
                json!({ "from": format!("s{}", index % (nodes - 1)), "to": format!("s{}", index % (nodes - 1) + 1) })
            })
            .collect();
        ChartSpec::from_json(
            &json!({
                "schemaVersion": 1, "type": "flow", "title": "Steps",
                "width": size.0, "height": size.1,
                "flow": { "nodes": steps, "edges": joins },
            })
            .to_string(),
        )
        .expect("a flow")
    }

    #[test]
    fn a_flow_over_nine_steps_or_twelve_edges_is_over_budget() {
        let spec = flow(10, 9, (800, 450));
        let found = warnings(&spec);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code, "diagram_over_budget");
        assert_eq!(found[0].path, "/flow");
        assert!(
            found[0]
                .message
                .contains("10 steps (budget 9 at this size)")
        );
        assert!(found[0].message.contains("split it into an overview"));
        assert_quiet(&flow(9, 8, (800, 450)));
        assert_quiet(&flow(9, 12, (800, 450)));
        assert_eq!(warnings(&flow(9, 13, (800, 450))).len(), 1);
    }

    #[test]
    fn a_crowded_small_diagram_below_the_budget_is_not_caught() {
        // Counting is a heuristic: nine steps and twelve edges in a small canvas pass.
        assert_quiet(&flow(9, 12, (320, 200)));
    }

    #[test]
    fn a_larger_canvas_raises_the_budget() {
        assert_eq!(warnings(&flow(12, 11, (800, 450))).len(), 1);
        assert_quiet(&flow(12, 11, (1200, 675)));
    }

    #[test]
    fn a_tree_has_a_larger_budget() {
        let nodes: Vec<Value> = (0..24)
            .map(|index| {
                let mut node = json!({ "id": format!("n{index}"), "label": format!("N{index}") });
                if index > 0 {
                    node["parent"] = json!(format!("n{}", (index - 1) / 3));
                }
                node
            })
            .collect();
        let tree = |nodes: &[Value]| {
            ChartSpec::from_json(
                &json!({ "schemaVersion": 1, "type": "tree", "title": "Org", "tree": { "nodes": nodes } })
                    .to_string(),
            )
            .expect("a tree")
        };
        assert_quiet(&tree(&nodes));
        let mut more = nodes;
        more.push(json!({ "id": "x", "label": "X", "parent": "n0" }));
        assert_eq!(warnings(&tree(&more)).len(), 1);
    }

    #[test]
    fn a_sequence_counts_participants_and_messages() {
        let sequence = |participants: usize, messages: usize| {
            let people: Vec<Value> = (0..participants)
                .map(|index| json!({ "id": format!("p{index}"), "label": format!("P{index}") }))
                .collect();
            let sent: Vec<Value> = (0..messages)
                .map(|index| {
                    json!({ "from": format!("p{}", index % 2), "to": format!("p{}", 1 - index % 2), "label": format!("m{index}") })
                })
                .collect();
            ChartSpec::from_json(
                &json!({
                    "schemaVersion": 1, "type": "sequence", "title": "Calls",
                    "sequence": { "participants": people, "messages": sent },
                })
                .to_string(),
            )
            .expect("a sequence")
        };
        assert_quiet(&sequence(6, 16));
        assert!(
            warnings(&sequence(7, 16))[0]
                .message
                .contains("7 participants")
        );
        assert!(
            warnings(&sequence(6, 17))[0]
                .message
                .contains("17 messages")
        );
    }

    #[test]
    fn other_types_have_no_budget() {
        let spec = ChartSpec::from_json(
            r#"{"schemaVersion":1,"type":"bar","title":"T","data":[{"label":"A","value":1}]}"#,
        )
        .expect("a bar chart");
        assert_quiet(&spec);
    }
}
