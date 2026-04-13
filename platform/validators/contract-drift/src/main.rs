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

#[derive(Debug, Deserialize)]
struct ContractEndpoint {
    path: String,
    method: String,
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

    let mut drifts = Vec::new();

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

    let mut drifts = Vec::new();
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

    let mut drifts = Vec::new();
    let services_dir = platform_dir.join("model/services");

    if !services_dir.exists() {
        return Ok(drifts);
    }

    // Collect all exported type names from contracts_api
    let api_lib = root_dir.join("packages/contracts/api/src/lib.rs");
    let mut contract_type_names = BTreeSet::new();

    if api_lib.exists() {
        let content = fs::read_to_string(&api_lib)?;
        // Look for struct definitions and pub use statements
        for line in content.lines() {
            let line = line.trim();
            // Match `pub struct Foo {` or `pub use ...::Foo;`
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

    // Check each service's ports
    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let service: ServiceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse: {}", path.display()))?;

        if let Some(ports) = &service.ports {
            for port in ports {
                if let Some(_port_name) = port.get("name").and_then(|v| v.as_str()) {
                    // Port names like "counter_repository" should map to contract types
                    // like "CounterResponse" — just verify contracts exist for this service
                    let service_upper = service
                        .name
                        .chars()
                        .next()
                        .unwrap_or(' ')
                        .to_uppercase()
                        .to_string()
                        + &service.name[1..];

                    let has_service_contracts = contract_type_names
                        .iter()
                        .any(|tn| tn.starts_with(&service_upper) || tn.starts_with(&service.name));

                    if !has_service_contracts && !contract_type_names.is_empty() {
                        drifts.push(format!(
                            "Service '{}' has ports but no contract types starting with '{}'",
                            service.name, service_upper
                        ));
                    }
                    break; // Only check once per service
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

    // Get service crate names from services/ directory
    let services_crate_dir = root_dir.join("services");
    let mut crate_service_names = BTreeSet::new();

    if services_crate_dir.exists() {
        for entry in fs::read_dir(&services_crate_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Check if it has a Cargo.toml (is a Rust crate)
            if path.join("Cargo.toml").exists()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                // Strip -service suffix for comparison
                let clean_name = name.strip_suffix("-service").unwrap_or(name);
                crate_service_names.insert(clean_name.to_string());
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
