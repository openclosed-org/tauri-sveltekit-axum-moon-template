# Shared

> 纯工具函数层。仅包含与具体业务无关的通用工具。

## 当前结构

```
shared/
└── utils/   # ID 生成、时间格式化、加密工具
```

## 已清理项

以下子目录已删除（零依赖 stub）：
- `errors/` — 仅 re-export `contracts_errors`
- `types/` — 空占位
- `config/` — 空占位
- `env/` — 空占位
- `testing/` — 空占位

## 规则

**不新增**新的 `shared/*` 子目录。通用能力优先放入 `kernel/` 或 `contracts/`。
