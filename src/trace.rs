use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What kind of step a trace call represents.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TraceCallKind {
    /// An LLM completion call.
    Llm,
    /// A tool invocation.
    Tool,
}

/// One step in an agent trace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceCall {
    /// Whether this is an LLM call or a tool invocation.
    pub kind: TraceCallKind,
    /// Display name (model id for LLM, function name for tool).
    pub name: String,
    /// Input payload (a JSON value — keep it normalized).
    pub input: Value,
    /// Output payload.
    pub output: Value,
}

impl TraceCall {
    /// Construct an LLM call entry.
    pub fn llm(name: impl Into<String>, input: Value, output: Value) -> Self {
        Self {
            kind: TraceCallKind::Llm,
            name: name.into(),
            input,
            output,
        }
    }

    /// Construct a tool call entry.
    pub fn tool(name: impl Into<String>, input: Value, output: Value) -> Self {
        Self {
            kind: TraceCallKind::Tool,
            name: name.into(),
            input,
            output,
        }
    }
}

/// One full agent run, identified by a name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    /// Snapshot identifier (used as the file name when no explicit path).
    pub name: String,
    /// Ordered list of steps.
    pub calls: Vec<TraceCall>,
}

impl Trace {
    /// Empty trace named `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            calls: Vec::new(),
        }
    }

    /// Append a call.
    pub fn push(&mut self, call: TraceCall) {
        self.calls.push(call);
    }

    /// Number of recorded calls.
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// True if the trace has no calls.
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
}
