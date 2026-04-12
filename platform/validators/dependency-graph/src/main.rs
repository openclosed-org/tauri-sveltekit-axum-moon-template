use anyhow::{Context, Result};
use clap::Parser;
use petgraph::stable_graph::{StableGraph, NodeIndex};
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "dependency-validator")]
#[command(about = "Validate platform model dependency graph for cycles and broken refs")]
struct Args {
    /// Path to platform directory
    #[arg(short, long, default_value = "platform")]
    platform_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ServiceModel {
    name: String,
    dependencies: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DeployableModel {
    name: String,
    services: Option<Vec<String>>,
    resources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TopologyDeployable {
    name: String,
    enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TopologyFile {
    deployables: Option<Vec<TopologyDeployable>>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let platform_dir = fs::canonicalize(&args.platform_dir)
        .with_context(|| format!("Platform directory not found: {}", args.platform_dir.display()))?;

    info!("Validating dependency graph in {}", platform_dir.display());

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Build dependency graph from service models
    let (graph, node_map) = build_service_dependency_graph(&platform_dir)?;

    // 2. Check for cycles using Tarjan's SCC
    let cycles = detect_cycles(&graph, &node_map);
    if !cycles.is_empty() {
        errors.push(format!("Circular dependencies detected: {:?}", cycles));
    }

    // 3. Check for broken references (service → service deps)
    let broken_refs = check_broken_references(&platform_dir)?;
    if !broken_refs.is_empty() {
        errors.extend(broken_refs);
    }

    // 4. Check topology → deployable references
    let topo_broken = check_topology_refs(&platform_dir)?;
    if !topo_broken.is_empty() {
        warnings.extend(topo_broken);
    }

    // 5. Check deployable → service references
    let deploy_broken = check_deployable_service_refs(&platform_dir)?;
    if !deploy_broken.is_empty() {
        errors.extend(deploy_broken);
    }

    // Report
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          Dependency Graph Validation Report             ║");
    println!("╠══════════════════════════════════════════════════════════╣");

    if errors.is_empty() && warnings.is_empty() {
        println!("║   No circular dependencies                              ");
        println!("║   All references resolvable                               ");
        println!("║   No broken deployable->service refs                      ");
    } else {
        if !cycles.is_empty() {
            println!("!! Circular dependencies: {:?}", cycles);
        }
        for err in &errors {
            println!("!! {}", err);
        }
        for warn_msg in &warnings {
            println!("!  WARNING: {}", warn_msg);
        }
    }

    println!("!----------------------------------------------------------");
    println!("  Errors:   {}", errors.len());
    println!("  Warnings: {}", warnings.len());
    println!("===========================================================\n");

    if errors.is_empty() {
        info!("Dependency graph valid!");
        Ok(())
    } else {
        anyhow::bail!("Dependency graph validation failed: {} errors", errors.len());
    }
}

fn build_service_dependency_graph(platform_dir: &Path) -> Result<(StableGraph<(), ()>, BTreeMap<String, NodeIndex>)> {
    info!("Building service dependency graph...");

    let services_dir = platform_dir.join("model/services");
    let mut graph = StableGraph::<(), ()>::new();
    let mut node_map: BTreeMap<String, NodeIndex> = BTreeMap::new();

    if !services_dir.exists() {
        return Ok((graph, node_map));
    }

    // First pass: register all service names as nodes
    let mut services = Vec::new();
    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let service: ServiceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse service model: {}", path.display()))?;

        let idx = graph.add_node(());
        node_map.insert(service.name.clone(), idx);
        services.push(service);
    }

    // Second pass: add edges for dependencies
    for service in &services {
        if let Some(deps) = &service.dependencies {
            for dep in deps {
                // Dependencies in platform models reference other service names
                // or packages. Only add edges for service-to-service refs.
                if let Some(&target_idx) = node_map.get(dep.as_str()) {
                    let source_idx = node_map[&service.name];
                    graph.add_edge(source_idx, target_idx, ());
                }
            }
        }
    }

    info!("  Graph: {} nodes, {} edges", graph.node_count(), graph.edge_count());
    Ok((graph, node_map))
}

fn detect_cycles(graph: &StableGraph<(), ()>, node_map: &BTreeMap<String, NodeIndex>) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();

    // Use Tarjan's SCC algorithm - any SCC with >1 node indicates a cycle
    let sccs = petgraph::algo::tarjan_scc(graph);

    for scc in &sccs {
        if scc.len() > 1 {
            // Map NodeIndex back to service names
            let reverse_map: BTreeMap<NodeIndex, String> = node_map.iter()
                .map(|(name, idx)| (*idx, name.clone()))
                .collect();

            let cycle_names: Vec<String> = scc.iter()
                .filter_map(|idx| reverse_map.get(idx).cloned())
                .collect();

            cycles.push(cycle_names);
        }
    }

    // Also check for self-loops
    for (name, &idx) in node_map {
        for edge in graph.edges_directed(idx, Direction::Outgoing) {
            let target = edge.target();
            if target == idx {
                cycles.push(vec![name.clone()]);
            }
        }
    }

    cycles
}

fn check_broken_references(platform_dir: &Path) -> Result<Vec<String>> {
    info!("Checking for broken service references...");

    let mut broken = Vec::new();
    let services_dir = platform_dir.join("model/services");

    if !services_dir.exists() {
        return Ok(broken);
    }

    // Collect all service names
    let mut service_names = BTreeSet::new();
    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let service: ServiceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse: {}", path.display()))?;

        service_names.insert(service.name);
    }

    // Check dependencies
    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let service: ServiceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse: {}", path.display()))?;

        if let Some(deps) = &service.dependencies {
            for dep in deps {
                // Skip package paths (packages/*, services/*) — those are not service refs
                if dep.contains('/') || dep.contains('-') && dep.len() > 15 {
                    continue;
                }
                // Only warn if it looks like a short service name that's not registered
                if !service_names.contains(dep.as_str()) && dep.len() < 30 {
                    broken.push(format!(
                        "Service '{}' depends on unknown service '{}'",
                        service.name, dep
                    ));
                }
            }
        }
    }

    if broken.is_empty() {
        info!("  All service references valid");
    }

    Ok(broken)
}

fn check_topology_refs(platform_dir: &Path) -> Result<Vec<String>> {
    info!("Checking topology -> deployable references...");

    let mut warnings = Vec::new();

    // Load all deployable names
    let deployables_dir = platform_dir.join("model/deployables");
    let mut deployable_names = BTreeSet::new();

    if deployables_dir.exists() {
        for entry in fs::read_dir(&deployables_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let deployable: DeployableModel = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse: {}", path.display()))?;

            deployable_names.insert(deployable.name);
        }
    }

    // Check each topology
    let topologies_dir = platform_dir.join("model/topologies");
    if !topologies_dir.exists() {
        return Ok(warnings);
    }

    for entry in fs::read_dir(&topologies_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let topology: TopologyFile = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse topology: {}", path.display()))?;

        if let Some(deployables) = topology.deployables {
            for dep in &deployables {
                if !deployable_names.contains(&dep.name) {
                    warnings.push(format!(
                        "Topology references unknown deployable '{}'",
                        dep.name
                    ));
                }
            }
        }
    }

    if warnings.is_empty() {
        info!("  All topology references valid");
    }

    Ok(warnings)
}

fn check_deployable_service_refs(platform_dir: &Path) -> Result<Vec<String>> {
    info!("Checking deployable -> service references...");

    let mut broken = Vec::new();

    // Load all service names
    let services_dir = platform_dir.join("model/services");
    let mut service_names = BTreeSet::new();

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

            service_names.insert(service.name);
        }
    }

    // Check each deployable
    let deployables_dir = platform_dir.join("model/deployables");
    if !deployables_dir.exists() {
        return Ok(broken);
    }

    for entry in fs::read_dir(&deployables_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let deployable: DeployableModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse: {}", path.display()))?;

        if let Some(services) = &deployable.services {
            for svc in services {
                if !service_names.contains(svc.as_str()) {
                    broken.push(format!(
                        "Deployable '{}' references unknown service '{}'",
                        deployable.name, svc
                    ));
                }
            }
        }
    }

    if broken.is_empty() {
        info!("  All deployable->service references valid");
    }

    Ok(broken)
}
