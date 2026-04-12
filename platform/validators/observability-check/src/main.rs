use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "observability-validator")]
#[command(
    about = "Validate observability/telemetry configuration completeness"
)]
struct Args {
    /// Path to platform directory
    #[arg(short, long, default_value = "platform")]
    platform_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ServiceModel {
    name: String,
    observability: Option<ObservabilityConfig>,
}

#[derive(Debug, Deserialize)]
struct ObservabilityConfig {
    tracing: Option<TracingConfig>,
    metrics: Option<MetricsConfig>,
    logging: Option<LoggingConfig>,
}

#[derive(Debug, Deserialize)]
struct TracingConfig {
    enabled: Option<bool>,
    sampler: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MetricsConfig {
    enabled: Option<bool>,
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: Option<String>,
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeployableModel {
    name: String,
    observability: Option<DeployableObservability>,
}

#[derive(Debug, Deserialize)]
struct DeployableObservability {
    health_check: Option<HealthCheck>,
    readiness_check: Option<HealthCheck>,
}

#[derive(Debug, Deserialize)]
struct HealthCheck {
    endpoint: Option<String>,
    interval: Option<String>,
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

    info!(
        "Validating observability configuration in {}",
        platform_dir.display()
    );

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Check services for observability configuration
    check_service_observability(&platform_dir, &mut errors, &mut warnings)?;

    // 2. Check deployables for health checks
    check_deployable_observability(&platform_dir, &mut errors, &mut warnings)?;

    // 3. Check that observability resources are defined
    check_observability_resources(&platform_dir, &mut errors, &mut warnings)?;

    // Report
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║        Observability Validation Report                    ║");
    println!("╠══════════════════════════════════════════════════════════╣");

    if errors.is_empty() && warnings.is_empty() {
        println!("║   All observability checks passed                          ");
    } else {
        for err in &errors {
            println!("!! {}", err);
        }
        for warn_msg in &warnings {
            println!("!  WARNING: {}", warn_msg);
        }
    }

    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ Errors:     {:<44} ║", errors.len());
    println!("║ Warnings:   {:<44} ║", warnings.len());
    println!("╚══════════════════════════════════════════════════════════╝\n");

    if errors.is_empty() {
        info!("Observability validation passed!");
        Ok(())
    } else {
        anyhow::bail!(
            "Observability validation failed: {} errors",
            errors.len()
        );
    }
}

fn check_service_observability(
    platform_dir: &Path,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    info!("Checking service observability configurations...");

    let services_dir = platform_dir.join("model/services");
    if !services_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let service: ServiceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse service: {}", path.display()))?;

        let obs = match &service.observability {
            Some(o) => o,
            None => {
                warnings.push(format!(
                    "Service '{}' has no observability configuration",
                    service.name
                ));
                continue;
            }
        };

        // Check tracing
        if obs.tracing.is_none() {
            warnings.push(format!(
                "Service '{}' has no tracing configuration",
                service.name
            ));
        } else if obs.tracing.as_ref().unwrap().enabled == Some(false) {
            warnings.push(format!(
                "Service '{}' has tracing explicitly disabled",
                service.name
            ));
        }

        // Check metrics
        if obs.metrics.is_none() {
            warnings.push(format!(
                "Service '{}' has no metrics configuration",
                service.name
            ));
        } else if obs.metrics.as_ref().unwrap().enabled == Some(false) {
            warnings.push(format!(
                "Service '{}' has metrics explicitly disabled",
                service.name
            ));
        }

        // Check logging
        if obs.logging.is_none() {
            warnings.push(format!(
                "Service '{}' has no logging configuration",
                service.name
            ));
        }
    }

    Ok(())
}

fn check_deployable_observability(
    platform_dir: &Path,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    info!("Checking deployable observability configurations...");

    let deployables_dir = platform_dir.join("model/deployables");
    if !deployables_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&deployables_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let deployable: DeployableModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse deployable: {}", path.display()))?;

        let obs = match &deployable.observability {
            Some(o) => o,
            None => {
                warnings.push(format!(
                    "Deployable '{}' has no observability configuration",
                    deployable.name
                ));
                continue;
            }
        };

        // Check health check
        if obs.health_check.is_none() {
            errors.push(format!(
                "Deployable '{}' has no health check endpoint",
                deployable.name
            ));
        } else if obs
            .health_check
            .as_ref()
            .unwrap()
            .endpoint
            .is_none()
        {
            errors.push(format!(
                "Deployable '{}' health check has no endpoint",
                deployable.name
            ));
        }

        // Check readiness check (warning, not error)
        if obs.readiness_check.is_none() {
            warnings.push(format!(
                "Deployable '{}' has no readiness check endpoint",
                deployable.name
            ));
        }
    }

    Ok(())
}

fn check_observability_resources(
    platform_dir: &Path,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    info!("Checking observability resources...");

    let resources_dir = platform_dir.join("model/resources");
    if !resources_dir.exists() {
        return Ok(());
    }

    let mut has_tracing_resource = false;
    let mut has_metrics_resource = false;
    let mut has_logging_resource = false;

    for entry in fs::read_dir(&resources_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;

        // Simple check for resource type in content
        if content.contains("type: tracing") || content.contains("type: jaeger") {
            has_tracing_resource = true;
        }
        if content.contains("type: metrics") || content.contains("type: prometheus") {
            has_metrics_resource = true;
        }
        if content.contains("type: logging") || content.contains("type: loki") {
            has_logging_resource = true;
        }
    }

    if !has_tracing_resource {
        warnings.push("No tracing resource defined in platform model".to_string());
    }
    if !has_metrics_resource {
        warnings.push("No metrics resource defined in platform model".to_string());
    }
    if !has_logging_resource {
        warnings.push("No logging resource defined in platform model".to_string());
    }

    Ok(())
}
