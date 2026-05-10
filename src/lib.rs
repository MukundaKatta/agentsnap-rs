//! Snapshot tests for AI agent traces.
//!
//! The Jest-snapshot pattern, applied to agent runs. Build a [`Trace`] of
//! LLM calls and tool invocations from your test, then call
//! [`assert_matches`]: the first run records the snapshot to disk; later
//! runs compare against it and panic with a unified diff if anything
//! changed. Set `AGENTSNAP_UPDATE=1` in the environment to refresh the
//! snapshot.
//!
//! # Quick start
//!
//! ```no_run
//! use agentsnap::{assert_matches, Trace, TraceCall};
//! use serde_json::json;
//!
//! let mut trace = Trace::new("answer_user");
//! trace.push(TraceCall::llm("planner", json!({"q": "hi"}), json!({"plan": ["greet"]})));
//! trace.push(TraceCall::tool("greet_tool", json!({"name": "world"}), json!("hi world")));
//!
//! // First run: writes snapshots/answer_user.snap.json
//! // Subsequent runs: panics if anything diverges
//! assert_matches(&trace, "snapshots/answer_user.snap.json");
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod snap;
mod trace;

pub use crate::snap::{assert_matches, compare, record, MatchResult, SnapError};
pub use crate::trace::{Trace, TraceCall, TraceCallKind};
