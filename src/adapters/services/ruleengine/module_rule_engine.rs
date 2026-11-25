use crate::domain::rule_engine::{RuleEngine, RuleRun, RuleRunError};
use async_trait::async_trait;
use ruleengine::{DatabaseConfig, Error, RuleEngine as ModuleEngine, SimulateRequest};
use serde_json::Value;
use std::time::Duration;

/// A rule that runs longer than this is stopped.
const RUN_TIMEOUT: Duration = Duration::from_secs(5);

/// Runs a rule version's compiled decision graph with the `ms-ruleengine-module-rust` engine.
///
/// The rule versions stay in this service's database, so the engine runs without S3 or NATS
/// (`standalone`): each run hands it the version's graph through `simulate`.
pub struct ModuleRuleEngine {
    engine: ModuleEngine,
}

impl ModuleRuleEngine {
    pub async fn new() -> Self {
        let engine = ModuleEngine::standalone(DatabaseConfig::default(), ModuleEngine::new_rules_storage())
            .await
            .expect("failed to start the rule engine");
        Self { engine }
    }
}

/// The engine's own error text is a JSON envelope for a node failure
/// (`{"type":"NodeError","nodeId":"ex","source":"..."}`): pull the node and the message out of it.
fn failure(text: String) -> RuleRunError {
    if let Ok(envelope) = serde_json::from_str::<Value>(&text) {
        let node = envelope.get("nodeId").and_then(Value::as_str).map(str::to_string);
        let message = envelope
            .get("source")
            .or_else(|| envelope.get("message"))
            .and_then(|v| v.as_str().map(str::to_string).or_else(|| Some(v.to_string())))
            .unwrap_or(text);
        return RuleRunError::Failed { node, message };
    }
    RuleRunError::Failed { node: None, message: text }
}

#[async_trait]
impl RuleEngine for ModuleRuleEngine {
    async fn run(&self, graph: &Value, input: Value, trace: bool) -> Result<RuleRun, RuleRunError> {
        let request = SimulateRequest { content: graph.clone(), context: input };
        let response = tokio::time::timeout(RUN_TIMEOUT, self.engine.simulate(request))
            .await
            .map_err(|_| RuleRunError::Timeout)?
            .map_err(|error| match error {
                Error::Content(message) => RuleRunError::BadGraph(message),
                Error::Engine(text) => failure(text),
                other => RuleRunError::Failed { node: None, message: other.to_string() },
            })?;
        // The engine always traces a simulation; the trace is only returned when asked for.
        Ok(RuleRun { result: response.result, trace: if trace { response.trace } else { None } })
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
    async fn runs_a_graph_and_keeps_decimals() {
        let run = ModuleRuleEngine::new()
            .await
            .run(&expression_graph("base_price - base_price * 0.1"), json!({"base_price": 40.1}), true)
            .await
            .unwrap();
        assert_eq!(run.result["base_price"].as_f64(), Some(40.1));
        assert_eq!(run.result["y"].as_f64(), Some(36.09));
        assert!(run.trace.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_trace_is_only_returned_when_asked_for() {
        let run = ModuleRuleEngine::new().await.run(&expression_graph("1"), json!({}), false).await.unwrap();
        assert!(run.trace.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_failing_node_is_named() {
        let error = ModuleRuleEngine::new()
            .await
            .run(&expression_graph("unknownFn(base_price)"), json!({"base_price": 1}), false)
            .await
            .unwrap_err();
        match error {
            RuleRunError::Failed { node, message } => {
                assert_eq!(node.as_deref(), Some("ex"), "{message}");
                assert!(message.contains("unknownFn"), "{message}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_graph_the_engine_cannot_load_is_reported() {
        let error = ModuleRuleEngine::new().await.run(&json!({"nodes": [1]}), json!({}), false).await.unwrap_err();
        assert!(matches!(error, RuleRunError::BadGraph(_) | RuleRunError::Failed { .. }), "{error:?}");
    }
}
