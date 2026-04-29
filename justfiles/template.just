# Template consumer commands

# Preview how this repository would be reduced for template consumers.
template-init PROFILE='backend-core' MODE='dry-run':
    cargo run -p repo-tools -- template-init --profile "{{PROFILE}}" --mode "{{MODE}}"

# Verify the default backend-core lane is independent from optional app shells.
audit-backend-core MODE='dry-run':
    cargo run -p repo-tools -- audit-backend-core "{{MODE}}"

# Repository-level SemVer check for the current template release line.
semver-check BASELINE='' RELEASE_TYPE='minor' RELEASE_TAG_GLOB='v[0-9]*.[0-9]*.[0-9]*':
    cargo run -p repo-tools -- semver-check --baseline "{{BASELINE}}" --release-type "{{RELEASE_TYPE}}" --release-tag-glob "{{RELEASE_TAG_GLOB}}"
