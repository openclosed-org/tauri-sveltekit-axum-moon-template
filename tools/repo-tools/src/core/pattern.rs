use regex::Regex;

pub(crate) fn strip_rust_comments(source: &str) -> String {
    let without_block = Regex::new(r"/\*[\s\S]*?\*/")
        .expect("valid regex")
        .replace_all(source, "")
        .to_string();
    Regex::new(r"(?m)^\s*//.*$")
        .expect("valid regex")
        .replace_all(&without_block, "")
        .to_string()
}

pub(crate) fn pattern_matches(target: &str, pattern: &str) -> bool {
    if pattern == format!("{target}/**") {
        return true;
    }
    if let Some(base) = pattern.strip_suffix("/**") {
        return target == base || target.starts_with(&format!("{base}/"));
    }
    if let Some(base) = pattern.strip_suffix("/*") {
        if !target.starts_with(&format!("{base}/")) {
            return false;
        }
        let rest = &target[base.len() + 1..];
        return !rest.is_empty() && !rest.contains('/');
    }
    target == pattern
}

pub(crate) fn same_module(source: &str, target: &str) -> bool {
    let source_parts = source.split('/').collect::<Vec<_>>();
    let target_parts = target.split('/').collect::<Vec<_>>();
    source_parts.len() > 1
        && target_parts.len() > 1
        && source_parts[0] == target_parts[0]
        && source_parts[1] == target_parts[1]
}

pub(crate) fn except_path(source: &str, target: &str, exceptions: &[String]) -> bool {
    exceptions.iter().any(|pattern| {
        let (source_pattern, target_pattern) = pattern.split_once(':').unwrap_or((pattern, "*"));
        pattern_matches(source, source_pattern)
            && (target_pattern == "*" || pattern_matches(target, target_pattern))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_recursive_patterns() {
        assert!(pattern_matches("services/counter-service", "services/**"));
        assert!(pattern_matches(
            "services/counter-service/src",
            "services/**"
        ));
        assert!(!pattern_matches("servers/web-bff", "services/**"));
    }

    #[test]
    fn matches_single_segment_patterns() {
        assert!(pattern_matches("services/counter-service", "services/*"));
        assert!(!pattern_matches(
            "services/counter-service/src",
            "services/*"
        ));
    }

    #[test]
    fn detects_same_top_level_module() {
        assert!(same_module(
            "services/counter-service",
            "services/counter-service/src"
        ));
        assert!(!same_module(
            "services/counter-service",
            "services/auth-service"
        ));
    }

    #[test]
    fn matches_source_target_exceptions() {
        let exceptions = vec!["services/**:packages/contracts/**".to_string()];
        assert!(except_path(
            "services/counter-service",
            "packages/contracts/api",
            &exceptions
        ));
        assert!(!except_path(
            "services/counter-service",
            "servers/web-bff",
            &exceptions
        ));
    }
}
