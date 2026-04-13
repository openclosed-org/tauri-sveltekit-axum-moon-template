use anyhow::{Context, Result};
use clap::Parser;
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "platform-validator")]
#[command(about = "Validate platform models against JSON schemas")]
struct Args {
    /// Path to platform directory
    #[arg(short, long, default_value = "platform")]
    platform_dir: PathBuf,

    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationResult {
    file: String,
    schema: String,
    valid: bool,
    errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Summary {
    total: usize,
    passed: usize,
    failed: usize,
    warnings: usize,
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

    info!("Validating platform models in {}", platform_dir.display());

    let schema_dir = platform_dir.join("schema");
    let model_dir = platform_dir.join("model");

    if !schema_dir.exists() {
        anyhow::bail!("Schema directory not found: {}", schema_dir.display());
    }
    if !model_dir.exists() {
        anyhow::bail!("Model directory not found: {}", model_dir.display());
    }

    let mut results = Vec::new();
    let mut warnings = 0;

    // Define model type to schema file mapping
    let model_schema_map = [
        ("services", "service.schema.json"),
        ("deployables", "deployable.schema.json"),
        ("resources", "resource.schema.json"),
        ("workflows", "workflow.schema.json"),
        ("topologies", "topology.schema.json"),
        ("policies", "policy.schema.json"),
    ];

    for (model_type, schema_file) in &model_schema_map {
        let schema_path = schema_dir.join(schema_file);
        if !schema_path.exists() {
            warn!(
                "Schema not found: {} — skipping {}",
                schema_path.display(),
                model_type
            );
            warnings += 1;
            continue;
        }

        let schema_content = fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema: {}", schema_path.display()))?;

        let schema: serde_json::Value = serde_json::from_str(&schema_content)
            .with_context(|| format!("Invalid JSON in schema: {}", schema_path.display()))?;

        let compiled_schema = Validator::new(&schema)
            .with_context(|| format!("Invalid schema structure: {}", schema_file))?;

        let model_type_dir = model_dir.join(model_type);
        if !model_type_dir.exists() {
            warn!(
                "Model directory not found: {} — skipping",
                model_type_dir.display()
            );
            warnings += 1;
            continue;
        }

        // Find all YAML files in this model type directory
        let yaml_pattern = format!("{}/*.yaml", model_type_dir.display());
        let yaml_files: Vec<PathBuf> = glob::glob(&yaml_pattern)
            .with_context(|| format!("Invalid glob pattern: {}", yaml_pattern))?
            .filter_map(|p| p.ok())
            .collect();

        for yaml_file in &yaml_files {
            let content = fs::read_to_string(yaml_file)
                .with_context(|| format!("Failed to read: {}", yaml_file.display()))?;

            let yaml_value: serde_json::Value = serde_yaml::from_str(&content)
                .with_context(|| format!("Invalid YAML in: {}", yaml_file.display()))?;

            let mut errors = Vec::new();

            // Validate using the Validator API - is_valid returns bool
            if !compiled_schema.is_valid(&yaml_value) {
                // Collect errors manually if needed
                errors.push("Validation failed".to_string());
            }

            let valid = errors.is_empty();
            let file_name = yaml_file
                .strip_prefix(&platform_dir)
                .unwrap_or(yaml_file)
                .to_string_lossy()
                .to_string();

            results.push(ValidationResult {
                file: file_name,
                schema: schema_file.to_string(),
                valid,
                errors,
            });
        }
    }

    // Additional checks: cross-reference validation
    info!("Running cross-reference checks...");
    warnings += check_service_deployable_refs(&platform_dir, &mut results);
    warnings += check_resource_refs(&platform_dir, &mut results);

    // Print results
    let passed = results.iter().filter(|r| r.valid).count();
    let failed = results.iter().filter(|r| !r.valid).count();

    match args.format {
        OutputFormat::Text => print_text_report(&results, passed, failed, warnings),
        OutputFormat::Json => print_json_report(&results, passed, failed, warnings),
    }

    if failed > 0 {
        anyhow::bail!("Validation failed: {} errors", failed);
    }

    info!("✅ All models valid!");
    Ok(())
}

fn check_service_deployable_refs(platform_dir: &Path, _results: &mut [ValidationResult]) -> usize {
    let mut warnings = 0;

    // Load all service models
    let services_dir = platform_dir.join("model/services");
    if !services_dir.exists() {
        return 0;
    }

    // Load all deployable models to check references
    let deployables_dir = platform_dir.join("model/deployables");
    if !deployables_dir.exists() {
        return 0;
    }

    let mut deployable_names = Vec::new();
    if let Ok(entries) = fs::read_dir(&deployables_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(content) = fs::read_to_string(entry.path())
                && let Ok(yaml) = serde_yaml::from_str::<serde_json::Value>(&content)
                && let Some(name) = yaml.get("name").and_then(|v| v.as_str())
            {
                deployable_names.push(name.to_string());
            }
        }
    }

    // Check service -> deployable references
    if let Ok(entries) = fs::read_dir(&services_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(content) = fs::read_to_string(entry.path())
                && let Ok(yaml) = serde_yaml::from_str::<serde_json::Value>(&content)
                && let Some(deployable) = yaml.get("deployable").and_then(|v| v.as_str())
                && !deployable_names.contains(&deployable.to_string())
            {
                warnings += 1;
                warn!(
                    "Service {} references non-existent deployable: {}",
                    entry.path().display(),
                    deployable
                );
            }
        }
    }

    warnings
}

fn check_resource_refs(platform_dir: &Path, _results: &mut [ValidationResult]) -> usize {
    let mut warnings = 0;

    // Load all resource names
    let resources_dir = platform_dir.join("model/resources");
    if !resources_dir.exists() {
        return 0;
    }

    let mut resource_names = Vec::new();
    if let Ok(entries) = fs::read_dir(&resources_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(content) = fs::read_to_string(entry.path())
                && let Ok(yaml) = serde_yaml::from_str::<serde_json::Value>(&content)
                && let Some(name) = yaml.get("name").and_then(|v| v.as_str())
            {
                resource_names.push(name.to_string());
            }
        }
    }

    // Check deployable -> resource references
    let deployables_dir = platform_dir.join("model/deployables");
    if !deployables_dir.exists() {
        return 0;
    }

    if let Ok(entries) = fs::read_dir(&deployables_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(content) = fs::read_to_string(entry.path())
                && let Ok(yaml) = serde_yaml::from_str::<serde_json::Value>(&content)
                && let Some(resources) = yaml.get("resources").and_then(|v| v.as_array())
            {
                for resource in resources {
                    if let Some(res_name) = resource.as_str()
                        && !resource_names.contains(&res_name.to_string())
                    {
                        warnings += 1;
                        warn!(
                            "Deployable {} references non-existent resource: {}",
                            entry.path().display(),
                            res_name
                        );
                    }
                }
            }
        }
    }

    warnings
}

fn print_text_report(results: &[ValidationResult], passed: usize, failed: usize, warnings: usize) {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          Platform Model Validation Report               ║");
    println!("╠══════════════════════════════════════════════════════════╣");

    for result in results {
        let status = if result.valid { "✅" } else { "❌" };
        println!("{} {} ({})", status, result.file, result.schema);

        if !result.valid {
            for error in &result.errors {
                println!("   └─ {}", error);
            }
        }
    }

    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ Summary:                                                 ║");
    println!("║   Passed:   {:<45} ║", passed);
    println!("║   Failed:   {:<45} ║", failed);
    println!("║   Warnings: {:<45} ║", warnings);
    println!("╚══════════════════════════════════════════════════════════╝\n");
}

fn print_json_report(results: &[ValidationResult], passed: usize, failed: usize, warnings: usize) {
    let report = serde_json::json!({
        "results": results,
        "summary": {
            "total": results.len(),
            "passed": passed,
            "failed": failed,
            "warnings": warnings
        }
    });

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
