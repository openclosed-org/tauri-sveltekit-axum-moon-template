use std::process::{Command, Stdio};

use anyhow::{Result, bail};

use crate::core::command::run_capture;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tool {
    Cargo,
    Just,
    Moon,
    Git,
    Podman,
    Kubectl,
    Sops,
    Age,
    Yq,
    Flux,
    K3s,
}

impl Tool {
    #[allow(dead_code)]
    pub(crate) fn binary(self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Just => "just",
            Self::Moon => "moon",
            Self::Git => "git",
            Self::Podman => "podman",
            Self::Kubectl => "kubectl",
            Self::Sops => "sops",
            Self::Age => "age",
            Self::Yq => "yq",
            Self::Flux => "flux",
            Self::K3s => "k3s",
        }
    }
}

pub(crate) fn has_tool(tool: &str) -> bool {
    let checker = if cfg!(windows) { "where" } else { "which" };
    Command::new(checker)
        .arg(tool)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[allow(dead_code)]
pub(crate) fn require_tool(tool: &str, hint: &str) -> Result<()> {
    if has_tool(tool) {
        Ok(())
    } else {
        bail!("missing tool '{tool}'. install hint: {hint}")
    }
}

#[allow(dead_code)]
pub(crate) fn command_exists_output(tool: &str) -> Result<String> {
    let outcome = run_capture(tool, &["--version"], None)?;
    if outcome.success {
        Ok(outcome.output)
    } else {
        bail!("{tool} --version failed: {}", outcome.error)
    }
}
