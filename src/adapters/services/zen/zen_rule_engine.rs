use crate::domain::rule_engine::{RuleEngine, RuleRun, RuleRunError};
use async_trait::async_trait;
use serde_json::{Map, Value};
use std::time::Duration;
use zen_engine::model::DecisionContent;
use zen_engine::{Decision, EvaluationError, EvaluationOptions, Variable};

/// A rule that runs longer than this is stopped.
const RUN_TIMEOUT: Duration = Duration::from_secs(5);

/// Runs a rule's compiled decision graph (GoRules JDM) with the zen engine, in this process.
pub struct ZenRuleEngine;

impl ZenRuleEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZenRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// What went wrong inside the engine: the node it stopped at (when it says) and a message.
fn failure(error: &EvaluationError) -> RuleRunError {
    match error {
        EvaluationError::NodeError { node_id, source, .. } => {
            RuleRunError::Failed { node: Some(node_id.to_string()), message: source.to_string() }
        }
        other => RuleRunError::Failed { node: None, message: other.to_string() },
    }
}

#[async_trait]
impl RuleEngine for ZenRuleEngine {
    async fn run(&self, graph: &Value, input: Value, trace: bool) -> Result<RuleRun, RuleRunError> {
        let content: DecisionContent = serde_json::from_value(graph.clone())
            .map_err(|err| RuleRunError::BadGraph(format!("the compiled graph cannot be loaded: {err}")))?;

        // The engine's values are not `Send`, so the run lives on one blocking thread, which drives it
        // to the end there; only plain JSON crosses back.
        //
        // The result is turned into JSON with `Variable::to_value`, not with the engine's own serializer
        // (`evaluate_serialized`): without serde_json's `arbitrary_precision` that serializer writes a
        // decimal as a whole number (40.10 becomes 40), which is wrong for prices.
        let handle = tokio::runtime::Handle::current();
        let run = tokio::task::spawn_blocking(move || {
            handle.block_on(async move {
                let decision = Decision::from(content);
                let options = EvaluationOptions { trace, ..Default::default() };
                tokio::time::timeout(RUN_TIMEOUT, decision.evaluate_with_opts(Variable::from(input), options))
                    .await
                    .map(|outcome| {
                        outcome.map(|response| RuleRun {
                            result: response.result.to_value(),
                            trace: response.trace.map(|nodes| {
                                Value::Object(
                                    nodes
                                        .into_iter()
                                        .map(|(id, node)| {
                                            let mut entry = Map::new();
                                            entry.insert("id".into(), Value::String(node.id.to_string()));
                                            entry.insert("name".into(), Value::String(node.name.to_string()));
                                            entry.insert("order".into(), Value::from(node.order));
                                            entry.insert("input".into(), node.input.to_value());
                                            entry.insert("output".into(), node.output.to_value());
                                            if let Some(performance) = node.performance {
                                                entry.insert("performance".into(), Value::String(performance.to_string()));
                                            }
                                            (id.to_string(), Value::Object(entry))
                                        })
                                        .collect(),
                                )
                            }),
                        })
                    })
                    .map_err(|_| RuleRunError::Timeout)
                    .and_then(|outcome| outcome.map_err(|error| failure(&error)))
            })
        })
        .await
        .map_err(|err| RuleRunError::Failed { node: None, message: format!("the rule run was interrupted: {err}") })??;

        Ok(run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn expression_graph(expression: &str) -> Value {
        json!({
            "nodes": [
                {"id": "in", "name": "Request", "type": "inputNode", "position": {"x": 0, "y": 0}},
                {"id": "ex", "name": "Calc", "type": "expressionNode", "position": {"x": 1, "y": 0},
                 "content": {"expressions": [{"id": "a", "key": "y", "value": expression}], "passThrough": true}},
                {"id": "out", "name": "Response", "type": "outputNode", "position": {"x": 2, "y": 0}}
            ],
            "edges": [
                {"id": "e1", "type": "edge", "sourceId": "in", "targetId": "ex"},
                {"id": "e2", "type": "edge", "sourceId": "ex", "targetId": "out"}
            ]
        })
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn decimals_are_not_cut_to_whole_numbers() {
        let run = ZenRuleEngine::new()
            .run(&expression_graph("base_price - base_price * 0.1"), json!({"base_price": 40.1}), true)
            .await
            .unwrap();
        assert_eq!(run.result["base_price"].as_f64(), Some(40.1));
        assert_eq!(run.result["y"].as_f64(), Some(36.09));
        let trace = run.trace.unwrap();
        assert_eq!(trace["in"]["output"]["base_price"].as_f64(), Some(40.1));
        assert_eq!(trace["ex"]["name"], "Calc");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_failing_node_is_named() {
        let error = ZenRuleEngine::new()
            .run(&expression_graph("unknownFn(base_price)"), json!({"base_price": 1}), false)
            .await
            .unwrap_err();
        match error {
            RuleRunError::Failed { node, message } => {
                assert_eq!(node.as_deref(), Some("ex"));
                assert!(message.contains("unknownFn"), "{message}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_graph_the_engine_cannot_load_is_reported() {
        let error = ZenRuleEngine::new().run(&json!({"nodes": [1]}), json!({}), false).await.unwrap_err();
        assert!(matches!(error, RuleRunError::BadGraph(_)), "{error:?}");
    }
}
