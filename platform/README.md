# Platform — 平台真理源

> **职责**: 描述平台，不实现业务。
> 决定"服务是什么、怎么部署、依赖什么资源、用什么策略、在什么拓扑运行"。

## 核心原则

1. **Model First**: 所有平台概念先在 `model/` 中定义，再生成其他产物
2. **Schema Driven**: 所有 YAML 文件必须能被 `schema/` 中的 JSON Schema 校验
3. **Generators Only**: `catalog/` 只能由 `generators/` 生成，不可手工修改
4. **Reproducible**: 删除所有生成产物后，重新生成必须零 diff

## 目录结构

```
platform/
├── README.md              ← 你在这里
├── schema/                ← JSON Schema 定义
│   ├── service.schema.json
│   ├── deployable.schema.json
│   ├── resource.schema.json
│   ├── workflow.schema.json
│   ├── topology.schema.json
│   └── policy.schema.json
├── model/                 ← 平台模型实例（YAML）
│   ├── services/          ← 服务定义（业务能力库）
│   ├── deployables/       ← 部署单元（进程/容器）
│   ├── resources/         ← 外部资源（DB/Cache/消息队列）
│   ├── workflows/         ← 业务流程（编排用例）
│   ├── policies/          ← 策略定义（超时/重试/幂等）
│   ├── topologies/        ← 拓扑定义（部署形态）
│   └── environments/      ← 环境配置（dev/staging/prod）
├── generators/            ← 代码/文档/配置生成器
│   ├── contracts/         ← 生成 API 契约
│   ├── sdk/               ← 生成 SDK 客户端
│   ├── compose/           ← 生成 Docker Compose
│   ├── kustomize/         ← 生成 Kustomize 配置
│   ├── flux/              ← 生成 Flux GitOps 配置
│   └── docs/              ← 生成架构文档
├── validators/            ← 校验工具
│   ├── model-lint/        ← Schema 校验 + 引用检查
│   ├── dependency-graph/  ← 依赖环检测
│   ├── contract-drift/    ← 契约漂移检测
│   ├── topology-check/    ← 拓扑一致性检查
│   ├── security-check/    ← 安全检查
│   └── observability-check/ ← 可观测性检查
└── catalog/               ← 生成产物（可审查，Git 追踪）
    ├── services.generated.yaml
    ├── deployables.generated.yaml
    ├── resources.generated.yaml
    ├── topology.generated.md
    └── architecture.generated.md
```

## 依赖方向

```
schema/  ←  定义所有概念的结构
  ↑
model/   ←  符合 schema 的实例数据
  ↑
generators/  ←  读取 model/ 生成各类产物
  ↑
validators/  ←  校验 model/ 合规性 + 生成产物一致性
```

## 使用方式

### 校验平台模型

```bash
just validate-platform
```

### 生成平台目录

```bash
just gen-platform
```

### 添加新服务

1. 在 `platform/model/services/<name>.yaml` 中定义服务
2. 运行 `just validate-platform` 确保合规
3. 运行 `just gen-platform` 更新目录
4. 在 `services/<name>/` 中实现业务逻辑

## 验证规则

- 所有 `model/*.yaml` 必须通过对应 schema 校验
- `model/` 中所有引用必须可解析（无悬空引用）
- 依赖图无环
- `catalog/` 中所有生成产物必须可重现
- 删除 `catalog/` 后重新生成必须零 diff

## 与 ARCHITECTURE.md 的关系

`platform/` 是 ARCHITECTURE.md §3.3 的具体实现。
所有规则以 ARCHITECTURE.md 为准，本目录只负责落地。
