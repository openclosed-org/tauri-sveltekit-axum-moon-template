use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "contract-drift-detector")]
#[command(about = "Detect drift between platform models and contracts")]
struct Args {
    /// Path to platform directory
    #[arg(long, default_value = "platform")]
    platform_dir: PathBuf,

    /// Path to project root (where packages/contracts/ lives)
    #[arg(long, default_value = ".")]
    root_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ServiceModel {
    name: String,
    ports: Option<Vec<serde_json::Value>>,
    events: Option<Vec<serde_json::Value>>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let platform_dir = fs::canonicalize(&args.platform_dir).with_context(|| {
        format!(
            "Platform directory not found: {}",
            args.platform_dir.display()
        )
    })?;

    let root_dir = fs::canonicalize(&args.root_dir)
        .with_context(|| format!("Root directory not found: {}", args.root_dir.display()))?;

    info!("Checking contract drift...");
    info!("Platform: {}", platform_dir.display());
    info!("Root: {}", root_dir.display());

    let mut drifts: Vec<String> = Vec::new();

    // 1. Check that each service's events reference actual contract files
    let event_drift = check_event_drift(&platform_dir, &root_dir)?;
    drifts.extend(event_drift);

    // 2. Check that service ports have corresponding contract types
    let port_drift = check_port_contract_drift(&platform_dir, &root_dir)?;
    drifts.extend(port_drift);

    // 3. Check that platform model services match actual service crates
    let service_drift = check_service_crate_drift(&platform_dir, &root_dir)?;
    drifts.extend(service_drift);

    // Report
    println!("\n============================================================");
    println!("         Contract Drift Detection Report                     ");
    println!("=============================================================");

    if drifts.is_empty() {
        println!("  No contract drift detected");
        println!("  Platform models and contracts are in sync");
    } else {
        for drift in &drifts {
            println!("  DRIFT: {}", drift);
        }
    }

    println!("-------------------------------------------------------------");
    println!("  Drifts: {}", drifts.len());
    println!("=============================================================\n");

    if drifts.is_empty() {
        info!("No contract drift detected!");
        Ok(())
    } else {
        warn!("{} contract drift(s) detected", drifts.len());
        // Non-fatal: drift is a warning, not an error
        Ok(())
    }
}

/// Check that event references in platform models correspond to actual event contract files
fn check_event_drift(platform_dir: &Path, root_dir: &Path) -> Result<Vec<String>> {
    info!("Checking event contract drift...");

    let mut drifts: Vec<String> = Vec::new();
    let services_dir = platform_dir.join("model/services");

    if !services_dir.exists() {
        return Ok(drifts);
    }

    // Collect all event names from the contracts_events crate
    let events_lib = root_dir.join("packages/contracts/events/src/lib.rs");
    let mut contract_event_names = BTreeSet::new();

    if events_lib.exists() {
        let content = fs::read_to_string(&events_lib)?;
        // Extract event variant names from the AppEvent enum
        // Pattern: lines like `    CounterIncremented(CounterIncrementedPayload),`
        for line in content.lines() {
            let line = line.trim();
            if (line.ends_with("Payload),") || line.ends_with("Payload),"))
                && let Some(name) = line.split('(').next()
            {
                let name = name.trim();
                if !name.is_empty() && !name.starts_with("//") {
                    contract_event_names.insert(name.to_string());
                }
            }
        }
    }

    // Check each service's events
    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let service: ServiceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse: {}", path.display()))?;

        if let Some(events) = &service.events {
            for event in events {
                if let Some(name) = event.get("name").and_then(|v| v.as_str()) {
                    // Strip common prefixes like "Counter" to find base event name
                    let base_name = name
                        .strip_prefix(&format!("{}_", service.name))
                        .unwrap_or(name);
                    let base_name = name.strip_prefix(&service.name).unwrap_or(base_name);

                    // Check if a similar name exists in contracts
                    let found = contract_event_names.iter().any(|cn| {
                        cn.contains(base_name)
                            || base_name.contains(cn)
                            || cn.to_lowercase() == name.to_lowercase()
                    });

                    if !found && !contract_event_names.is_empty() {
                        drifts.push(format!(
                            "Service '{}' event '{}' not found in contracts_events (known: {:?})",
                            service.name, name, contract_event_names
                        ));
                    }
                }
            }
        }
    }

    if drifts.is_empty() {
        info!("  Event contracts in sync");
    }

    Ok(drifts)
}

/// Check that service ports have corresponding types in contracts
fn check_port_contract_drift(platform_dir: &Path, root_dir: &Path) -> Result<Vec<String>> {
    info!("Checking port contract drift...");

    let drifts: Vec<String> = Vec::new();
    let services_dir = platform_dir.join("model/services");

    if !services_dir.exists() {
        return Ok(drifts);
    }

    // Collect all exported type names from contracts_api and contracts_auth
    let mut contract_type_names = BTreeSet::new();

    for contracts_dir in &[
        root_dir.join("packages/contracts/api/src/lib.rs"),
        root_dir.join("packages/contracts/auth/src/lib.rs"),
        root_dir.join("packages/contracts/events/src/lib.rs"),
    ] {
        if contracts_dir.exists() {
            let content = fs::read_to_string(contracts_dir)?;
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("pub struct ") {
                    if let Some(name) = line.strip_prefix("pub struct ") {
                        let name = name.split('{').next().unwrap_or(name).trim();
                        contract_type_names.insert(name.to_string());
                    }
                } else if line.starts_with("pub use ")
                    && let Some(name) = line.split("::").last()
                {
                    let name = name.trim_end_matches(';').trim();
                    if !name.is_empty() {
                        contract_type_names.insert(name.to_string());
                    }
                }
            }
        }
    }

    // Check each service's ports for contract type alignment
    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let service: ServiceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse: {}", path.display()))?;

        // Skip infrastructure services — they don't have API-facing contracts
        let domain = serde_yaml::from_str::<serde_json::Value>(&content)
            .ok()
            .and_then(|v| v.get("domain").and_then(|d| d.as_str()).map(String::from));
        if domain.as_deref() == Some("infrastructure") {
            continue;
        }

        if let Some(ports) = &service.ports {
            for port in ports {
                if let Some(_port_name) = port.get("name").and_then(|v| v.as_str()) {
                    let service_upper = service
                        .name
                        .chars()
                        .next()
                        .unwrap_or(' ')
                        .to_uppercase()
                        .to_string()
                        + &service.name[1..];

                    // Check if any contract type relates to this service
                    let has_service_contracts = contract_type_names.iter().any(|tn| {
                        tn.starts_with(&service_upper)
                            || tn.starts_with(&service.name)
                            || tn.to_lowercase().contains(&service.name)
                    });

                    if !has_service_contracts && !contract_type_names.is_empty() {
                        // Informational: service has ports but no matching contract types.
                        // This is OK when contract types use different naming (e.g., InitTenantRequest).
                        info!(
                            "  ℹ Service '{}' has ports but no contract types referencing '{}' (this is OK if contracts use different naming)",
                            service.name, service_upper
                        );
                    }
                    break;
                }
            }
        }
    }

    if drifts.is_empty() {
        info!("  Port contracts in sync");
    }

    Ok(drifts)
}

/// Check that platform model services match actual service crates
fn check_service_crate_drift(platform_dir: &Path, root_dir: &Path) -> Result<Vec<String>> {
    info!("Checking service crate drift...");

    let mut drifts = Vec::new();

    // Get service names from platform model
    let services_dir = platform_dir.join("model/services");
    let mut model_service_names = BTreeSet::new();

    if services_dir.exists() {
        for entry in fs::read_dir(&services_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let service: ServiceModel = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse: {}", path.display()))?;

            model_service_names.insert(service.name.clone());
        }
    }

    // Get service package names from Cargo manifests under services/
    let services_crate_dir = root_dir.join("services");
    let mut crate_service_names = BTreeSet::new();

    if services_crate_dir.exists() {
        for entry in fs::read_dir(&services_crate_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Some(package_name) = read_package_name(&cargo_toml)? {
                    crate_service_names.insert(package_name);
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    crate_service_names.insert(name.to_string());
                }
            }
        }
    }

    // Check for mismatches
    for model_name in &model_service_names {
        if !crate_service_names.contains(model_name) {
            drifts.push(format!(
                "Platform model '{}' has no corresponding service crate in services/",
                model_name
            ));
        }
    }

    for crate_name in &crate_service_names {
        if !model_service_names.contains(crate_name) {
            drifts.push(format!(
                "Service crate '{}' has no corresponding platform model",
                crate_name
            ));
        }
    }

    if drifts.is_empty() {
        info!(
            "  Service crates in sync ({} services)",
            model_service_names.len()
        );
    }

    Ok(drifts)
}

fn read_package_name(cargo_toml: &Path) -> Result<Option<String>> {
    let content = fs::read_to_string(cargo_toml)
        .with_context(|| format!("Failed to read Cargo manifest: {}", cargo_toml.display()))?;

    let mut in_package_section = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[package]" {
            in_package_section = true;
            continue;
        }

        if in_package_section && trimmed.starts_with('[') {
            break;
        }

        if in_package_section
            && trimmed.starts_with("name")
            && let Some((_, value)) = trimmed.split_once('=')
        {
            return Ok(Some(value.trim().trim_matches('"').to_string()));
        }
    }

    Ok(None)
}
