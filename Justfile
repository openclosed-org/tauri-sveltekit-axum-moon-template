# Justfile — 统一人类/Agent 命令入口
# 工具链由 mise 管理,任务编排由 moon 负责
# Justfile 只暴露稳定、可读性高的接口
set export  # 导出环境变量到子进程

# ── 模块导入 ────────────────────────────────────────────────
import? 'justfiles/setup.just'
import? 'justfiles/dev.just'
import? 'justfiles/test.just'
import? 'justfiles/quality.just'
import? 'justfiles/build.just'
import? 'justfiles/migrate.just'
import? 'justfiles/clean.just'
import? 'justfiles/deploy.just'
import? 'justfiles/processes.just'
import? 'justfiles/skills.just'
import? 'justfiles/platform.just'
import? 'justfiles/gates.just'
import? 'justfiles/sops.just'

# ── 默认行为 ────────────────────────────────────────────────

default:
    @just --list
