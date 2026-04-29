use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::cli::{AppsArgs, AppsCommand, AppsDevDesktopArgs, AppsE2eCommand};
use crate::support::{
    has_tool, normalize_slashes, run_capture, run_inherit, wait_for_port, workspace_root,
};

const API_HOST: &str = "127.0.0.1";
const API_PORT: u16 = 3010;
const WEB_PORT: u16 = 5173;
const API_WAIT_SECONDS: u64 = 180;

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

pub(crate) fn run(args: AppsArgs) -> Result<()> {
    match args.command {
        AppsCommand::E2e(args) => match args.command {
            AppsE2eCommand::Preflight => e2e_preflight(),
            AppsE2eCommand::Run => e2e_run(),
        },
        AppsCommand::DevDesktop(args) => dev_desktop(args),
    }
}

fn e2e_preflight() -> Result<()> {
    let root = workspace_root()?;
    let web_dir = root.join("apps/web");
    let desktop_e2e_dir = root.join("apps/desktop/tests/e2e");
    let tauri_dir = root.join("apps/desktop/src-tauri");

    println!("=== Apps E2E Preflight ===");
    require_existing_dir(&web_dir, "web app")?;
    require_existing_dir(&desktop_e2e_dir, "desktop e2e")?;
    require_existing_dir(&tauri_dir, "desktop tauri")?;
    require_existing_file(&web_dir.join("package.json"), "web package.json")?;
    require_existing_file(
        &desktop_e2e_dir.join("package.json"),
        "desktop e2e package.json",
    )?;
    require_existing_file(&tauri_dir.join("tauri.conf.json"), "tauri config")?;

    println!("tool preflight:");
    require_tool_for_apps("bun", "install Bun for app-shell commands")?;
    require_tool_for_apps("cargo", "install Rust/Cargo for Tauri desktop commands")?;
    require_tool_for_apps("cargo-tauri", "install Tauri CLI: cargo install tauri-cli")?;

    println!("runtime preflight:");
    if readyz_ok()? {
        println!("  - API readyz: ok at http://{API_HOST}:{API_PORT}/readyz");
    } else {
        bail!(
            "API runtime is not ready at http://{API_HOST}:{API_PORT}/readyz; start it with `rtk cargo run -p web-bff` or run `rtk just dev-desktop`"
        );
    }

    let svelte_types = web_dir.join(".svelte-kit/types");
    if svelte_types.exists() {
        println!("  - SvelteKit types: {}", normalize(&root, &svelte_types)?);
    } else {
        bail!(
            "missing required SvelteKit types directory {}; run `rtk bun run --cwd apps/web check`",
            normalize(&root, &svelte_types)?
        );
    }

    if is_port_open(WEB_PORT) {
        bail!(
            "port {WEB_PORT} is occupied before web lane bootstrap; stop the stale process first"
        );
    }
    if !is_port_open(API_PORT) {
        bail!("port {API_PORT} is not listening although readyz check passed");
    }
    println!("  - port hygiene: {WEB_PORT} free, {API_PORT} listening");
    println!("Apps E2E preflight passed.");
    Ok(())
}

fn e2e_run() -> Result<()> {
    e2e_preflight()?;
    let root = workspace_root()?;
    let web_dir = root.join("apps/web");
    let desktop_e2e_dir = root.join("apps/desktop/tests/e2e");

    println!("\n=== Web Playwright E2E ===");
    let web = run_inherit(bun_program(), &["run", "test:e2e"], Some(&web_dir))?;

    println!("\n=== Tauri Playwright Desktop E2E ===");
    let desktop = run_inherit(bun_program(), &["run", "test:ci"], Some(&desktop_e2e_dir))?;

    println!("\n=== Apps E2E Summary ===");
    print_lane("Web Playwright E2E", web);
    print_lane("Tauri Playwright Desktop E2E", desktop);

    if web != 0 || desktop != 0 {
        bail!("one or more app e2e lanes failed");
    }
    Ok(())
}

fn dev_desktop(args: AppsDevDesktopArgs) -> Result<()> {
    let root = workspace_root()?;
    let tauri_dir = root.join("apps/desktop/src-tauri");
    let web_dir = root.join("apps/web");

    println!("[dev-desktop] === Desktop Dev ===");
    println!("[dev-desktop] Workspace: {}", root.display());
    println!("[dev-desktop] Tauri dir: {}", tauri_dir.display());
    println!("[dev-desktop] Web dir: {}", web_dir.display());
    print_optimization_tips();

    require_existing_dir(&tauri_dir, "desktop tauri")?;
    require_existing_dir(&web_dir, "web app")?;
    require_existing_file(&tauri_dir.join("tauri.conf.json"), "tauri config")?;
    require_tool_for_apps("cargo", "install Rust/Cargo for Tauri desktop commands")?;
    require_tool_for_apps("cargo-tauri", "install Tauri CLI: cargo install tauri-cli")?;
    require_tool_for_apps("bun", "install Bun for Tauri beforeDevCommand")?;

    if args.dry_run {
        println!("[dev-desktop] Dry run; no child processes started.");
        println!("[dev-desktop] Would run: cargo run -p web-bff");
        println!("[dev-desktop] Would wait for {API_HOST}:{API_PORT} up to {API_WAIT_SECONDS}s");
        println!(
            "[dev-desktop] Would run in {}: cargo tauri dev",
            tauri_dir.display()
        );
        return Ok(());
    }

    println!("[dev-desktop] Step 1/2: Starting Axum API server...");
    ctrlc::set_handler(|| {
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
    })
    .context("failed to install Ctrl+C handler")?;

    let mut api = spawn_inherit("cargo", &["run", "-p", "web-bff"], &root)?;
    if !wait_for_port(API_HOST, API_PORT, Duration::from_secs(API_WAIT_SECONDS)) {
        println!(
            "[dev-desktop] WARNING: API server not ready after {API_WAIT_SECONDS}s; starting Tauri anyway"
        );
    }

    println!("[dev-desktop] Step 2/2: Starting Tauri desktop app...");
    let mut tauri = spawn_inherit("cargo", &["tauri", "dev"], &tauri_dir)?;

    println!("[dev-desktop] === Services Started ===");
    println!("[dev-desktop] Web BFF:  http://localhost:{API_PORT}");
    println!("[dev-desktop] Frontend: http://localhost:{WEB_PORT} (managed by Tauri)");
    println!("[dev-desktop] Press Ctrl+C to stop all services");

    let tauri_status = loop {
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            println!("[dev-desktop] Shutdown requested, stopping children...");
            terminate_child(&mut tauri, "Tauri")?;
            break tauri
                .wait()
                .context("failed to wait for Tauri desktop process")?;
        }
        if let Some(status) = tauri.try_wait()? {
            break status;
        }
        std::thread::sleep(Duration::from_millis(250));
    };
    println!("[dev-desktop] Tauri exited with status {tauri_status}");
    terminate_child(&mut api, "API server")?;
    Ok(())
}

fn require_existing_dir(path: &Path, label: &str) -> Result<()> {
    if path.is_dir() {
        println!("  - {label}: {}", path.display());
        return Ok(());
    }
    bail!("missing {label} directory: {}", path.display())
}

fn require_existing_file(path: &Path, label: &str) -> Result<()> {
    if path.is_file() {
        println!("  - {label}: {}", path.display());
        return Ok(());
    }
    bail!("missing {label}: {}", path.display())
}

fn require_tool_for_apps(tool: &str, hint: &str) -> Result<()> {
    if has_tool(tool) {
        let version = run_capture(tool, &["--version"], None)
            .map(|outcome| first_line(&outcome.output))
            .unwrap_or_else(|_| "installed".to_string());
        println!("  - {tool}: {version}");
        Ok(())
    } else {
        bail!("required app-shell tool `{tool}` is missing; {hint}")
    }
}

fn readyz_ok() -> Result<bool> {
    let url = format!("http://{API_HOST}:{API_PORT}/readyz");
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .context("failed to create HTTP client")?
        .get(url)
        .send();
    Ok(response
        .map(|resp| resp.status().is_success())
        .unwrap_or(false))
}

fn is_port_open(port: u16) -> bool {
    crate::support::wait_for_port(API_HOST, port, Duration::from_millis(1))
}

fn normalize(root: &Path, path: &Path) -> Result<String> {
    Ok(normalize_slashes(path.strip_prefix(root).unwrap_or(path)))
}

fn first_line(output: &str) -> String {
    output.lines().next().unwrap_or("installed").to_string()
}

fn bun_program() -> &'static str {
    if cfg!(windows) { "bun.cmd" } else { "bun" }
}

fn print_lane(name: &str, code: i32) {
    let status = if code == 0 { "PASS" } else { "FAIL" };
    println!("[{status}] {name} (exit {code})");
}

fn print_optimization_tips() {
    if std::env::var("RUSTC_WRAPPER").ok().as_deref() == Some("sccache") {
        println!("[dev-desktop] sccache enabled");
    } else {
        println!("[dev-desktop] sccache not enabled; optional: rtk just setup-sccache");
    }
    if std::env::var("CARGO_HAKARI").ok().as_deref() == Some("0") {
        println!("[dev-desktop] cargo-hakari disabled");
    } else {
        println!("[dev-desktop] cargo-hakari enabled by default");
    }
}

fn spawn_inherit(program: &str, args: &[&str], cwd: &Path) -> Result<Child> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
        .spawn()
        .with_context(|| format!("failed to start {program} {}", args.join(" ")))
}

fn terminate_child(child: &mut Child, label: &str) -> Result<()> {
    match child.try_wait()? {
        Some(status) => {
            println!("[dev-desktop] {label} already exited with status {status}");
        }
        None => {
            println!("[dev-desktop] Stopping {label}...");
            kill_child_tree(child, label)?;
            let _ = child.wait();
        }
    }
    Ok(())
}

#[cfg(windows)]
fn kill_child_tree(child: &mut Child, label: &str) -> Result<()> {
    let pid = child.id().to_string();
    let status = Command::new("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .status()
        .with_context(|| format!("failed to run taskkill for {label}"))?;
    if !status.success() {
        child
            .kill()
            .with_context(|| format!("failed to stop {label}"))?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn kill_child_tree(child: &mut Child, label: &str) -> Result<()> {
    child
        .kill()
        .with_context(|| format!("failed to stop {label}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bun_program_uses_platform_specific_name() {
        if cfg!(windows) {
            assert_eq!(bun_program(), "bun.cmd");
        } else {
            assert_eq!(bun_program(), "bun");
        }
    }

    #[test]
    fn first_line_falls_back_for_empty_output() {
        assert_eq!(first_line(""), "installed");
        assert_eq!(first_line("bun 1.3.12\nextra"), "bun 1.3.12");
    }
}
