import process from "node:process";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";

type Mode = "dry-run" | "apply";
type Profile = "backend-core" | "backend-desktop" | "full-research";

interface ParsedArgs {
  mode: Mode;
  profile: Profile;
}

const DEFAULT_MODE: Mode = "dry-run";
const DEFAULT_PROFILE: Profile = "backend-core";

const PROFILE_PLANS: Record<
  Profile,
  { keep: string[]; review: string[]; removeCandidates: string[] }
> = {
  "backend-core": {
    keep: [
      "README.md",
      "LICENSE",
      "AGENTS.md",
      "agent/**",
      "docs/operations/**",
      "docs/contracts/README.md",
      "services/counter-service/**",
      "servers/bff/web-bff/**",
      "workers/outbox-relay/**",
      "workers/projector/**",
      "packages/contracts/**",
      "packages/kernel/**",
      "packages/platform/**",
      "packages/messaging/**",
      "packages/data/**",
      "packages/data-traits/**",
      "packages/data-adapters/turso/**",
      "packages/observability/**",
      "infra/**",
      "Justfile",
      "justfiles/setup.just",
      "justfiles/dev.just",
      "justfiles/test.just",
      "justfiles/quality.just",
      "justfiles/platform.just",
      "justfiles/sops.just",
    ],
    review: [
      "CONTRIBUTING.md",
      "CODE_OF_CONDUCT.md",
      "docs/template-users/**",
      ".github/workflows/**",
      "verification/**",
      "services/tenant-service/**",
    ],
    removeCandidates: [
      "apps/**",
      "packages/ui/**",
      "verification/e2e/**",
      "scripts/dev-desktop.ts",
      "scripts/test/run-frontend.ts",
      "scripts/e2e/**",
      "docs/governance/**",
      "docs/archive/**",
      "release-plz.toml",
      "release-plz.template.toml",
      ".github/workflows/release-plz.yml",
      "tools/repo-release/**",
      ".github/ISSUE_TEMPLATE/**",
      ".github/pull_request_template.md",
    ],
  },
  "backend-desktop": {
    keep: ["everything in backend-core", "apps/desktop/**"],
    review: ["agent/**", "docs/architecture/**", "docs/archive/**"],
    removeCandidates: ["agent/**", "docs/architecture/**"],
  },
  "full-research": {
    keep: ["entire repository"],
    review: [],
    removeCandidates: [],
  },
};

function normalizeProfile(value: string | undefined): Profile | null {
  if (!value) return null;
  const raw = value.startsWith("PROFILE=")
    ? value.slice("PROFILE=".length)
    : value;
  if (
    raw === "backend-core" ||
    raw === "backend-desktop" ||
    raw === "full-research"
  ) {
    return raw;
  }
  return null;
}

function normalizeMode(value: string | undefined): Mode | null {
  if (!value) return null;
  const raw = value.startsWith("MODE=") ? value.slice("MODE=".length) : value;
  if (raw === "dry-run" || raw === "apply") {
    return raw;
  }
  return null;
}

function parseArgs(argv: string[]): ParsedArgs {
  let mode = DEFAULT_MODE;
  let profile = DEFAULT_PROFILE;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const inlineProfile = normalizeProfile(arg);
    if (inlineProfile) {
      profile = inlineProfile;
      continue;
    }

    const inlineMode = normalizeMode(arg);
    if (inlineMode) {
      mode = inlineMode;
      continue;
    }

    if (arg === "--mode") {
      const value = normalizeMode(argv[index + 1]);
      if (value) {
        mode = value;
        index += 1;
        continue;
      }
    }

    if (arg === "--profile") {
      const value = normalizeProfile(argv[index + 1]);
      if (value) {
        profile = value;
        index += 1;
        continue;
      }
    }

    console.error(`Unknown argument: ${arg}`);
    console.error(
      "Usage: bun run scripts/template-init.ts [--profile backend-core|backend-desktop|full-research] [--mode dry-run|apply]",
    );
    process.exit(1);
  }

  return { mode, profile };
}

function printList(title: string, values: string[]): void {
  console.log(`\n${title}`);
  if (values.length === 0) {
    console.log("  (none)");
    return;
  }

  for (const value of values) {
    console.log(`  - ${value}`);
  }
}

function removePathPattern(pattern: string): void {
  const path = pattern.endsWith("/**") ? pattern.slice(0, -3) : pattern;
  rmSync(path, { recursive: true, force: true });
  console.log(`  removed ${path}`);
}

function ensureSafeToApply(): void {
  if (process.env.TEMPLATE_INIT_ALLOW_DIRTY === "1") {
    console.log("\nDirty worktree check bypassed by TEMPLATE_INIT_ALLOW_DIRTY=1.");
    return;
  }

  const gitProbe = spawnSync("git", ["rev-parse", "--is-inside-work-tree"], {
    encoding: "utf8",
  });
  if (gitProbe.status !== 0 || gitProbe.stdout.trim() !== "true") {
    console.log("\nNo git worktree detected; skipping dirty worktree check.");
    return;
  }

  const status = spawnSync("git", ["status", "--porcelain"], {
    encoding: "utf8",
  });
  if (status.status !== 0) {
    console.error("Unable to inspect git worktree status; refusing to apply cleanup.");
    process.exit(1);
  }

  if (status.stdout.trim().length > 0) {
    console.error("Refusing to apply template cleanup with a dirty worktree.");
    console.error("Commit or stash local changes first, or set TEMPLATE_INIT_ALLOW_DIRTY=1 after reviewing the risk.");
    process.exit(1);
  }
}

function removeRepositoryReleaseAnchor(): void {
  const cargoTomlPath = "Cargo.toml";
  if (!existsSync(cargoTomlPath)) {
    return;
  }

  const original = readFileSync(cargoTomlPath, "utf8");
  if (!original.includes('name = "axum-harness"')) {
    return;
  }

  let next = original.replace(
    /^\[package\]\nname = "axum-harness"\n[\s\S]*?\n(?=\[workspace\]\n)/,
    "",
  );
  next = next.replace(
    /\n\[lib\]\npath = "tools\/repo-release\/src\/lib\.rs"\n\n\[lints\]\nworkspace = true\n/,
    "\n",
  );

  if (next !== original) {
    writeFileSync(cargoTomlPath, next);
    console.log("  removed repository release anchor from Cargo.toml");
  }
}

function main(): number {
  const args = parseArgs(process.argv.slice(2));
  const plan = PROFILE_PLANS[args.profile];

  console.log("=== Template Init Plan ===");
  console.log(`profile: ${args.profile}`);
  console.log(`mode: ${args.mode}`);
  console.log("scope: backend-core template cleanup");

  printList("Keep", plan.keep);
  printList("Review manually", plan.review);
  printList("Removal candidates", plan.removeCandidates);

  if (args.mode === "apply") {
    ensureSafeToApply();
    console.log("\nApplying removal candidates:");
    for (const pattern of plan.removeCandidates) {
      removePathPattern(pattern);
    }
    if (args.profile === "backend-core") {
      removeRepositoryReleaseAnchor();
    }
    console.log("\nApply complete. Run `just audit-backend-core` and `just verify` next.");
    return 0;
  }

  console.log("\nDry-run only. No files were changed.");
  console.log("Run with MODE=apply only after reviewing the removal candidates.");
  console.log(
    "See docs/template-users/template-init.md for the current design contract.",
  );
  return 0;
}

process.exit(main());
