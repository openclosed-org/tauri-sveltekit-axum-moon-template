---
phase: 08-agent-dualpath-prompts
plan: 02
subsystem: agent-infrastructure
tags: [prompts, verification, agent-friendly]
dependency_graph:
  requires: [AGENT-DEV-01]
  provides: [prompt-templates, phase5-verification]
  affects: [.agents/prompts/, .planning/phases/05-agent-friendly/]
tech_stack:
  added: []
  patterns: [prompt-templates, verification-docs, playbook-references]
key_files:
  created:
    - .agents/prompts/add-feature.md
    - .agents/prompts/add-host.md
    - .agents/prompts/refactor-boundary.md
    - .planning/phases/05-agent-friendly/05-VERIFICATION.md
  modified: []
decisions:
  - "Prompt templates use ## Purpose heading (not inline **Purpose:**) to match verification check"
  - "Each prompt template references create-feature playbook and boundary-compliance rubric"
  - "VERIFICATION.md marks skills and prompts as deferred per D-08"
metrics:
  duration_minutes: 5
  completed_date: "2026-04-03"
---

# Phase 08 Plan 02: Prompts Templates + Phase 5 VERIFICATION.md Summary

## One-liner

补全 `.agents/prompts/` 三个 prompt 模板（add-feature、add-host、refactor-boundary），并生成 Phase 5 VERIFICATION.md 验证已有 playbooks + rubrics 的可用性。

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | 创建三个 prompt 模板 | see below | `.agents/prompts/add-feature.md`, `.agents/prompts/add-host.md`, `.agents/prompts/refactor-boundary.md` |
| 2 | 生成 Phase 5 VERIFICATION.md | see below | `.planning/phases/05-agent-friendly/05-VERIFICATION.md` |

## Task 1: Prompt Templates

### add-feature.md (80 lines)
- Purpose: 引导 agent 添加新功能模块，从 contract 到 frontend 消费
- 引用 `create-feature.md` playbook 作为详细流程参考
- 包含快速检查清单：contracts DTO、domain port、usecases service、adapter、host、frontend
- 提醒边界合规规则

### add-host.md (95 lines)
- Purpose: 引导 agent 添加新的 Tauri host adapter
- 步骤：创建 Tauri command（参考 counter.rs 模式）、注册到 mod.rs、注册到 invoke_handler
- 提醒：command 通过 AppHandle 获取 state，委托给 usecases service
- 包含前端 IPC 客户端模板

### refactor-boundary.md (95 lines)
- Purpose: 引导 agent 进行边界重构
- 引用 boundary-compliance rubric 作为判断标准
- 步骤：识别违规 import → 确定正确归属层 → 迁移代码 → 验证
- 提醒：保持接口不变，只移动实现位置

## Task 2: Phase 5 VERIFICATION.md

创建了 `.planning/phases/05-agent-friendly/05-VERIFICATION.md`，包含 4 个验证步骤：
1. **File Existence Check** — 验证 5 个 playbooks/rubrics 文件存在
2. **Format Validation** — 验证每个文件包含标题、目的/触发、步骤、验证章节
3. **Content Executability** — 验证每个文件引用真实路径和可执行命令
4. **Cross-Reference Consistency** — 验证 playbook 引用 rubric，rubric 引用 AGENTS.md

包含 Overall Result 表格和 Deferred Items 部分（skills 和 prompts 标记为 deferred）。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Prompt templates used inline **Purpose:** instead of ## Purpose heading**
- **Found during:** Task 1 verification
- **Issue:** Plan verification check `grep -l "## Purpose"` failed because templates used `**Purpose:**` inline format
- **Fix:** Changed all three templates to use `## Purpose` section heading
- **Files modified:** `.agents/prompts/add-feature.md`, `.agents/prompts/add-host.md`, `.agents/prompts/refactor-boundary.md`
- **Commit:** fix(08-02): add ## Purpose heading to all three prompt templates

## Verification Results

- [x] 3 prompt files exist in `.agents/prompts/`
- [x] Each prompt file contains `## Purpose` heading
- [x] `find .agents/prompts/ -type f -name "*.md" | wc -l` returns 3
- [x] `05-VERIFICATION.md` exists with 4 verification steps
- [x] `05-VERIFICATION.md` contains Overall Result table
- [x] `05-VERIFICATION.md` marks skills and prompts as deferred
- [x] All 5 verification targets (playbooks + rubrics) exist

## Self-Check: PASSED

All created files verified to exist. All verification checks pass.
