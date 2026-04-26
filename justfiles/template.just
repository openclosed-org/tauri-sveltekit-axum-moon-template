# Template consumer commands

# Preview how this repository would be reduced for template consumers.
template-init PROFILE='backend-core' MODE='dry-run':
    bun run scripts/template-init.ts --profile "{{PROFILE}}" --mode "{{MODE}}"

# Repository-level SemVer check for the current template release line.
semver-check BASELINE='' RELEASE_TYPE='minor' RELEASE_TAG_GLOB='v[0-9]*.[0-9]*.[0-9]*':
    #!/usr/bin/env bash
    set -euo pipefail
    baseline="{{BASELINE}}"
    release_type="{{RELEASE_TYPE}}"
    release_tag_glob="{{RELEASE_TAG_GLOB}}"
    repo_root="$(git rev-parse --show-toplevel)"

    if [ -z "$baseline" ]; then
      baseline="$(git tag -l "$release_tag_glob" | sort -V | tail -n 1)"
    fi

    if [ -z "$baseline" ]; then
      echo "No repository semver tag found; skipping SemVer baseline check until the first official template release exists."
      exit 0
    fi

    if ! git rev-parse -q --verify "refs/tags/$baseline" >/dev/null; then
      echo "Configured SemVer baseline tag does not exist: $baseline"
      exit 1
    fi

    echo "Using SemVer baseline: $baseline"
    echo "Using release type: $release_type"
    echo "Using release tag glob: $release_tag_glob"

    packages="contracts_api contracts_events contracts_errors contracts_auth"

    for pkg in $packages; do
      manifest_path="$(cargo metadata --no-deps --format-version 1 | jq -r --arg pkg "$pkg" '.packages[] | select(.name == $pkg) | .manifest_path')"
      manifest_rel_path="${manifest_path#$repo_root/}"

      if git cat-file -e "$baseline:$manifest_rel_path" 2>/dev/null; then
        cargo semver-checks --baseline-rev "$baseline" --release-type "$release_type" --package "$pkg"
      else
        echo "Skipping $pkg: package did not exist at baseline $baseline"
      fi
    done
