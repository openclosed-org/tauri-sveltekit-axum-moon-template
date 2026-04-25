use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "topology-validator")]
#[command(about = "Validate deployment topology completeness and consistency")]
struct Args {
    /// Path to platform directory
    #[arg(short, long, default_value = "platform")]
    platform_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct TopologyModel {
    name: String,
    description: Option<String>,
    deployables: Option<Vec<TopologyDeployableRef>>,
    environment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TopologyDeployableRef {
    name: String,
    replicas: Option<u32>,
    resources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DeployableModel {
    name: String,
    description: Option<String>,
    services: Option<Vec<String>>,
    resources: Option<Vec<String>>,
    health_check: Option<HealthCheck>,
    current_status: Option<String>,
    independent_deploy: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct HealthCheck {
    endpoint: Option<String>,
    interval: Option<String>,
    timeout: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResourceModel {
    name: String,
    r#type: String,
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

    info!("Validating topologies in {}", platform_dir.display());

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Load all deployables
    let deployables = load_deployables(&platform_dir)?;
    info!("Loaded {} deployables", deployables.len());

    // 2. Load all resources
    let resources = load_resources(&platform_dir)?;
    info!("Loaded {} resources", resources.len());

    // 3. Load and validate each topology
    let topologies = load_topologies(&platform_dir)?;
    info!("Loaded {} topologies", topologies.len());

    for (topology_name, topology) in &topologies {
        info!("Validating topology: {}", topology_name);

        // Check deployable references
        if let Some(deployable_refs) = &topology.deployables {
            for deployable_ref in deployable_refs {
                if !deployables.contains_key(&deployable_ref.name) {
                    errors.push(format!(
                        "Topology '{}' references unknown deployable: {}",
                        topology_name, deployable_ref.name
                    ));
                } else {
                    // Check resource references in deployable override
                    if let Some(resource_overrides) = &deployable_ref.resources {
                        let deployable = &deployables[&deployable_ref.name];
                        let base_resources: BTreeSet<_> =
                            deployable.resources.iter().flatten().cloned().collect();

                        for resource in resource_overrides {
                            if !base_resources.contains(resource)
                                && !resources.contains_key(resource)
                            {
                                warnings.push(format!(
                                    "Topology '{}' deployable '{}' references unknown resource: {}",
                                    topology_name, deployable_ref.name, resource
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Check that topology has at least one deployable
        if topology.deployables.as_ref().is_none_or(|d| d.is_empty()) {
            warnings.push(format!(
                "Topology '{}' has no deployables defined",
                topology_name
            ));
        }

        // Check completeness: all critical deployables should be present
        let critical_deploy: BTreeSet<_> = deployables
            .iter()
            .filter(|(_, d)| is_critical_deployable(d))
            .map(|(name, _)| name.clone())
            .collect();

        let topology_deployables: BTreeSet<_> = topology
            .deployables
            .iter()
            .flatten()
            .map(|d| d.name.clone())
            .collect();

        let missing_critical: BTreeSet<_> = critical_deploy
            .difference(&topology_deployables)
            .cloned()
            .collect();

        if !missing_critical.is_empty() {
            warnings.push(format!(
                "Topology '{}' missing critical deployables: {:?}",
                topology_name, missing_critical
            ));
        }
    }

    // 4. Check that all topologies have corresponding verification tests
    let verification_dir = platform_dir
        .parent()
        .unwrap_or(&platform_dir)
        .join("verification");
    for topology_name in topologies.keys() {
        let test_dir = verification_dir.join("topology").join(topology_name);
        if !test_dir.exists() {
            warnings.push(format!(
                "No verification tests found for topology '{}'",
                topology_name
            ));
        }
    }

    // Report
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║           Topology Validation Report                     ║");
    println!("╠══════════════════════════════════════════════════════════╣");

    if errors.is_empty() && warnings.is_empty() {
        println!("║   All topologies valid                                   ");
        println!("║   All references resolvable                              ");
    } else {
        for err in &errors {
            println!("!! {}", err);
        }
        for warn_msg in &warnings {
            println!("!  WARNING: {}", warn_msg);
        }
    }

    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ Topologies: {:<44} ║", topologies.len());
    println!("║ Errors:     {:<44} ║", errors.len());
    println!("║ Warnings:   {:<44} ║", warnings.len());
    println!("╚══════════════════════════════════════════════════════════╝\n");

    if errors.is_empty() {
        info!("Topology validation passed!");
        Ok(())
    } else {
        anyhow::bail!("Topology validation failed: {} errors", errors.len());
    }
}

fn load_deployables(platform_dir: &Path) -> Result<BTreeMap<String, DeployableModel>> {
    let deployables_dir = platform_dir.join("model/deployables");
    let mut deployables = BTreeMap::new();

    if !deployables_dir.exists() {
        return Ok(deployables);
    }

    for entry in fs::read_dir(&deployables_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        let model: DeployableModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse deployable: {}", path.display()))?;

        deployables.insert(model.name.clone(), model);
    }

    Ok(deployables)
}

fn load_resources(platform_dir: &Path) -> Result<BTreeMap<String, ResourceModel>> {
    let resources_dir = platform_dir.join("model/resources");
    let mut resources = BTreeMap::new();

    if !resources_dir.exists() {
        return Ok(resources);
    }

    for entry in fs::read_dir(&resources_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        let model: ResourceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse resource: {}", path.display()))?;

        resources.insert(model.name.clone(), model);
    }

    Ok(resources)
}

fn load_topologies(platform_dir: &Path) -> Result<BTreeMap<String, TopologyModel>> {
    let topologies_dir = platform_dir.join("model/topologies");
    let mut topologies = BTreeMap::new();

    if !topologies_dir.exists() {
        return Ok(topologies);
    }

    for entry in fs::read_dir(&topologies_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        let model: TopologyModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse topology: {}", path.display()))?;

        let name = model.name.clone();
        topologies.insert(name, model);
    }

    Ok(topologies)
}

fn is_critical_deployable(deployable: &DeployableModel) -> bool {
    matches!(deployable.current_status.as_deref(), Some("implemented"))
        && deployable.independent_deploy.unwrap_or(false)
}
