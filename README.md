# agentsnap

[![crates.io](https://img.shields.io/crates/v/agentsnap.svg)](https://crates.io/crates/agentsnap)
[![docs.rs](https://docs.rs/agentsnap/badge.svg)](https://docs.rs/agentsnap)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Snapshot tests for AI agent traces. The Jest-snapshot pattern, applied to LLM agents.

```toml
[dev-dependencies]
agentsnap = "0.1"
```

## Why

The agent regression problem: today the agent answers correctly; tomorrow a small change to the system prompt or tool list silently breaks it. Manual eyeballing scales poorly. `agentsnap` is the smallest possible primitive that catches it: record a trace once, fail loud on any future divergence, refresh with one env var.

## Quick start

```rust,no_run
use agentsnap::{assert_matches, Trace, TraceCall};
use serde_json::json;

#[test]
fn answer_question() {
    let mut trace = Trace::new("answer_question");
    trace.push(TraceCall::llm(
        "planner",
        json!({"q": "what is 2+2?"}),
        json!({"plan": ["compute"]}),
    ));
    trace.push(TraceCall::tool(
        "compute",
        json!({"expr": "2+2"}),
        json!(4),
    ));
    trace.push(TraceCall::llm(
        "responder",
        json!({"result": 4}),
        json!("The answer is 4."),
    ));

    assert_matches(&trace, "tests/snapshots/answer_question.snap.json");
}
```

First run: writes the snapshot. Subsequent runs: compare. On mismatch, panics with a unified diff:

```text
agentsnap mismatch at tests/snapshots/answer_question.snap.json:
@@ -... @@
-      "output": 4
+      "output": "four"
Run with AGENTSNAP_UPDATE=1 to refresh.
```

## Updating snapshots

Intentional change? Refresh:

```bash
AGENTSNAP_UPDATE=1 cargo test
```

Then commit the updated snapshot files.

## API

```rust
Trace::new(name)
trace.push(TraceCall::llm(name, input, output))
trace.push(TraceCall::tool(name, input, output))

assert_matches(&trace, path)        // panics on mismatch (test helper)
compare(&trace, path)               // returns MatchResult::{Recorded,Match,Mismatch(diff)}
record(&trace, path)                // unconditionally write the snapshot
```

`Trace` and `TraceCall` derive `Serialize`/`Deserialize` — snapshots are plain JSON, easy to grep, easy to review in PRs.

## What it doesn't do

- Doesn't capture traces automatically. You build the `Trace` from your test (calling whatever your agent code returns).
- Doesn't tolerate non-determinism. Stub timestamps, IDs, and any clock-dependent output before snapshotting.
- Doesn't compare semantically — exact JSON equality. For LLM-output-as-ground-truth, layer an LLM-as-judge on top.

## Sibling: JS `@mukundakatta/agentsnap`

JS users: see [@mukundakatta/agentsnap](https://www.npmjs.com/package/@mukundakatta/agentsnap) on npm.

## License

MIT

## Repository Health

This repository includes a dependency-free health check for core documentation, metadata, and CI wiring. Run it locally before publishing changes:

```sh
python3 scripts/check_repository_health.py
```

The same check runs in GitHub Actions on pushes and pull requests.
