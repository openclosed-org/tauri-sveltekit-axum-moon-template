import { writeFileSync, mkdirSync, existsSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Directory Categories Generator (v2.0)
 *
 * Scans actual project directories and categorizes them by functional domain.
 * Output: agent/directory_categories.json — a soft-priority guide for Code Agents.
 *
 * Design philosophy: NOT a hard filter, but a search-priority hint.
 * Agents should always read 'shared' and 'contracts' regardless of task domain.
 *
 * Usage: bun run scripts/gen-directory-categories.ts
 *   or:  just gen-dir-cats
 */

const PROJECT_ROOT = process.cwd();

// Category assignment rules (first match wins)
// Patterns are regex tested against relative directory paths
const categoryPatterns: Array<{ pattern: RegExp; category: string }> = [
  // ── Agent & AI constraints
  { pattern: /^agent\//, category: "agent" },
  { pattern: /^\.agents\//, category: "agent" },

  // ── Contracts (single truth source)
  { pattern: /^packages\/contracts\//, category: "contracts" },

  // ── Shared abstractions (features + shared utils + core)
  { pattern: /^packages\/core\//, category: "shared" },
  { pattern: /^packages\/features\//, category: "shared" },
  { pattern: /^packages\/shared\//, category: "shared" },

  // ── Frontend packages & apps
  { pattern: /^packages\/ui\//, category: "frontend" },
  { pattern: /^packages\/sdk\/typescript\//, category: "frontend" },
  { pattern: /^apps\/web\//, category: "frontend" },
  { pattern: /^apps\/mobile\//, category: "frontend" },
  { pattern: /^apps\/desktop\/src\//, category: "frontend" },
  { pattern: /^apps\/browser-extension\//, category: "frontend" },

  // ── Web3 (before generic services/ rule)
  { pattern: /^packages\/web3\//, category: "web3" },
  { pattern: /^services\/indexer/, category: "web3" },
  { pattern: /^tools\/web3/, category: "web3" },

  // ── Backend servers & services
  { pattern: /^servers\//, category: "backend" },
  { pattern: /^apps\/bff\//, category: "backend" },
  { pattern: /^apps\/desktop\/src-tauri\//, category: "backend" },
  { pattern: /^services\//, category: "backend" },
  { pattern: /^packages\/adapters\//, category: "backend" },

  // ── Infrastructure & ops
  { pattern: /^infra/, category: "infra" },
  { pattern: /^ops/, category: "infra" },
  { pattern: /^scripts/, category: "infra" },
  { pattern: /^justfiles/, category: "infra" },
  { pattern: /^\.cargo/, category: "infra" },
  { pattern: /^\.github/, category: "infra" },
  { pattern: /^\.config/, category: "infra" },

  // ── Test fixtures
  { pattern: /^fixtures\//, category: "tests" },
];

// Test pattern: matches any directory path where "test" or "tests" is a standalone segment
// e.g. "apps/web/tests", "services/counter/tests/unit" but NOT "test-results" or "e2e-test"
const testSegmentPattern = /(?:^|\/)(?:tests?)(?:\/|$)/;

/**
 * Collect directories up to depth 2 from project root.
 * Skips node_modules, target, and dot-prefixed dirs (except .agents, .cargo, etc.)
 */
function collectDirectories(): string[] {
  const dirs: string[] = [];

  function walk(dir: string, depth: number) {
    if (depth > 2) return;
    const fullPath = dir === "" ? PROJECT_ROOT : join(PROJECT_ROOT, dir);
    try {
      const entries = readdirSync(fullPath, { withFileTypes: true });
      for (const entry of entries) {
        if (!entry.isDirectory()) continue;
        const name = entry.name;
        // Skip large/generated directories
        if (name === "node_modules" || name === "target" || name === ".jj" || name === ".moon" || name === ".cocoindex_code") continue;
        // Skip generic dot-directories we don't care about
        if (name.startsWith(".") && name !== ".agents" && name !== ".cargo" && name !== ".config" && name !== ".github") continue;

        const relPath = dir === "" ? name : join(dir, name);
        dirs.push(relPath);
        walk(relPath, depth + 1);
      }
    } catch {
      // Permission denied or other error — skip
    }
  }

  walk("", 0);
  return dirs;
}

/**
 * Assign each directory to a category based on first-match rule.
 * Returns Map<category, Set<directory>>
 */
function categorizeDirectories(dirs: string[]): Map<string, Set<string>> {
  const categories = new Map<string, Set<string>>();

  for (const dir of dirs) {
    // Check test segment pattern first (it's a catch-all for any depth)
    if (testSegmentPattern.test(dir)) {
      if (!categories.has("tests")) categories.set("tests", new Set());
      categories.get("tests")!.add(dir);
      continue;
    }

    for (const { pattern, category } of categoryPatterns) {
      if (pattern.test(dir)) {
        if (!categories.has(category)) categories.set(category, new Set());
        categories.get(category)!.add(dir);
        break;
      }
    }
  }

  return categories;
}

// ── Main ─────────────────────────────────────────────────────────────────

const dirs = collectDirectories();
const categoryMap = categorizeDirectories(dirs);

const output = {
  version: "2.0",
  generated_at: new Date().toISOString(),
  categories: {} as Record<string, string[]>,
  phase: "0 — 模块化单体（业务逻辑在 packages/core/usecases/ + servers/api/）",
  priority_rules: [
    "始终阅读 'shared' 和 'contracts' 分类，无论任务类型。它们是系统的抽象基石。",
    "前端任务 → 优先搜索 'frontend' + 'shared' + 'contracts'。",
    "后端任务 → 优先搜索 'backend' + 'shared' + 'contracts'。注意：Phase 0 业务逻辑在 packages/core/usecases/，不在 services/。",
    "全栈任务 → 平等对待 'frontend' + 'backend' + 'shared' + 'contracts'。",
    "基础设施任务 → 阅读 'infra' + 'agent'（约束定义）。",
    "测试文件在 'tests' 分类中 — 修改业务代码时不要忽略相关测试。",
    "新增业务模块时，应按目标架构在 services/<domain>/ 下创建（即使当前是 stub）。",
  ],
};

// Build category lists (sorted for deterministic output)
for (const [cat, dirSet] of categoryMap.entries()) {
  output.categories[cat] = Array.from(dirSet).sort();
}

// Ensure agent/ directory exists
const agentDir = join(PROJECT_ROOT, "agent");
if (!existsSync(agentDir)) {
  mkdirSync(agentDir, { recursive: true });
}

const outputPath = join(agentDir, "directory_categories.json");
writeFileSync(outputPath, JSON.stringify(output, null, 2));

console.log(`✅ Generated agent/directory_categories.json with ${categoryMap.size} categories.`);
console.log(`   Directories scanned: ${dirs.length}`);
for (const [cat, dirs] of Object.entries(output.categories)) {
  console.log(`   ${cat}: ${dirs.length} directories`);
}
