# Template consumer commands

# Preview how this repository would be reduced for template consumers.
template-init PROFILE='backend-core' MODE='dry-run':
    bun run scripts/template-init.ts --profile "{{PROFILE}}" --mode "{{MODE}}"

# Repository-level SemVer check for the current template release line.
semver-check BASELINE='' RELEASE_TYPE='minor':
    #!/usr/bin/env bash
    set -euo pipefail
    baseline="{{BASELINE}}"
    release_type="{{RELEASE_TYPE}}"

    if [ -z "$baseline" ]; then
      baseline="$(git tag -l 'v[0-9]*.[0-9]*.[0-9]*' | sort -V | tail -n 1)"
    fi

    if [ -z "$baseline" ]; then
      echo "No repository semver tag found; skipping SemVer baseline check until the first official template release exists."
      exit 0
    fi

    echo "Using SemVer baseline: $baseline"
    echo "Using release type: $release_type"

    packages="contracts_api contracts_events contracts_errors contracts_auth"

    for pkg in $packages; do
      if git cat-file -e "$baseline:$(cargo metadata --no-deps --format-version 1 | jq -r --arg pkg "$pkg" '.packages[] | select(.name == $pkg) | .manifest_path' | sed 's#^.*/tauri-sveltekit-axum-moon-template/##')" 2>/dev/null; then
        cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package "$pkg"
      else
        echo "Skipping $pkg: package did not exist at baseline $baseline"
      fi
    done
