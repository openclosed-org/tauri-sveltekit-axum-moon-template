# Template consumer commands

# Preview how this repository would be reduced for template consumers.
template-init PROFILE='backend-core' MODE='dry-run':
    bun run scripts/template-init.ts --profile "{{PROFILE}}" --mode "{{MODE}}"

# Repository-level SemVer check for release automation visibility.
semver-check BASELINE='' RELEASE_TYPE='minor':
    #!/usr/bin/env bash
    set -euo pipefail
    baseline="{{BASELINE}}"
    release_type="{{RELEASE_TYPE}}"

    if [ -z "$baseline" ]; then
      baseline="$(git describe --tags --abbrev=0)"
    fi

    echo "Using SemVer baseline: $baseline"
    echo "Using release type: $release_type"

    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_api
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_events
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_errors
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_auth
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package sdk-counter
