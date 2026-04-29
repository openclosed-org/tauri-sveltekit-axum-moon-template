use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::cli::{OpsArgs, OpsBootstrapVpsArgs, OpsCommand, OpsMigrateArgs, OpsMigrationDirection};
use crate::support::{has_tool, normalize_slashes, read, run_capture, workspace_root};

const MIGRATION_SERVICES: &[&str] = &[
    "user-service",
    "tenant-service",
    "auth-service",
    "counter-service",
    "settings-service",
];

const VPS_TOOLS: &[(&str, &str)] = &[
    ("git", "system package manager"),
    ("just", "https://just.systems/install.sh"),
    ("mise", "https://mise.run"),
    ("podman", "system package manager"),
    ("kubectl", "https://kubernetes.io/docs/tasks/tools/"),
    (
        "kustomize",
        "https://kubectl.docs.kubernetes.io/installation/kustomize/",
    ),
    ("age", "system package manager or GitHub release"),
    ("sops", "https://github.com/getsops/sops/releases"),
];

pub(crate) fn run(args: OpsArgs) -> Result<()> {
    match args.command {
        OpsCommand::Migrate(args) => migrate(args),
        OpsCommand::BootstrapVps(args) => bootstrap_vps(args),
    }
}

fn migrate(args: OpsMigrateArgs) -> Result<()> {
    if args.dry_run && args.apply {
        bail!("choose either --dry-run or --apply, not both");
    }

    let root = workspace_root()?;
    let plan = migration_plan(&root)?;

    println!("Migration environment: {}", args.env);
    println!("Migration direction: {:?}", args.direction);
    println!("Mode: {}", if args.apply { "apply" } else { "dry-run" });
    println!("Database target: {}", database_target(&args.env)?);
    println!();

    match args.direction {
        OpsMigrationDirection::Status => print_migration_status(&root, &plan),
        OpsMigrationDirection::Up => migrate_up(&root, &plan, args.apply),
        OpsMigrationDirection::Down => {
            println!(
                "Rollback is not implemented; create a forward migration that reverses the change."
            );
            if args.apply {
                bail!("migration rollback is not supported");
            }
            Ok(())
        }
        OpsMigrationDirection::Reset => {
            println!("Reset is destructive and is not implemented in repo-tools Phase 7.");
            println!(
                "Recovery guidance: stop services, back up data, remove only the intended local DB, then restart with migrations enabled."
            );
            if args.apply {
                bail!("migration reset is intentionally blocked");
            }
            Ok(())
        }
    }
}

fn migration_plan(root: &Path) -> Result<Vec<ServiceMigrations>> {
    let mut services = Vec::new();
    for service in MIGRATION_SERVICES {
        let dir = root.join("services").join(service).join("migrations");
        let migrations = if dir.is_dir() {
            let mut files = fs::read_dir(&dir)
                .with_context(|| format!("failed to read {}", dir.display()))?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
                .collect::<Vec<_>>();
            files.sort();
            files
        } else {
            Vec::new()
        };
        services.push(ServiceMigrations {
            name: (*service).to_string(),
            migrations,
        });
    }
    Ok(services)
}

fn print_migration_status(root: &Path, plan: &[ServiceMigrations]) -> Result<()> {
    println!("Migration status:");
    for service in plan {
        if service.migrations.is_empty() {
            println!("  {}: no migrations", service.name);
            continue;
        }
        println!(
            "  {}: {} migration(s)",
            service.name,
            service.migrations.len()
        );
        for migration in &service.migrations {
            println!("    - {}", normalize_slashes(migration.strip_prefix(root)?));
        }
    }
    Ok(())
}

fn migrate_up(root: &Path, plan: &[ServiceMigrations], apply: bool) -> Result<()> {
    print_migration_status(root, plan)?;
    if !apply {
        println!();
        println!("Dry run complete; no migration SQL was executed.");
        println!("Use --apply to run the Phase 7 sqlite3 syntax smoke for listed SQL files.");
        return Ok(());
    }

    if !has_tool("sqlite3") {
        bail!(
            "sqlite3 is required for Phase 7 migration apply smoke; install sqlite3 or run without --apply"
        );
    }

    for service in plan {
        for migration in &service.migrations {
            let sql = read(migration)?;
            let mut child = std::process::Command::new("sqlite3")
                .arg(":memory:")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .spawn()
                .with_context(|| format!("failed to start sqlite3 for {}", migration.display()))?;
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(sql.as_bytes())?;
            }
            let status = child.wait().with_context(|| {
                format!("failed to wait for sqlite3 for {}", migration.display())
            })?;
            if !status.success() {
                bail!("migration smoke failed: {}", migration.display());
            }
        }
    }
    println!("Migration apply smoke complete.");
    Ok(())
}

fn database_target(env: &str) -> Result<&'static str> {
    match env {
        "local" => Ok("libsql://localhost:8080"),
        "dev" => Ok("libsql://turso.app.svc.cluster.local:8080"),
        "staging" | "prod" => Ok("managed secret-backed database URL (not printed)"),
        other => bail!("unknown migration environment: {other}"),
    }
}

fn bootstrap_vps(args: OpsBootstrapVpsArgs) -> Result<()> {
    if args.plan && args.apply {
        bail!("choose either --plan or --apply, not both");
    }

    println!("VPS bootstrap plan:");
    println!("1. detect OS and package manager");
    println!("2. verify root privileges for host-level package installation");
    println!("3. install or verify git, just, mise, podman, kubectl, kustomize, age, and sops");
    println!("4. print post-install verification and next deploy steps");
    println!();
    vps_preflight()?;

    if !args.apply {
        println!("Plan complete; not modifying host.");
        println!("To execute in a future implementation, rerun with --apply --confirm <host>.");
        return Ok(());
    }

    let confirm = args
        .confirm
        .as_deref()
        .context("--apply requires --confirm <host>")?;
    if confirm.trim().is_empty() {
        bail!("--confirm must name the target host");
    }
    bail!(
        "VPS bootstrap apply is intentionally not implemented in Phase 7; no host changes were made"
    )
}

fn vps_preflight() -> Result<()> {
    let os = detected_os();
    println!("OS: {os}");
    println!("ARCH: {}", std::env::consts::ARCH);
    println!("Package manager: {}", package_manager(&os));
    println!("Root privileges: {}", root_status());
    println!();
    println!("Tool preflight:");
    for (tool, install_hint) in VPS_TOOLS {
        if has_tool(tool) {
            let version = tool_version(tool).unwrap_or_else(|_| "installed".to_string());
            println!("  - {tool}: {version}");
        } else {
            println!("  - {tool}: missing; install via {install_hint}");
        }
    }
    Ok(())
}

fn detected_os() -> String {
    let path = PathBuf::from("/etc/os-release");
    if path.is_file()
        && let Ok(content) = read(&path)
        && let Some(id) = content.lines().find_map(|line| {
            line.strip_prefix("ID=")
                .map(|value| value.trim_matches('"'))
        })
    {
        return id.to_string();
    }
    std::env::consts::OS.to_string()
}

fn package_manager(os: &str) -> &'static str {
    match os {
        "ubuntu" | "debian" => "apt-get",
        "rocky" | "centos" | "rhel" | "fedora" => "dnf",
        _ => "unsupported",
    }
}

fn root_status() -> String {
    #[cfg(unix)]
    {
        match run_capture("id", &["-u"], None) {
            Ok(outcome) if outcome.output.trim() == "0" => "root".to_string(),
            Ok(outcome) => format!("non-root uid {}", outcome.output.trim()),
            Err(_) => "unknown".to_string(),
        }
    }
    #[cfg(not(unix))]
    {
        "unsupported platform".to_string()
    }
}

fn tool_version(tool: &str) -> Result<String> {
    let args = match tool {
        "age" => vec!["-version"],
        "kubectl" => vec!["version", "--client"],
        _ => vec!["--version"],
    };
    let outcome = run_capture(tool, &args, None)?;
    if outcome.output.is_empty() {
        Ok("installed".to_string())
    } else {
        Ok(outcome
            .output
            .lines()
            .next()
            .unwrap_or("installed")
            .to_string())
    }
}

#[derive(Debug)]
struct ServiceMigrations {
    name: String,
    migrations: Vec<PathBuf>,
}
