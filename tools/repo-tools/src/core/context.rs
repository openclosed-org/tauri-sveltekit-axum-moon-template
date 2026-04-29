use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceContext {
    root: PathBuf,
}

impl WorkspaceContext {
    pub(crate) fn discover() -> Result<Self> {
        Ok(Self {
            root: std::env::current_dir().context("failed to read current directory")?,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    #[allow(dead_code)]
    pub(crate) fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    #[allow(dead_code)]
    pub(crate) fn read(&self, relative: impl AsRef<Path>) -> Result<String> {
        crate::core::fs::read(self.path(relative))
    }

    #[allow(dead_code)]
    pub(crate) fn relative(&self, path: &Path) -> Result<String> {
        Ok(crate::core::fs::normalize_slashes(
            path.strip_prefix(&self.root)?,
        ))
    }
}

pub(crate) fn workspace_root() -> Result<PathBuf> {
    WorkspaceContext::discover().map(|ctx| ctx.root)
}
