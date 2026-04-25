# Template consumer commands

# Preview how this repository would be reduced for template consumers.
template-init PROFILE='backend-core' MODE='dry-run':
    bun run scripts/template-init.ts --profile "{{PROFILE}}" --mode "{{MODE}}"

# Repository-level SemVer check for release automation visibility.
semver-check:
    cargo semver-checks --package contracts_api
    cargo semver-checks --package contracts_events
    cargo semver-checks --package contracts_errors
    cargo semver-checks --package contracts_auth
    cargo semver-checks --package sdk-counter
