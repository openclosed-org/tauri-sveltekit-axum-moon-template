# Agent Harness

`agent/` 只保留最小的 agent 协作控制面，不承载业务逻辑、完整系统模型或形式化证明。

## 目录职责

1. `codemap.yml`
   负责目录边界、写权限、依赖方向、生成物只读位置、反模式与修改顺序。
2. `manifests/routing-rules.yml`
   负责 touched paths 到 subagent 的路由与派发顺序。
3. `manifests/gate-matrix.yml`
   负责按 changed paths、risk category、evidence level 选择 advisory、guardrail、invariant gates。

## 使用顺序

1. 先读根级 `AGENTS.md`。
2. 再读 `docs/architecture/harness-philosophy.md` 和 `agent/codemap.yml`，确认 harness 边界、导航地图与禁止事项。
3. 最后根据 `routing-rules.yml` 和 `gate-matrix.yml` 决定派发与验证。

## 说明

1. 详细 subagent 行为定义仍在 `.agents/skills/*/SKILL.md`。
2. 参考实现与真实开发模式优先从现有 `services/*`、`workers/*`、`servers/*` 和 `packages/contracts/*` 获取。
3. 如果 `agent/` 文档与代码冲突，以代码和可执行验证结果为准。
4. YAML 只能声明 intent 或 summary；不能单独证明系统语义正确。
