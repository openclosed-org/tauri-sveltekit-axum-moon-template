# Template consumer commands

# Preview how this repository would be reduced for template consumers.
template-init PROFILE='backend-core' MODE='dry-run':
    bun run scripts/template-init.ts --profile "{{PROFILE}}" --mode "{{MODE}}"

# Repository-level SemVer check for the current 0.1.x template line.
semver-check BASELINE='' RELEASE_TYPE='minor':
    #!/usr/bin/env bash
    set -euo pipefail
    baseline="{{BASELINE}}"
    release_type="{{RELEASE_TYPE}}"

    if [ -z "$baseline" ]; then
      baseline="$(git tag -l 'v0.1.*' | sort -V | tail -n 1)"
    fi

    if [ -z "$baseline" ]; then
      echo "No v0.1.x repository tag found; skipping SemVer baseline check until the first official 0.1.x release exists."
      exit 0
    fi

    echo "Using SemVer baseline: $baseline"
    echo "Using release type: $release_type"

    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_api
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_events
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_errors
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package contracts_auth
    cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package sdk-counter
