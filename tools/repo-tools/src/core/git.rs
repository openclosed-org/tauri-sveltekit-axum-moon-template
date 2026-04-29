use std::path::Path;

use anyhow::Result;

use crate::core::command::run_capture;

pub(crate) fn git_changed_paths(root: &Path) -> Result<Vec<String>> {
    let staged = run_capture(
        "git",
        &["diff", "--staged", "--name-only", "--diff-filter=ACMR"],
        Some(root),
    )?;
    if staged.success && !staged.output.is_empty() {
        return Ok(staged.output.lines().map(ToOwned::to_owned).collect());
    }

    let unstaged = run_capture("git", &["diff", "--name-only"], Some(root))?;
    if unstaged.success && !unstaged.output.is_empty() {
        return Ok(unstaged.output.lines().map(ToOwned::to_owned).collect());
    }

    Ok(Vec::new())
}
