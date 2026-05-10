use crate::trace::Trace;
use similar::TextDiff;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors from snapshot ops.
#[derive(Debug, Error)]
pub enum SnapError {
    /// File I/O failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Couldn't (de)serialize a snapshot file.
    #[error("snapshot format error: {0}")]
    Format(#[from] serde_json::Error),
}

/// Outcome of a [`compare`] call.
#[derive(Debug)]
pub enum MatchResult {
    /// No snapshot existed; one was just written.
    Recorded,
    /// Snapshot matched exactly.
    Match,
    /// Differences found — payload is a unified diff.
    Mismatch(String),
}

/// Write the trace as a snapshot at `path`, overwriting if present.
pub fn record(trace: &Trace, path: impl AsRef<Path>) -> Result<(), SnapError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(trace)?;
    fs::write(path, json)?;
    Ok(())
}

/// Compare a trace to the snapshot at `path`. Records on first run.
///
/// Returns:
/// - [`MatchResult::Recorded`] if no snapshot existed (one was created).
/// - [`MatchResult::Match`] if the trace matches the snapshot.
/// - [`MatchResult::Mismatch`] with a unified diff if they differ.
pub fn compare(trace: &Trace, path: impl AsRef<Path>) -> Result<MatchResult, SnapError> {
    let path: PathBuf = path.as_ref().to_path_buf();
    if !path.exists() {
        record(trace, &path)?;
        return Ok(MatchResult::Recorded);
    }
    let saved_text = fs::read_to_string(&path)?;
    let saved: Trace = serde_json::from_str(&saved_text)?;
    if &saved == trace {
        return Ok(MatchResult::Match);
    }
    let actual = serde_json::to_string_pretty(trace)?;
    let diff = TextDiff::from_lines(&saved_text, &actual)
        .unified_diff()
        .header("snapshot", "actual")
        .to_string();
    Ok(MatchResult::Mismatch(diff))
}

/// Test helper: compare-or-record, panic on mismatch.
///
/// Set `AGENTSNAP_UPDATE=1` to overwrite a stale snapshot instead of
/// panicking. Intended for use inside `#[test]` functions.
pub fn assert_matches(trace: &Trace, path: impl AsRef<Path>) {
    let path = path.as_ref();
    let update = std::env::var("AGENTSNAP_UPDATE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if update {
        record(trace, path).expect("agentsnap: record failed");
        return;
    }
    match compare(trace, path).expect("agentsnap: compare failed") {
        MatchResult::Recorded => {} // first run: pass and move on
        MatchResult::Match => {}
        MatchResult::Mismatch(diff) => panic!(
            "agentsnap mismatch at {}:\n{}\nRun with AGENTSNAP_UPDATE=1 to refresh.",
            path.display(),
            diff
        ),
    }
}
