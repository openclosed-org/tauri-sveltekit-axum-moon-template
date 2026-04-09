# LLM Context — infiniloom 驱动的代码智能工具集
# 基于 tree-sitter AST + PageRank + 依赖图，为 LLM agent 提供结构化上下文
#
# 前置依赖: cargo install infiniloom
# 文档: https://github.com/Topos-Labs/infiniloom

# ── 默认排除模式 ─────────────────────────────────────────────

exclude_patterns := "--exclude 'node_modules' --exclude 'target' --exclude '.git' --exclude 'bun.lock' --exclude 'package-lock.json' --exclude '*.icns' --exclude '*.ico'"

# ── 内部工具 ─────────────────────────────────────────────────

# 确保符号索引存在（impact / diff 的前置依赖）
_llm-ensure-index:
    @test -d .infiniloom || (echo "⏳ 首次构建符号索引..." && infiniloom index .)

# ── 快速概览 ─────────────────────────────────────────────────

# 仓库扫描：文件数、语言分布、token 总量
llm-scan:
    infiniloom scan .

# 仓库扫描（安全检查：泄露的密钥、token）
llm-scan-safe:
    infiniloom scan . --security-check

# 符号地图（PageRank 排序，核心 struct/函数排前面）
llm-map:
    infiniloom map .

# 符号地图（指定 token 预算）
llm-map-budget BUDGET='4000':
    infiniloom map . --budget {{BUDGET}}

# 符号地图（仅 Rust 代码）
llm-map-rust:
    infiniloom map . --include '**/*.rs'

# 符号地图（仅 Svelte/TypeScript 前端）
llm-map-web:
    infiniloom map . --include '**/*.svelte' --include '**/*.ts'

# ── 上下文打包 ───────────────────────────────────────────────

# 生成 Claude 上下文（XML 格式，120k token 预算）
llm-pack FILE='.llm-context.xml':
    infiniloom pack . -f xml --max-tokens 120000 --full --redact-secrets -o {{FILE}}
    @echo "✅ 已生成: {{FILE}}"

# 生成 GPT 上下文（Markdown 格式，80k token 预算）
llm-pack-gpt FILE='.llm-context.md':
    infiniloom pack . -f markdown -m gpt4o --max-tokens 80000 --full --redact-secrets -o {{FILE}}
    @echo "✅ 已生成: {{FILE}}"

# 生成极致压缩上下文（仅签名，适合 token 极度紧张时）
llm-pack-skeleton FILE='.llm-skeleton.xml':
    infiniloom pack . -f xml --max-tokens 30000 -c aggressive --redact-secrets -o {{FILE}}
    @echo "✅ 已生成: {{FILE}}"

# 生成 TOON 格式（比 XML/MD 节省 ~40% token）
llm-pack-toon FILE='.llm-context.toon':
    infiniloom pack . -f toon --max-tokens 120000 --full --redact-secrets -o {{FILE}}
    @echo "✅ 已生成: {{FILE}}"

# 打包指定目录的上下文
llm-pack-dir DIR BUDGET='120000' FILE='':
    @if [ -z "{{FILE}}" ]; then \
        infiniloom pack {{DIR}} -f xml --max-tokens {{BUDGET}} --full --redact-secrets; \
    else \
        infiniloom pack {{DIR}} -f xml --max-tokens {{BUDGET}} --full --redact-secrets -o {{FILE}}; \
        echo "✅ 已生成: {{FILE}}"; \
    fi

# 查看 token 分布（哪些文件占了最多 token）
llm-pack-tree:
    infiniloom pack . --token-tree --no-content

# ── 符号索引 ─────────────────────────────────────────────────

# 构建/更新符号索引（diff 和 impact 的前置条件）
llm-index:
    infiniloom index .

# 强制重建索引
llm-index-rebuild:
    infiniloom index . --force

# 查看索引状态
llm-index-status:
    infiniloom index . --status

# ── 爆炸半径分析 ─────────────────────────────────────────────

# 分析文件的影响范围（改之前必跑）
llm-impact TARGET: _llm-ensure-index
    infiniloom impact . {{TARGET}} --depth 2

# 分析文件影响范围（含调用图）
llm-impact-graph TARGET: _llm-ensure-index
    infiniloom impact . {{TARGET}} --depth 2 --call-graph

# 分析影响范围（JSON 输出，方便管道处理）
llm-impact-json TARGET: _llm-ensure-index
    infiniloom impact . {{TARGET}} --depth 2 --json

# 分析符号的影响范围（按符号名而非文件路径）
llm-impact-symbol SYMBOL: _llm-ensure-index
    infiniloom impact . {{SYMBOL}} --symbol --depth 2 --call-graph

# ── Diff 与代码审查 ─────────────────────────────────────────

# 未暂存变更的上下文（含依赖分析）
llm-diff: _llm-ensure-index
    infiniloom diff . --include-diff --depth 2 -f xml -o .llm-diff.xml
    @echo "✅ 已生成: .llm-diff.xml"

# 已暂存变更的上下文
llm-diff-staged: _llm-ensure-index
    infiniloom diff . --staged --include-diff --depth 2 -f xml -o .llm-diff-staged.xml
    @echo "✅ 已生成: .llm-diff-staged.xml"

# 分支对比上下文（PR review 用）
llm-diff-branch REF='main': _llm-ensure-index
    infiniloom diff . {{REF}}..HEAD --include-diff --depth 2 -f xml -o .llm-diff-review.xml
    @echo "✅ 已生成: .llm-diff-review.xml（{{REF}}..HEAD）"

# 分支对比上下文（Markdown 格式，适合丢给 ChatGPT）
llm-diff-branch-md REF='main': _llm-ensure-index
    infiniloom diff . {{REF}}..HEAD --include-diff --depth 2 -f markdown -o .llm-diff-review.md
    @echo "✅ 已生成: .llm-diff-review.md（{{REF}}..HEAD）"

# 深度分析（含调用者 + git 历史）
llm-diff-deep REF='main': _llm-ensure-index
    infiniloom diff . {{REF}}..HEAD --include-diff --depth 3 --include-history -f xml -o .llm-diff-deep.xml
    @echo "✅ 已生成: .llm-diff-deep.xml（depth=3 + 历史）"

# ── 分块与 Embedding（RAG）──────────────────────────────────

# 按模块分块（适合多轮对话逐步注入上下文）
llm-chunk BUDGET='8000':
    infiniloom chunk . -s module --max-tokens {{BUDGET}} --priority-first

# 按模块分块到目录
llm-chunk-save BUDGET='8000' DIR='.llm-chunks':
    infiniloom chunk . -s module --max-tokens {{BUDGET}} --priority-first -o {{DIR}}
    @echo "✅ 已分块到: {{DIR}}"

# 生成 embedding chunks（JSONL 格式，喂向量库）
llm-embed FILE='.llm-embed.jsonl':
    infiniloom embed . -o {{FILE}}
    @echo "✅ 已生成: {{FILE}}"

# 增量更新 embedding（只处理变更文件）
llm-embed-diff FILE='.llm-embed.jsonl':
    infiniloom embed . --diff -o {{FILE}}
    @echo "✅ 增量更新: {{FILE}}"

# ── 帮助 ────────────────────────────────────────────────────

# 显示所有 llm-* 命令及用法
llm-help:
    @echo "=== infiniloom LLM 工具集 ==="
    @echo ""
    @echo "概览:"
    @echo "  just llm-scan              仓库扫描（文件/语言/token）"
    @echo "  just llm-scan-safe         扫描 + 安全检查"
    @echo "  just llm-map               符号地图（PageRank 排序）"
    @echo "  just llm-map-budget 4000   符号地图（指定预算）"
    @echo "  just llm-map-rust          符号地图（仅 Rust）"
    @echo "  just llm-map-web           符号地图（仅 Svelte/TS）"
    @echo ""
    @echo "打包上下文:"
    @echo "  just llm-pack              Claude XML（120k tokens）"
    @echo "  just llm-pack-gpt          GPT Markdown（80k tokens）"
    @echo "  just llm-pack-skeleton     极致压缩（仅签名）"
    @echo "  just llm-pack-toon         TOON 格式（-40% tokens）"
    @echo "  just llm-pack-dir servers/ 打包指定目录"
    @echo "  just llm-pack-tree         token 分布分析"
    @echo ""
    @echo "索引:"
    @echo "  just llm-index             构建/更新符号索引"
    @echo "  just llm-index-rebuild     强制重建索引"
    @echo "  just llm-index-status      查看索引状态"
    @echo ""
    @echo "爆炸半径（改 contracts/features/usecases 前必跑）:"
    @echo "  just llm-impact <file>     文件影响范围"
    @echo "  just llm-impact-graph <f>  含调用图"
    @echo "  just llm-impact-symbol <s> 按符号名分析"
    @echo ""
    @echo "Diff 审查:"
    @echo "  just llm-diff              未暂存变更上下文"
    @echo "  just llm-diff-staged       已暂存变更上下文"
    @echo "  just llm-diff-branch main  分支对比（PR review）"
    @echo "  just llm-diff-deep main    深度分析（+历史）"
    @echo ""
    @echo "分块 & Embedding:"
    @echo "  just llm-chunk             按模块分块（多轮对话）"
    @echo "  just llm-embed             生成向量库 chunks"
    @echo "  just llm-embed-diff        增量更新 chunks"
