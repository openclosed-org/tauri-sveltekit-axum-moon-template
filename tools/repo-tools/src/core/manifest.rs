use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::core::command::run_capture;
use crate::core::fs::read;

#[derive(Debug, Deserialize)]
pub(crate) struct RoutingRulesManifest {
    #[serde(default)]
    pub(crate) dispatch_order: Vec<String>,
    #[serde(default)]
    pub(crate) rules: Vec<RoutingRule>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoutingRule {
    pub(crate) r#match: String,
    pub(crate) primary: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GateMatrixManifest {
    #[serde(default)]
    pub(crate) path_rules: Vec<GatePathRule>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct GatePathRule {
    #[serde(default)]
    pub(crate) r#match: Vec<String>,
    #[serde(default)]
    pub(crate) recommended: Vec<String>,
    #[serde(default)]
    pub(crate) required: Vec<String>,
    #[serde(default)]
    pub(crate) note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct CodemapManifest {
    #[serde(default)]
    pub(crate) generated_readonly: Vec<String>,
    #[serde(default)]
    pub(crate) write_boundaries: BTreeMap<String, CodemapBoundary>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CodemapBoundary {
    #[serde(default)]
    pub(crate) may_modify: Vec<String>,
    #[serde(default)]
    pub(crate) must_not_modify: Vec<String>,
}

pub(crate) fn cargo_metadata(root: &Path) -> Result<JsonValue> {
    let outcome = run_capture(
        "cargo",
        &["metadata", "--no-deps", "--format-version", "1"],
        Some(root),
    )?;
    if !outcome.success {
        bail!(
            "cargo metadata failed: {}",
            if outcome.error.is_empty() {
                outcome.output
            } else {
                outcome.error
            }
        );
    }
    serde_json::from_str(&outcome.output).context("failed to parse cargo metadata json")
}

pub(crate) fn load_routing_rules(root: &Path) -> Result<RoutingRulesManifest> {
    load_yaml(root.join("agent/manifests/routing-rules.yml"))
}

pub(crate) fn load_gate_matrix(root: &Path) -> Result<GateMatrixManifest> {
    load_yaml(root.join("agent/manifests/gate-matrix.yml"))
}

pub(crate) fn load_codemap(root: &Path) -> Result<CodemapManifest> {
    load_yaml(root.join("agent/codemap.yml"))
}

fn load_yaml<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let path = path.as_ref();
    serde_yaml::from_str(&read(path)?)
        .with_context(|| format!("failed to parse {}", path.display()))
}

#[allow(dead_code)]
pub(crate) fn first_top_level_yaml_string(value: &serde_yaml::Value, key: &str) -> Option<String> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml::Value::from(key)))
        .and_then(serde_yaml::Value::as_str)
        .map(ToOwned::to_owned)
}

#[allow(dead_code)]
pub(crate) fn recursive_yaml_lookup<'a>(
    value: &'a serde_yaml::Value,
    key: &str,
) -> Vec<&'a serde_yaml::Value> {
    let mut values = Vec::new();
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            for (k, v) in mapping {
                if k.as_str() == Some(key) {
                    values.push(v);
                }
                values.extend(recursive_yaml_lookup(v, key));
            }
        }
        serde_yaml::Value::Sequence(sequence) => {
            for item in sequence {
                values.extend(recursive_yaml_lookup(item, key));
            }
        }
        _ => {}
    }
    values
}

#[allow(dead_code)]
pub(crate) fn parse_simple_yaml_exports(plain: &str) -> BTreeMap<String, String> {
    let mut exports = BTreeMap::new();
    if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(plain) {
        if let Some(mapping) = value
            .get("stringData")
            .and_then(serde_yaml::Value::as_mapping)
        {
            for (key, value) in mapping {
                if let (Some(key), Some(value)) = (key.as_str(), value.as_str()) {
                    exports.insert(key.to_string(), value.to_string());
                }
            }
        } else if let Some(mapping) = value.as_mapping() {
            for (key, value) in mapping {
                if let (Some(key), Some(value)) = (key.as_str(), value.as_str()) {
                    exports.insert(key.to_string(), value.to_string());
                }
            }
        }
    }
    exports
}
