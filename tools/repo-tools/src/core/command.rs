use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub(crate) struct CommandOutcome {
    pub(crate) success: bool,
    pub(crate) output: String,
    pub(crate) error: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct CommandRunner {
    cwd: Option<PathBuf>,
    dry_run: bool,
}

impl CommandRunner {
    pub(crate) fn new(cwd: Option<&Path>) -> Self {
        Self {
            cwd: cwd.map(Path::to_path_buf),
            dry_run: false,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub(crate) fn capture(&self, program: &str, args: &[&str]) -> Result<CommandOutcome> {
        let spec = Cmd::new(program, args).with_cwd(self.cwd.as_deref());
        if self.dry_run {
            return Ok(CommandOutcome {
                success: true,
                output: format!("DRY-RUN: {}", spec.display()),
                error: String::new(),
                exit_code: 0,
            });
        }
        spec.capture()
    }

    pub(crate) fn inherit(&self, program: &str, args: &[&str]) -> Result<i32> {
        let spec = Cmd::new(program, args).with_cwd(self.cwd.as_deref());
        if self.dry_run {
            println!("DRY-RUN: {}", spec.display());
            return Ok(0);
        }
        spec.inherit()
    }
}

#[derive(Debug, Clone)]
struct Cmd {
    program: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    env: BTreeMap<String, String>,
}

impl Cmd {
    fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            cwd: None,
            env: BTreeMap::new(),
        }
    }

    fn with_cwd(mut self, cwd: Option<&Path>) -> Self {
        self.cwd = cwd.map(Path::to_path_buf);
        self
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        for (key, value) in &self.env {
            command.env(key, value);
        }
        command
    }

    fn capture(&self) -> Result<CommandOutcome> {
        let output = self
            .command()
            .output()
            .with_context(|| format!("failed to run command: {}", self.display()))?;

        Ok(CommandOutcome {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            error: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(1),
        })
    }

    fn inherit(&self) -> Result<i32> {
        let mut command = self.command();
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let status = command
            .status()
            .with_context(|| format!("failed to run command: {}", self.display()))?;
        Ok(status.code().unwrap_or(1))
    }

    fn display(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

pub(crate) fn run_capture(
    program: &str,
    args: &[&str],
    cwd: Option<&Path>,
) -> Result<CommandOutcome> {
    CommandRunner::new(cwd).capture(program, args)
}

pub(crate) fn run_inherit(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<i32> {
    CommandRunner::new(cwd).inherit(program, args)
}
