use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "security-validator")]
#[command(about = "Validate platform models for security best practices")]
struct Args {
    /// Path to platform directory
    #[arg(short, long, default_value = "platform")]
    platform_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ServiceModel {
    name: String,
    authentication: Option<AuthConfig>,
    authorization: Option<AuthzConfig>,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    required: Option<bool>,
    methods: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AuthzConfig {
    required: Option<bool>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeployableModel {
    name: String,
    environment_variables: Option<Vec<EnvVar>>,
    health_check: Option<HealthCheck>,
}

#[derive(Debug, Deserialize)]
struct EnvVar {
    name: String,
    sensitive: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct HealthCheck {
    endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResourceModel {
    name: String,
    r#type: String,
    encryption: Option<EncryptionConfig>,
    access_control: Option<AccessControl>,
}

#[derive(Debug, Deserialize)]
struct EncryptionConfig {
    at_rest: Option<bool>,
    in_transit: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AccessControl {
    rbac: Option<bool>,
    abac: Option<bool>,
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

    info!("Running security validation in {}", platform_dir.display());

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Check services for authentication/authorization configuration
    check_service_security(&platform_dir, &mut errors, &mut warnings)?;

    // 2. Check resources for encryption and access control
    check_resource_security(&platform_dir, &mut errors, &mut warnings)?;

    // 3. Check deployables for sensitive environment variables
    check_deployable_security(&platform_dir, &mut errors, &mut warnings)?;

    // 4. Check for common security anti-patterns
    check_security_antipatterns(&platform_dir, &mut errors, &mut warnings)?;

    // Report
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║           Security Validation Report                      ║");
    println!("╠══════════════════════════════════════════════════════════╣");

    if errors.is_empty() && warnings.is_empty() {
        println!("║   All security checks passed                             ");
    } else {
        for err in &errors {
            println!("!! SECURITY ERROR: {}", err);
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
        info!("Security validation passed!");
        Ok(())
    } else {
        anyhow::bail!("Security validation failed: {} errors", errors.len());
    }
}

fn check_service_security(
    platform_dir: &Path,
    _errors: &mut [String],
    warnings: &mut Vec<String>,
) -> Result<()> {
    info!("Checking service security configurations...");

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

        // Check if authentication is configured
        if service
            .authentication
            .as_ref()
            .is_none_or(|a| a.required.unwrap_or(false))
        {
            // Auth required but check if methods are specified
            if service
                .authentication
                .as_ref()
                .and_then(|a| a.methods.as_ref())
                .is_none_or(|m| m.is_empty())
            {
                warnings.push(format!(
                    "Service '{}' requires authentication but no auth methods specified",
                    service.name
                ));
            }
        }

        // Check if authorization is configured for services that likely need it
        if service.authorization.is_none() {
            // Services with "admin", "tenant", or "user" in name likely need authz
            let name_lower = service.name.to_lowercase();
            if name_lower.contains("admin")
                || name_lower.contains("tenant")
                || name_lower.contains("user")
            {
                warnings.push(format!(
                    "Service '{}' likely needs authorization configuration",
                    service.name
                ));
            }
        }
    }

    Ok(())
}

fn check_resource_security(
    platform_dir: &Path,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    info!("Checking resource security configurations...");

    let resources_dir = platform_dir.join("model/resources");
    if !resources_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&resources_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let resource: ResourceModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse resource: {}", path.display()))?;

        // Check encryption for sensitive resource types
        let is_sensitive = matches!(
            resource.r#type.to_lowercase().as_str(),
            "database" | "object-storage" | "secrets" | "cache"
        );

        if is_sensitive {
            if let Some(enc) = resource.encryption.as_ref() {
                if enc.at_rest == Some(false) {
                    errors.push(format!(
                        "Sensitive resource '{}' explicitly disables encryption at rest",
                        resource.name
                    ));
                }
                if enc.in_transit == Some(false) {
                    errors.push(format!(
                        "Sensitive resource '{}' explicitly disables encryption in transit",
                        resource.name
                    ));
                }
            } else {
                warnings.push(format!(
                    "Sensitive resource '{}' has no encryption configuration",
                    resource.name
                ));
            }

            // Check access control
            if resource.access_control.is_none() {
                warnings.push(format!(
                    "Sensitive resource '{}' has no access control configuration",
                    resource.name
                ));
            }
        }
    }

    Ok(())
}

fn check_deployable_security(
    platform_dir: &Path,
    _errors: &mut [String],
    warnings: &mut Vec<String>,
) -> Result<()> {
    info!("Checking deployable security configurations...");

    let deployables_dir = platform_dir.join("model/deployables");
    if !deployables_dir.exists() {
        return Ok(());
    }

    // Patterns that suggest sensitive data
    let sensitive_patterns = [
        "password",
        "secret",
        "key",
        "token",
        "credential",
        "api_key",
        "private",
    ];

    for entry in fs::read_dir(&deployables_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let deployable: DeployableModel = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse deployable: {}", path.display()))?;

        if let Some(env_vars) = &deployable.environment_variables {
            for env_var in env_vars {
                let name_lower = env_var.name.to_lowercase();
                let is_likely_sensitive = sensitive_patterns
                    .iter()
                    .any(|pattern| name_lower.contains(*pattern));

                if is_likely_sensitive && env_var.sensitive != Some(true) {
                    warnings.push(format!(
                        "Deployable '{}' has environment variable '{}' that appears sensitive but is not marked",
                        deployable.name, env_var.name
                    ));
                }
            }
        }
    }

    Ok(())
}

fn check_security_antipatterns(
    platform_dir: &Path,
    _errors: &mut [String],
    warnings: &mut Vec<String>,
) -> Result<()> {
    info!("Checking for security antipatterns...");

    // Walk all YAML files and check for hardcoded secrets
    let yaml_files = walkdir::WalkDir::new(platform_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|ext| ext.to_str()) == Some("yaml")
                && !e.path().to_string_lossy().contains("/target/")
        });

    // Patterns that might indicate hardcoded secrets
    let secret_patterns =
        [
            regex::Regex::new(r#"(?i)(password|secret|key|token)\s*:\s*['"]?[a-zA-Z0-9]{16,}"#)
                .unwrap(),
        ];

    for yaml_file in yaml_files {
        let content = fs::read_to_string(yaml_file.path())?;

        for pattern in &secret_patterns {
            if let Some(mat) = pattern.find(&content) {
                warnings.push(format!(
                    "Possible hardcoded secret in {}: {}",
                    yaml_file.path().display(),
                    mat.as_str()
                ));
            }
        }
    }

    Ok(())
}
