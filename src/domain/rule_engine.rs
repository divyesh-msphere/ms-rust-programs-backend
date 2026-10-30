// domain/rule_engine.rs
//
// Running a rule: the decision graph a rule version carries, evaluated against the caller's input.
use async_trait::async_trait;
use serde_json::Value;

/// What a run produced.
#[derive(Debug, Clone)]
pub struct RuleRun {
    /// Whatever the rule outputs: dynamic JSON, never reshaped.
    pub result: Value,
    /// The nodes that were visited, when the caller asked for the trace.
    pub trace: Option<Value>,
}

/// Why a run did not produce a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleRunError {
    /// The version's graph is not one the engine can load.
    BadGraph(String),
    /// The engine failed while running it; `node` is the decision node it stopped at.
    Failed { node: Option<String>, message: String },
    /// It ran past the time limit.
    Timeout,
}

#[async_trait]
pub trait RuleEngine: Send + Sync {
    /// Runs `graph` (a compiled rule version) on `input`.
    async fn run(&self, graph: &Value, input: Value, trace: bool) -> Result<RuleRun, RuleRunError>;
}
