use agentsnap::{assert_matches, compare, record, MatchResult, Trace, TraceCall};
use serde_json::json;
use tempfile::tempdir;

fn sample_trace() -> Trace {
    let mut t = Trace::new("answer_user");
    t.push(TraceCall::llm("planner", json!({"q": "hi"}), json!({"plan": ["greet"]})));
    t.push(TraceCall::tool("greet", json!({"name": "world"}), json!("hi world")));
    t
}

#[test]
fn first_compare_records() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.snap.json");
    let trace = sample_trace();
    let result = compare(&trace, &path).unwrap();
    assert!(matches!(result, MatchResult::Recorded));
    assert!(path.exists());
}

#[test]
fn matching_trace_passes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.snap.json");
    let trace = sample_trace();
    record(&trace, &path).unwrap();
    let result = compare(&trace, &path).unwrap();
    assert!(matches!(result, MatchResult::Match));
}

#[test]
fn divergent_trace_returns_mismatch_with_diff() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.snap.json");
    record(&sample_trace(), &path).unwrap();

    let mut altered = sample_trace();
    altered.calls[1].output = json!("different output");
    let result = compare(&altered, &path).unwrap();
    match result {
        MatchResult::Mismatch(diff) => {
            assert!(diff.contains("hi world") || diff.contains("different output"));
            assert!(diff.contains("@@") || diff.contains("snapshot"));
        }
        _ => panic!("expected Mismatch"),
    }
}

#[test]
fn record_creates_parent_dirs() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nested/deep/a.snap.json");
    record(&sample_trace(), &path).unwrap();
    assert!(path.exists());
}

#[test]
fn assert_matches_first_run_passes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.snap.json");
    assert_matches(&sample_trace(), &path);
    assert!(path.exists());
}

#[test]
fn assert_matches_second_identical_run_passes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.snap.json");
    assert_matches(&sample_trace(), &path);
    assert_matches(&sample_trace(), &path);
}

#[test]
#[should_panic(expected = "agentsnap mismatch")]
fn assert_matches_panics_on_divergence() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.snap.json");
    record(&sample_trace(), &path).unwrap();
    let mut altered = sample_trace();
    altered.calls[1].output = json!("different");
    assert_matches(&altered, &path);
}

#[test]
fn update_env_var_refreshes_snapshot() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.snap.json");
    record(&sample_trace(), &path).unwrap();
    let mut altered = sample_trace();
    altered.calls[1].output = json!("refreshed");

    // SAFETY: tests run in parallel by default; we set then unset around the
    // single call so the env variable doesn't leak. Acceptable for a unit
    // test demonstrating behavior.
    std::env::set_var("AGENTSNAP_UPDATE", "1");
    assert_matches(&altered, &path);
    std::env::remove_var("AGENTSNAP_UPDATE");

    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("refreshed"));
}

#[test]
fn trace_roundtrip_serialization() {
    let trace = sample_trace();
    let s = serde_json::to_string(&trace).unwrap();
    let back: Trace = serde_json::from_str(&s).unwrap();
    assert_eq!(trace, back);
}
