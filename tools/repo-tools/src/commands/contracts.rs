use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::support;
use crate::support::{
    Issue, Mode, Report, collect_files_named, collect_files_with_extension, list_directories,
    normalize_slashes, read, run_capture, run_inherit, strip_rust_comments, workspace_root,
};

pub(crate) fn boundary_check() -> Result<()> {
    let root = workspace_root()?;
    let metadata = support::cargo_metadata(&root)?;
    let packages = metadata["packages"]
        .as_array()
        .context("cargo metadata missing packages array")?;
    let rules: [(&str, &[&str], &str); 9] = [
        (
            "kernel",
            &["async-trait", "serde", "serde_json"],
            r"^(storage_|runtime_|contracts_|counter-service|auth-service|tenant-service|user-service)",
        ),
        (
            "contracts_api",
            &["serde", "utoipa", "validator"],
            r"^(kernel|storage_|runtime_|counter-service|auth-service|tenant-service|user-service)",
        ),
        (
            "contracts_auth",
            &["serde", "utoipa", "validator"],
            r"^(kernel|storage_|runtime_|counter-service|tenant-service|user-service)",
        ),
        (
            "contracts_events",
            &["serde", "utoipa", "validator"],
            r"^(kernel|storage_|runtime_|counter-service|auth-service|tenant-service|user-service)",
        ),
        (
            "contracts_errors",
            &["serde", "utoipa", "validator"],
            r"^(kernel|storage_|runtime_|counter-service|auth-service|tenant-service|user-service)",
        ),
        (
            "counter-service",
            &[
                "async-trait",
                "serde",
                "serde_json",
                "thiserror",
                "contracts_events",
                "contracts_errors",
                "kernel",
                "data",
            ],
            r"^(storage_|runtime_|auth-service|tenant-service|user-service)",
        ),
        (
            "auth-service",
            &[
                "async-trait",
                "serde",
                "serde_json",
                "thiserror",
                "contracts_auth",
                "contracts_errors",
                "kernel",
                "data",
            ],
            r"^(storage_|runtime_|counter-service|tenant-service|user-service)",
        ),
        (
            "tenant-service",
            &[
                "async-trait",
                "serde",
                "serde_json",
                "thiserror",
                "contracts_errors",
                "kernel",
                "data",
            ],
            r"^(storage_|runtime_|counter-service|auth-service|user-service)",
        ),
        (
            "user-service",
            &[
                "async-trait",
                "serde",
                "serde_json",
                "thiserror",
                "contracts_errors",
                "kernel",
                "data",
            ],
            r"^(storage_|runtime_|counter-service|auth-service|tenant-service)",
        ),
    ];

    let mut all_clean = true;
    println!("=== Architecture Boundary Check ===\n");
    println!("Rules: services MUST NOT depend on other services\n");
    println!("Rules: contracts MUST be Single Source of Truth for shared types\n");
    for (package_name, allowed_patterns, disallowed_pattern) in rules {
        println!("=== Checking {package_name} dependencies ===");
        let package = packages
            .iter()
            .find(|pkg| pkg["name"].as_str() == Some(package_name));
        let Some(package) = package else {
            println!("Warning: Could not get dependency metadata for {package_name}");
            continue;
        };
        let regex = Regex::new(disallowed_pattern)?;
        let mut violations = Vec::new();
        if let Some(dependencies) = package["dependencies"].as_array() {
            for dependency in dependencies {
                if dependency["kind"]
                    .as_str()
                    .is_some_and(|kind| kind != "normal")
                {
                    continue;
                }
                let Some(name) = dependency["name"].as_str() else {
                    continue;
                };
                if name == package_name
                    || allowed_patterns
                        .iter()
                        .any(|pattern| name.contains(pattern))
                {
                    continue;
                }
                if regex.is_match(name) {
                    violations.push(name.to_string());
                }
            }
        }
        if violations.is_empty() {
            println!("OK: {package_name} boundary clean");
        } else {
            all_clean = false;
            println!("FAIL: {package_name} depends on illegal crates:");
            for violation in violations {
                println!("  - {violation}");
            }
        }
    }
    if all_clean {
        println!("\nAll boundary checks passed");
        Ok(())
    } else {
        bail!("boundary check failed - review architectural dependencies")
    }
}

pub(crate) fn typegen() -> Result<()> {
    println!("=== Running contract generation ===\n");
    println!("[1/3] Cleaning stale generated contract outputs...");
    let root = workspace_root()?;
    let stale_paths = [
        root.join("packages/contracts/generated/api"),
        root.join("packages/contracts/generated/auth"),
        root.join("packages/contracts/generated/events"),
        root.join("packages/contracts/api/bindings"),
        root.join("packages/contracts/auth/bindings"),
        root.join("packages/contracts/events/bindings"),
    ];
    for path in stale_paths {
        if path.exists() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            println!("  Removed {}", normalize_slashes(path.strip_prefix(&root)?));
        }
    }
    println!();

    println!("[2/3] Exporting web-bff OpenAPI...");
    let status = run_inherit(
        "cargo",
        &[
            "run",
            "-p",
            "web-bff",
            "--bin",
            "export-openapi",
            "--",
            "--format",
            "yaml",
            "--output",
            "packages/contracts/generated/openapi/web-bff.openapi.yaml",
        ],
        None,
    )?;
    if status != 0 {
        bail!("OpenAPI generation failed");
    }
    let artifact = root.join("packages/contracts/generated/openapi/web-bff.openapi.yaml");
    if !artifact.exists() {
        bail!("OpenAPI artifact was not generated: {}", artifact.display());
    }
    println!(
        "  Generated {}\n",
        normalize_slashes(artifact.strip_prefix(&root)?)
    );

    println!("[3/3] Backend generated contracts ready.");
    println!("\n=== Contract generation complete ===\n");
    Ok(())
}

pub(crate) fn drift_check() -> Result<()> {
    check_git_drift(
        "contract drift",
        "packages/contracts/generated/",
        "DRIFT CHECK PASSED",
        "DRIFT DETECTED: Run 'just typegen' to regenerate",
    )
}

pub(crate) fn sdk_drift_check() -> Result<()> {
    let root = workspace_root()?;
    let sdk_root = root.join("packages/sdk");
    println!("=== Checking SDK drift ===");
    let generated_files = collect_files_with_extension(&sdk_root, "ts")
        .into_iter()
        .filter(|path| {
            read(path)
                .map(|content| content.contains("This file was generated"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    if generated_files.is_empty() {
        println!("SDK DRIFT CHECK PASSED");
        println!(
            "No generated SDK files found under packages/sdk; hand-written SDK backends are not drift-checked here."
        );
        return Ok(());
    }

    let mut drifted = Vec::new();
    for file in generated_files {
        let relative = normalize_slashes(file.strip_prefix(&root)?);
        if !git_dirty_paths(&root, &relative)?.is_empty() {
            drifted.push(relative);
        }
    }

    if drifted.is_empty() {
        println!("SDK DRIFT CHECK PASSED");
        return Ok(());
    }

    println!("SDK DRIFT DETECTED: Run 'just typegen' to regenerate");
    for file in drifted {
        println!("{file}");
    }
    bail!("SDK drift detected")
}

fn check_git_drift(label: &str, pathspec: &str, pass: &str, fail: &str) -> Result<()> {
    let root = workspace_root()?;
    println!("=== Checking {label} ===");
    let dirty_paths = git_dirty_paths(&root, pathspec)?;
    if dirty_paths.is_empty() {
        println!("{pass}");
        return Ok(());
    }
    println!("{fail}");
    for path in dirty_paths {
        println!("{path}");
    }
    bail!("{label} detected")
}

fn git_dirty_paths(root: &Path, pathspec: &str) -> Result<Vec<String>> {
    let status = run_capture(
        "git",
        &["status", "--porcelain=v1", "--", pathspec],
        Some(root),
    )?;
    if !status.success {
        bail!("git status failed for {pathspec}: {}", status.error);
    }
    Ok(parse_git_status_paths(&status.output))
}

fn parse_git_status_paths(output: &str) -> Vec<String> {
    let mut paths = output
        .lines()
        .filter_map(|line| line.get(2..))
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| path.split(" -> ").last().unwrap_or(path).to_string())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

pub(crate) fn validate_contract_boundaries(mode: Mode) -> Result<()> {
    let root = workspace_root()?;
    let services_root = root.join("services");
    if !services_root.exists() {
        println!("No services directory found; skipping contract boundary validation");
        return Ok(());
    }

    let mut issues = Vec::new();
    let app_event_pattern =
        Regex::new(r"contracts_events::AppEvent|use\s+contracts_events::\{[^}]*AppEvent")?;
    let local_event_pattern = Regex::new(r"pub\s+enum\s+\w+Event|pub\s+struct\s+\w+Event")?;
    let outbox_pattern = Regex::new(r"INSERT\s+INTO\s+event_outbox|UPDATE\s+event_outbox")?;
    for service_dir in list_directories(&services_root)? {
        let src_dir = service_dir.join("src");
        if !src_dir.exists() {
            continue;
        }
        let rust_files = collect_files_with_extension(&src_dir, "rs");
        let service_has_shared_app_event = rust_files.iter().any(|file| {
            read(file)
                .map(|content| app_event_pattern.is_match(&strip_rust_comments(&content)))
                .unwrap_or(false)
        });
        for file in rust_files {
            let relative = normalize_slashes(file.strip_prefix(&root)?);
            let code = strip_rust_comments(&read(&file)?);
            if local_event_pattern.is_match(&code) && outbox_pattern.is_match(&code) {
                issues.push((true, relative.clone(), "service-local event definitions must not live in files that write to event_outbox; promote cross-boundary events to contracts_events::AppEvent first".to_string()));
            }
            if outbox_pattern.is_match(&code) && !service_has_shared_app_event {
                issues.push((true, relative, "services writing to event_outbox must define the cross-process payload via contracts_events::AppEvent somewhere in the same service crate".to_string()));
            }
        }
    }

    let mut report = Report::new("validate-contract-boundaries", mode);
    report.extend(issues.iter().map(|(error, scope, message)| {
        if *error {
            Issue::error(scope.clone(), message.clone())
        } else {
            Issue::warn(scope.clone(), message.clone())
        }
    }));
    report.print();
    if issues.is_empty() {
        println!("✓ shared contract boundaries clean");
        return Ok(());
    }
    report.exit_if_needed();
    Ok(())
}

pub(crate) fn validate_contracts(mode: Mode) -> Result<()> {
    let root = workspace_root()?;
    let contracts_dir = root.join("packages/contracts");
    let servers_dir = root.join("servers");
    let contract_type_pattern = Regex::new(r"pub\s+(struct|enum)\s+(\w+)")?;
    let contract_dep_pattern = Regex::new(r"^\s*(contracts_\w+)\s*=")?;

    let mut contract_crates = Vec::new();
    if contracts_dir.exists() {
        for entry in list_directories(&contracts_dir)? {
            let Some(name) = entry.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if matches!(name, "bindings" | "generated") {
                continue;
            }
            let cargo_toml = entry.join("Cargo.toml");
            let lib_rs = entry.join("src/lib.rs");
            if !cargo_toml.exists() || !lib_rs.exists() {
                continue;
            }
            let cargo: toml::Value = toml::from_str(&read(&cargo_toml)?)?;
            let package_name = cargo
                .get("package")
                .and_then(|value| value.get("name"))
                .and_then(toml::Value::as_str)
                .unwrap_or(name)
                .to_string();
            let lib_content = read(&lib_rs)?;
            let mut exported_types = Vec::new();
            for captured in contract_type_pattern.captures_iter(&lib_content) {
                let Some(full_match) = captured.get(0) else {
                    continue;
                };
                let Some(type_name) = captured.get(2) else {
                    continue;
                };
                let context = lib_content[..full_match.start()]
                    .chars()
                    .rev()
                    .take(200)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<String>();
                if context.contains("ToSchema") {
                    exported_types.push(type_name.as_str().to_string());
                }
            }
            exported_types.sort();
            contract_crates.push((
                package_name,
                normalize_slashes(entry.strip_prefix(&root)?),
                exported_types,
            ));
        }
    }
    contract_crates.sort_by(|left, right| left.0.cmp(&right.0));

    let mut server_modules = Vec::new();
    if servers_dir.exists() {
        for manifest in collect_files_named(&servers_dir, "Cargo.toml") {
            let Some(server_dir) = manifest.parent() else {
                continue;
            };
            let cargo_content = read(&manifest)?;
            let has_static_openapi =
                server_dir.join("openapi.yaml").exists() || server_dir.join("openapi.yml").exists();
            let has_generated_openapi = root
                .join("packages/contracts/generated/openapi")
                .join(format!(
                    "{}.openapi.yaml",
                    server_dir
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or_default()
                ))
                .exists();
            let has_runtime_openapi = [
                server_dir.join("src/lib.rs"),
                server_dir.join("src/main.rs"),
            ]
            .into_iter()
            .filter(|path| path.exists())
            .any(|path| {
                read(path)
                    .map(|content| {
                        content.contains("utoipa_scalar")
                            || content.contains("Scalar::new")
                            || content.contains("\"/scalar\"")
                    })
                    .unwrap_or(false)
            });
            let has_openapi = has_static_openapi || has_generated_openapi || has_runtime_openapi;
            let has_handlers = [server_dir.join("handlers"), server_dir.join("src/handlers")]
                .into_iter()
                .filter(|dir| dir.exists())
                .any(|dir| {
                    fs::read_dir(dir)
                        .map(|entries| {
                            entries.filter_map(Result::ok).any(|entry| {
                                entry
                                    .file_type()
                                    .map(|kind| kind.is_file())
                                    .unwrap_or(false)
                                    && entry.file_name() != ".gitkeep"
                                    && entry.file_name().to_string_lossy().ends_with(".rs")
                            })
                        })
                        .unwrap_or(false)
                });
            let has_routes =
                server_dir.join("routes").exists() || server_dir.join("src/routes").exists();
            let mut contract_dependencies = cargo_content
                .lines()
                .filter_map(|line| contract_dep_pattern.captures(line))
                .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
                .collect::<Vec<_>>();
            contract_dependencies.sort();
            contract_dependencies.dedup();
            server_modules.push((
                server_dir
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
                    .to_string(),
                normalize_slashes(server_dir.strip_prefix(&root)?),
                has_openapi,
                has_runtime_openapi || has_generated_openapi,
                has_handlers,
                has_routes,
                contract_dependencies,
            ));
        }
    }
    server_modules.sort_by(|left, right| left.0.cmp(&right.0));

    let mut workspace_usage: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for scope in ["packages", "services", "servers", "workers", "platform"] {
        let base = root.join(scope);
        if !base.exists() {
            continue;
        }
        for manifest in collect_files_named(&base, "Cargo.toml") {
            if manifest.starts_with(&contracts_dir) {
                continue;
            }
            let Some(consumer_dir) = manifest.parent() else {
                continue;
            };
            let consumer = normalize_slashes(consumer_dir.strip_prefix(&root)?);
            for line in read(&manifest)?.lines() {
                if let Some(captures) = contract_dep_pattern.captures(line) {
                    let Some(dep) = captures.get(1) else {
                        continue;
                    };
                    workspace_usage
                        .entry(dep.as_str().to_string())
                        .or_default()
                        .push(consumer.clone());
                }
            }
        }
    }
    for consumers in workspace_usage.values_mut() {
        consumers.sort();
        consumers.dedup();
    }

    let mut issues = Vec::new();
    if root.join("docs/contracts/api-routes.yaml").exists() {
        let message = "hand-written canonical OpenAPI YAML is not allowed; generate packages/contracts/generated/openapi/web-bff.openapi.yaml from Rust/Axum sources".to_string();
        issues.push((
            mode.is_strict(),
            "docs/contracts/api-routes.yaml".to_string(),
            message,
        ));
    }
    if root.join("packages/contracts/generated/api").exists() {
        issues.push((
            mode.is_strict(),
            "packages/contracts/generated/api".to_string(),
            "HTTP API TypeScript generated artifacts are retired; use generated OpenAPI instead"
                .to_string(),
        ));
    }
    if root
        .join("packages/contracts/generated/openapi/web-bff.openapi.yaml")
        .exists()
    {
        let content =
            read(&root.join("packages/contracts/generated/openapi/web-bff.openapi.yaml"))?;
        if !content.contains("openapi: 3.1.0") {
            issues.push((
                mode.is_strict(),
                "packages/contracts/generated/openapi/web-bff.openapi.yaml".to_string(),
                "generated web-bff OpenAPI artifact must use OpenAPI 3.1.0".to_string(),
            ));
        }
    } else {
        issues.push((
            mode.is_strict(),
            "packages/contracts/generated/openapi/web-bff.openapi.yaml".to_string(),
            "missing generated web-bff OpenAPI artifact; run just typegen".to_string(),
        ));
    }
    for (crate_name, crate_path, _) in &contract_crates {
        let used_by_server = server_modules
            .iter()
            .any(|server| server.6.contains(crate_name));
        if !used_by_server && workspace_usage.get(crate_name).is_none_or(Vec::is_empty) {
            issues.push((false, crate_path.clone(), format!("contract crate '{crate_name}' is not directly depended on by any workspace crate")));
        }
    }
    for (
        _server_name,
        server_path,
        has_openapi,
        has_runtime_openapi,
        has_handlers,
        _has_routes,
        contract_dependencies,
    ) in &server_modules
    {
        if *has_handlers && !*has_openapi {
            issues.push((
                mode.is_strict(),
                server_path.clone(),
                "has handlers but missing openapi documentation (static or runtime via /scalar)"
                    .to_string(),
            ));
        }
        if *has_handlers && contract_dependencies.is_empty() {
            issues.push((false, server_path.clone(), "has handlers but does not depend on any contracts_* crate — verify this is intentional".to_string()));
        }
        if *has_runtime_openapi || !*has_openapi {
            continue;
        }
        let openapi_yaml = root.join(server_path).join("openapi.yaml");
        match read(&openapi_yaml) {
            Ok(content) => {
                if (content.contains("paths: {}") || content.contains("paths:{}")) && *has_handlers
                {
                    issues.push((false, server_path.clone(), "openapi.yaml has empty paths section but handlers exist — consider documenting endpoints".to_string()));
                }
            }
            Err(_) => issues.push((
                mode.is_strict(),
                server_path.clone(),
                "openapi.yaml exists but could not be read".to_string(),
            )),
        }
    }

    let type_to_crate = contract_crates
        .iter()
        .flat_map(|(crate_name, _, exported_types)| {
            exported_types
                .iter()
                .map(move |type_name| (type_name.clone(), crate_name.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    for (
        _server_name,
        server_path,
        _has_openapi,
        _has_runtime_openapi,
        has_handlers,
        _has_routes,
        contract_dependencies,
    ) in &server_modules
    {
        if !*has_handlers {
            continue;
        }
        let mut uses_contract_types = false;
        for dir in [
            root.join(server_path).join("handlers"),
            root.join(server_path).join("src/handlers"),
        ] {
            if !dir.exists() {
                continue;
            }
            for handler in collect_files_with_extension(&dir, "rs") {
                let content = read(&handler)?;
                if type_to_crate
                    .keys()
                    .any(|type_name| content.contains(type_name))
                {
                    uses_contract_types = true;
                    break;
                }
            }
            if uses_contract_types {
                break;
            }
        }
        if !contract_dependencies.is_empty() && !uses_contract_types {
            issues.push((
                false,
                server_path.clone(),
                format!(
                    "depends on contracts but may not use exported types: {}",
                    contract_dependencies.join(", ")
                ),
            ));
        }
    }

    let mut report = Report::new("validate-contracts", mode);
    report.extend(issues.iter().map(|(error, scope, message)| {
        if *error {
            Issue::error(scope.clone(), message.clone())
        } else {
            Issue::warn(scope.clone(), message.clone())
        }
    }));
    report.print();
    println!("\n--- Contract-Coverage Summary ---");
    println!("Contract crates discovered: {}", contract_crates.len());
    println!("Server modules discovered: {}", server_modules.len());
    if !contract_crates.is_empty() {
        println!("\nContract crates:");
        for (crate_name, _, exported_types) in &contract_crates {
            println!(
                "  - {} ({} exported types)",
                crate_name,
                exported_types.len()
            );
        }
    }
    if !server_modules.is_empty() {
        println!("\nServer modules:");
        for (
            server_name,
            _path,
            has_openapi,
            has_runtime_openapi,
            has_handlers,
            has_routes,
            contract_dependencies,
        ) in &server_modules
        {
            let flags = [
                has_handlers.then_some("handlers"),
                has_routes.then_some("routes"),
                has_openapi.then_some(if *has_runtime_openapi {
                    "runtime-openapi"
                } else {
                    "static-openapi"
                }),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(", ");
            let deps = if contract_dependencies.is_empty() {
                String::new()
            } else {
                format!(" [contracts: {}]", contract_dependencies.join(", "))
            };
            println!("  - {} [{}]{}", server_name, flags, deps);
        }
    }
    println!(
        "\nContract issues: {} error(s), {} warning(s)",
        report.error_count(),
        report.warning_count()
    );
    if issues.is_empty() {
        println!("\nNo contract issues found");
        return Ok(());
    }
    report.exit_if_needed();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_git_status_paths;

    #[test]
    fn parse_git_status_paths_reports_tracked_and_untracked_files() {
        let output = " M packages/contracts/generated/openapi/web-bff.openapi.yaml\n?? packages/contracts/generated/openapi/extra.openapi.yaml\nR  old/path.yaml -> packages/contracts/generated/openapi/renamed.openapi.yaml";

        assert_eq!(
            parse_git_status_paths(output),
            vec![
                "packages/contracts/generated/openapi/extra.openapi.yaml".to_string(),
                "packages/contracts/generated/openapi/renamed.openapi.yaml".to_string(),
                "packages/contracts/generated/openapi/web-bff.openapi.yaml".to_string(),
            ]
        );
    }
}
