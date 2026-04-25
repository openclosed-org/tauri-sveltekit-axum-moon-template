import process from "node:process";

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
      "justfile",
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
      "apps/**",
      "verification/**",
      "services/tenant-service/**",
    ],
    removeCandidates: [
      "docs/governance/**",
      "docs/archive/**",
      "release-plz.toml",
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

function main(): number {
  const args = parseArgs(process.argv.slice(2));
  const plan = PROFILE_PLANS[args.profile];

  console.log("=== Template Init Plan ===");
  console.log(`profile: ${args.profile}`);
  console.log(`mode: ${args.mode}`);
  console.log("scope: upstream-maintainer/open-source cleanup only");

  printList("Keep", plan.keep);
  printList("Review manually", plan.review);
  printList("Removal candidates", plan.removeCandidates);

  if (args.mode === "apply") {
    console.error("\napply mode is not implemented yet.");
    console.error("Use this output as a review plan first.");
    return 1;
  }

  console.log("\nDry-run only. No files were changed.");
  console.log(
    "This preview is intentionally conservative and does not propose broad code-tree deletion.",
  );
  console.log(
    "See docs/template-users/template-init.md for the current design contract.",
  );
  return 0;
}

process.exit(main());
