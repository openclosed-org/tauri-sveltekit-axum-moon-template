这份对照的核心结论先放前面：

1. docs/ARCHITECTURE.md 描述的是“最终态骨架”，当前仓库是“最终态骨架已大量落地，但仍混有旧分层和迁移中结构”的混合态。
2. 明显落后文档的，主要不是 platform/、workers/、verification/ 这些骨架层，而是 services/ 业务承载层、packages/ 的最终分层收敛、以及 apps/mobile / 部分 server 目录的完整性。
3. 明显领先文档的，是平台模型、校验器、生成产物、验证体系、ADR/重构文档沉淀，以及部分异步 worker 的落地程度。
4. 有一些地方不是“落后”，而是“实际比文档更合理或更成熟”，例如：
    - servers/bff/web-bff 与 servers/bff/admin-bff 用分组目录组织，而不是根下平铺，实际更利于扩展。
    - infra/ 比文档更丰富，额外有 docker/、k3s/、terraform/ 三条演进路径并存。
    - docs/refactoring/、justfiles/、scripts/ 这些补充层，虽然不在文档最终骨架里，但对真实协作和迁移管理是加分项。
      下面按你要求的两种视角完整汇报。
      总体判断
      先给三类定义，避免后文语义混淆：

- 落后：文档目标已明确，但当前仓库尚未按该目标落地，或者仍明显停留在旧结构。
- 领先：文档里只是目标或骨架，但当前仓库已经做出较完整实现或验证链路。
- 更好：当前实现不完全等于文档模板，但从工程组织上看更实用、更成熟，未必需要回退到文档写法。
  一、根目录 + 根目录文件汇总

1. 根目录整体
   当前根目录实际存在这些一级项：

- 隐藏/配置：.agents/、.cargo/、.config/、.github/、.mise.toml、.tool-versions、.editorconfig、.gitattributes、.gitignore
- 核心目录：agent/、apps/、docs/、fixtures/、infra/、ops/、packages/、platform/、servers/、services/、tools/、verification/、workers/
- 补充目录：justfiles/、scripts/
- 根文件：AGENTS.md、README.md、Cargo.toml、Cargo.lock、package.json、package-lock.json、bun-workspace.yaml、moon.yml、Justfile、rust-toolchain.toml、biome.json、clippy.toml、deny.toml、typos.toml、rustfmt.toml
  对照 docs/ARCHITECTURE.md：
  落后
- 缺少文档模板中的 apps/mobile/
- 缺少模板根文件 GOAL.md、CONTRIBUTING.md、CHANGELOG.md
- 文档模板里根层强调“所有工作区成员在 Cargo.toml / bun-workspace.yaml 可追踪”，但当前 bun-workspace.yaml 只列了 apps/web、apps/browser-extension、packages/ui，与实际项目规模不一致
- 文档模板把根层 README 视为仓库入口，但当前 README.md 实际写的是 scripts/ 说明，不是仓库总览
  领先
- 多了 .agents/ 与 AGENTS.md，说明 Agent 协作协议已经真正落地
- 多了 justfiles/ 模块化任务拆分，Justfile 只做稳定入口，工程上比单文件 justfile 更成熟
- 多了 scripts/ 作为跨平台脚本层，且与 Justfile/moon.yml 形成配套
- 根层已有 Cargo.lock、package-lock.json、.tool-versions、rustfmt.toml，工具链治理更细
  更好
- Justfile + justfiles/\*.just 的结构，比文档里单个 justfile 更适合多人/多阶段演进
- 根层多出 scripts/ 不属于文档模板，但这是当前仓库真实的自动化执行层，不应视为坏味道
- 根层保留 .refactoring-state.yaml、docs/refactoring/ 等迁移痕迹，虽然不“最终态”，但对长期演进有价值

---

2. Cargo.toml
   证据：Cargo.toml:1-93
   它列出的 workspace member 覆盖了：

- apps/desktop/src-tauri
- packages/kernel、packages/platform、packages/runtime
- packages/contracts/\*
- packages/shared/\*
- packages/adapters/\*
- packages/features/\*
- services/\*
- servers/\*
- workers/\*
- platform/validators/\*
- platform/generators
  落后
- 与文档目标的 packages/ 最终分层不一致，仍保留 packages/core、packages/shared、packages/features
- services/_ 仍是 _-service 命名，并未收敛到文档中的纯领域名
- servers/ 中仍有 servers/api、servers/indexer 这类过渡成员，不是文档中的最终拓扑命名
  领先
- workspace 已经把平台生成器、平台校验器、worker、BFF、桌面端、服务层都纳入统一工作区，实际覆盖比文档“骨架”更深
- workspace.dependencies 已形成统一依赖注入入口
  更好
- 在迁移期保留 packages/core/features/shared 进入工作区是务实做法，便于逐步搬迁，不必为了匹配文档先做大重构

---

3. package.json
   证据：package.json:1-5
   只有最小内容：
   {
   "name": "tauri-sveltekit-axum-moon-template",
   "private": true,
   "packageManager": "bun@1.3.11"
   }
   落后

- 相比文档的 monorepo 预期，这里几乎不承载工作区说明、脚本约定、前端工程信息
- 根层 JS 工作区信息较弱，更多依赖 bun-workspace.yaml 和脚本层
  领先
- 明确锁定 bun 版本，足够简洁
  更好
- 如果团队已把任务入口收口到 Justfile/moon.yml，根 package.json 保持极简并不是问题

---

4. bun-workspace.yaml
   证据：bun-workspace.yaml:1-4
   只包含：

- apps/web
- apps/browser-extension
- packages/ui
  落后
- 明显没有反映整个前端/JS 工作区现状
- 文档要求“所有工作区成员可追踪”，这里做不到
- apps/desktop/tests/e2e 实际有独立 package.json，却不在这里
- 文档强调 packages/sdk 作为前端消费入口，但当前没有反映
  领先
- 无明显领先点
  更好
- 无。这个文件是根层最明显的失真项之一

---

5. Justfile
   证据：Justfile:1-22
   领先

- 很符合文档“根层命令统一暴露”的原则
- 已模块化导入：
    - setup.just
    - dev.just
    - test.just
    - quality.just
    - build.just
    - migrate.just
    - clean.just
    - deploy.just
    - processes.just
    - skills.just
    - platform.just
      更好
- 比文档模板中的单文件 justfile 更适合演进
- “根层只暴露稳定接口，细节拆到 justfiles/” 是比模板更成熟的组织方式
  落后
- 还不能仅凭此文件确认文档第 9 节推荐命令是否全部落地，但入口机制本身是对的

---

6. moon.yml
   证据：moon.yml:1-220
   领先

- 已有 build/check/lint/test/doctor/dev-\* 等任务
- 已有 contracts-check、test-e2e-full 这类复合质量门
- 已把脚本、Rust 构建、前端任务整合到统一编排器
  落后
- 任务命名与文档第 9 节推荐命令并未完全对齐，例如没有直接看到：
    - gen-contracts
    - gen-sdk
    - verify-single-vps
    - verify-k3s
- 有些任务仍直接 cd apps/web && bun run ...，不完全符合“根层只调统一命令、不直拼深层脚本”的理想化描述
  更好
- moon + just 的双层模式，比只靠 just 更适合增量缓存和任务图

---

7. .mise.toml
   证据：.mise.toml:1-11
   领先

- 工具链已锁定：rust、node、bun
- 与文档要求一致，且粒度更细
  落后
- 无明显落后
  更好
- 与 .tool-versions 共存，兼容性更强

---

8. README.md
   证据：README.md:1-38
   落后

- 这是根层最明显落后项之一
- 当前内容实际上是 scripts/ 目录说明，不是仓库总览
- 文档希望新成员通过 README/docs 快速理解仓库，但根 README 现在无法承担这个角色
  领先
- 无
  更好
- 无。这个文件应该重写或搬迁为 scripts/README.md

---

9. AGENTS.md
   证据：AGENTS.md:1-104
   领先

- 已经把仓库级协作协议、工具规则、禁止读取目录、风险升级机制写清楚
- 这类 Agent 协议在文档模板中只有 agent/ 目录约束，而这里是根入口级约束，成熟度更高
  更好
- 把“先读 docs/architecture/repo-layout.md”作为启动清单写进来，能有效减少 Agent 乱改风险
  落后
- 无明显落后

---

10. 其他根文件简评

- biome.json、clippy.toml、deny.toml、typos.toml、rustfmt.toml
    - 领先：质量治理工具链齐
- package-lock.json
    - 落后/噪音：根 packageManager 是 bun，但保留 npm lock，说明 JS 包管理还未完全收口
- .env、.env.example
    - 中性：合理，但根目录环境说明需要更清晰治理
- Cargo.lock
    - 正常
- .tool-versions
    - 更好：提升多工具兼容性

---

二、主目录领域汇总
下面按主目录逐个详细说：职责现状、与文档差距、落后项、领先项、更好项。

1. agent/
   实际内容证据：

- agent/codemap.yml
- agent/constraints/dependencies.yaml
- agent/constraints/patterns.yaml
- agent/constraints/contracts.yaml
- agent/constraints/storage-policy.yaml
- agent/checklists/{release,migration,schema-change,sync-conflict}.md
- agent/prompts/{add-module,add-endpoint,add-sync-strategy,split-service}.md
- agent/templates/module/\*
- agent/templates/bff-endpoint/\*
  领先
- 已经有机器可读约束、模板、checklist、prompt
- dependencies.yaml 正是文档第 7 节强调的那类规则落地
- 模板不仅有 module，还有 bff-endpoint
  落后
- 与文档模板相比，约束文件数量还不全，缺少一些更细粒度项，例如：
    - telemetry-policy.yaml
    - authz-policy.yaml
    - topology-policy.yaml
- 模板体系还不如文档“最终态”那样全面
  更好
- 当前 agent/ 规模已经够支持真实协作，不必为了匹配文档继续机械补齐所有文件名
- 现有模板和约束都围绕当前代码库真实需要，而不是空泛模板

---

2. platform/
   实际内容证据：

- platform/schema/\*.schema.json
- platform/model/services/\*.yaml
- platform/model/deployables/\*.yaml
- platform/model/resources/\*.yaml
- platform/model/workflows/\*.yaml
- platform/model/policies/\*.yaml
- platform/model/topologies/\*.yaml
- platform/model/environments/\*.yaml
- platform/generators/src/main.rs
- platform/validators/\*/src/main.rs
- platform/catalog/_.generated._
  领先
- 这是当前仓库最接近文档最终态的目录之一
- schema/model/generators/validators/catalog 全套都在
- 已生成：
    - services.generated.yaml
    - deployables.generated.yaml
    - resources.generated.yaml
    - topology.generated.md
    - architecture.generated.md
- 校验器也不只是占位，而是每个 validator 都有独立 crate
  落后
- 文档模板里提到的更丰富资源/拓扑组合尚未完全齐，例如：
    - split-edge-workers.yaml
    - k3s-microservices.yaml
    - 更多 resources 如 object-storage、authn-zitadel、authz-openfga
- platform/model/services/ 当前实际服务集与模板不完全一致：
    - 有 agent、chat、event-bus
    - 没有模板里的 indexing
- 说明平台模型已落地，但业务域映射尚未完全稳定
  更好
- 当前 platform/README.md 明确把自己定义为 ARCHITECTURE.md §3.3 的具体实现，这种“文档互相绑定”的做法比模板更成熟
- api-server.yaml 出现在 deployables 中，虽然不属于最终态命名，但对迁移期很实用

---

3. apps/
   实际一级目录：

- apps/web
- apps/desktop
- apps/browser-extension
  整体判断
- web、desktop 已有真实实现
- browser-extension 仍是占位
- mobile 缺失
  3.1 apps/web
  可见证据：
- apps/web/src/routes/...
- apps/web/src/lib/components/ui/\*.svelte
- apps/web/src/lib/ipc/\*.ts
- 页面直接从 $lib/generated/api/\* 引类型
  落后
- 文档要求 apps/_ API 调用应走 packages/sdk/_
- 当前前端类型消费路径是应用内 $lib/generated/api/\*，不是 packages/sdk
- 所以 packages/sdk 还没真正成为前端消费真理源
- 文档建议同步逻辑收口到 lib/sync/，当前 web 侧没有明显看到这条结构作为主要主干
  领先
- Web 前端已明显不是占位，Svelte 页面和 UI 组件存在且规模可用
- 实际页面覆盖 agent/admin/counter/settings/login 等场景
- 已有自定义 UI 组件层
  更好
- 应用内生成代码路径 $lib/generated/api 在开发体验上可能比跨 package 引用更直接
- 如果未来需要 SDK 真理源收敛，可以生成到 packages/sdk 再在 app 侧 re-export，而不是否定当前结构
  3.2 apps/desktop
  证据：
- apps/desktop/src-tauri/src/main.rs
- src/commands/sync.rs
- src/sync/engine.rs
- tests/e2e/tests/specs/\*.spec.ts
  领先
- 桌面壳不只是 Tauri 空项目，已经有命令层、同步层和 e2e 测试
- 比文档“桌面端壳 + 同步”目标落得更具体
  落后
- 文档期望 apps/desktop/src/lib/api/tauri/sync/store 这类前端壳分层，当前主要实现集中在 src-tauri Rust 侧，前端壳结构与模板不完全同构
- 若从跨端统一壳层视角看，仍偏向“桌面能力优先”而非“桌面 app 完整分层优先”
  更好
- 当前以 src-tauri 为主的组织更符合 Tauri 项目真实重心
  3.3 apps/browser-extension
  落后
- 基本占位
- 与文档所强调的多端目标相比明显未启动
  领先
- 仅预留目录
  更好
- 预留目录本身没问题，但现阶段价值有限

---

4. servers/
   实际一级目录：

- api/
- bff/
- gateway/
- indexer/
- realtime/
  整体判断
- 已经开始向文档的 BFF + gateway + worker 形态演进
- 但仍保留 servers/api 这一强中心入口
- 还有旧残留和占位项
  4.1 servers/api
  证据：
- servers/api/openapi.yaml
- servers/api/src/main.rs
- src/routes/{counter,tenant,user,settings,agent,admin}.rs
- src/middleware/tenant.rs
- tests/\*
  领先
- 完整度高，有 OpenAPI、路由、中间件、测试
- 是当前 server 层最成熟的可运行服务之一
  落后
- 从最终架构看，它本身是过渡态角色
- 文档更强调 BFF 聚合入口，而不是单个综合 API server 长期作为主入口
- 它的存在说明同步入口层还没有完全拆分到文档目标形态
  更好
- 在迁移期保留一个完整 api server 作为总入口是非常务实的
- 能保证前后端和桌面端持续可运行，不必等待 BFF 全量完成
  4.2 servers/bff/web-bff 与 servers/bff/admin-bff
  证据：
- src/main.rs
- src/handlers/\*
- src/routes/\*
- src/middleware/tenant.rs
  领先
- BFF 已经不是口号，而是已实现
- 结构基本符合文档对 server 层的期待：handler / routes / middleware / main
  落后
- 文档要求“每个 HTTP server 都有 openapi.yaml”，当前只明确发现 servers/api/openapi.yaml
- web-bff 和 admin-bff 看起来没有各自 openapi.yaml
- 这说明 BFF 已实装，但协议文档还没跟上最终规则
  更好
- 用 servers/bff/ 分组承载多个 BFF，比根下平铺 servers/web-bff、servers/admin-bff 更清晰，也方便未来加 mobile-bff
  4.3 servers/gateway
  证据：
- servers/gateway/src/main.rs
  领先
- 代码上已经有真实实现
- 甚至比 servers/README.md 自己说的 “stub” 更先进
  落后
- 命名上还没收敛成文档里的 edge-gateway
- README 认知与代码现状不一致，属于文档滞后
  更好
- gateway 作为过渡命名也能接受，等边缘职责稳定后再统一命名更稳妥
  4.4 servers/indexer
  落后
- 这个目录很像旧位置残留
- 与文档“所有异步执行单元进入 workers/\*”原则冲突
- 真正的 workers/indexer 已存在
  领先
- 无
  更好
- 无。长期看应清理或明确废弃
  4.5 servers/realtime
  落后
- 占位目录
  更好
- 仅占位，无实质价值

---

5. services/
   实际一级目录：

- admin-service
- agent-service
- auth-service
- chat-service
- counter-service
- event-bus
- settings-service
- tenant-service
- user-service
  整体判断
- 这里是“文档目标最明确，但当前最不均衡”的领域
- 有黄金样板，也有明确未迁完的壳
  5.1 明显较完整
- counter-service
- user-service
- auth-service
- settings-service
  这些目录已有较完整的：
- domain
- application
- ports
- contracts
- events
- tests
- migrations
  领先
- 已经开始接近文档要求的 Clean Architecture service 形态
- counter-service 基本可以视作当前服务层样板
  落后
- 文档要求标准结构中包含 policies/
- 当前不少 service 没有完整显式 policies/
- 仍可见 infrastructure/、interfaces/、sync/ 等过渡目录，和文档标准结构不完全一致
- 命名仍是 \*-service
  更好
- 在迁移期保留 infrastructure/、interfaces/ 有助于分辨旧逻辑与新逻辑边界
- 不需要为了匹配文档立刻删掉这些目录名
  5.2 明显过渡态
- tenant-service
- agent-service
  证据：
- services/tenant-service/README.md 写明业务实现在 packages/core/usecases/tenant_service.rs
- services/agent-service/README.md 写明业务实现在 packages/core/usecases/agent_service.rs
  落后
- 这是当前仓库最明确的“尚未达到文档目标”证据之一
- 文档要求 services/\* 成为业务能力主载体，但这两项还只是迁移壳
  领先
- 至少已经建好目录和迁移目标，不是完全没开始
  更好
- README 明确标注“待迁移”非常好，避免误判完成度
  5.3 特殊项：event-bus
  落后
- 从最终架构角度看，event-bus 更像平台能力或 messaging 能力，不太像业务 service
- 放在 services/ 下并不符合文档最终分层方向
  领先
- 它本身已有 ports、adapters、outbox、tests，成熟度不低
  更好
- 迁移期把事件总线放在 services/ 下未必有害，但最终更适合移动到 packages/messaging 或类似能力层
  5.4 admin-service / chat-service
  落后
- 完整度不如 counter/user/auth/settings
- services/README.md 对其状态描述与目录现状有一定漂移，说明服务层文档本身也有滞后
  领先
- 已有目录与部分实现，不是空白

---

6. workers/
   实际目录：

- indexer
- outbox-relay
- projector
- scheduler
- sync-reconciler
  整体判断
- 这是当前仓库最领先的领域之一
- 每个 worker 都有自己独立 crate 和 main.rs
- 但很多实现仍是“完整骨架 + stub backend”，不是生产级接入
  领先
- 非常符合文档“workers 是一等公民”的思想
- 各 worker 拆分明确：
    - indexer: sources/transforms/sinks/checkpoint
    - outbox-relay: polling/publish/dedupe/checkpoint
    - projector: consumers/readmodels/checkpoint
    - scheduler: jobs/dispatch
    - sync-reconciler: plans/executors/conflict
- 这些目录组织已经和文档最终态高度一致
  落后
- 真实外部系统接入尚未完全落地，README 明确还有 memory/stub backend
- 文档强调 retry/idempotency/checkpoint 策略，当前虽然结构有了，但是否全面实现还不能一概视为完成
  更好
- 先把 worker 拆分和恢复点骨架搭出来，再逐步接入真实后端，是正确顺序
- 比把所有异步逻辑继续塞进 server 更好得多

---

7. packages/
   实际一级目录：

- adapters/
- contracts/
- core/
- features/
- kernel/
- platform/
- runtime/
- sdk/
- shared/
- ui/
  整体判断
- 这是整个仓库与文档差距最大的主目录
- 不是简单“缺少很多目录”，而是当前采用了另一套分层体系，并处于向文档目标收敛的中间阶段
  7.1 符合文档方向、较成熟的部分
- kernel/
- runtime/
- contracts/
- adapters/
- ui/
  领先
- contracts/ 已不只是空目录，有 Rust crate、事件 bindings、结构说明
- runtime/ 已是独立 crate
- adapters/ 覆盖了 host/auth/storage/chains/protocols/telemetry，内容很多
- ui/ 已有共享组件基础
  落后
- 仍未收敛成文档那套能力型包分层：
    - authn/
    - authz/
    - data/
    - messaging/
    - cache/
    - storage/
    - observability/
    - security/
    - web3/
    - wasm/
    - devx/
- 说明“能力存在”与“目录治理收敛”是两回事
  更好
- adapters/ 统一收纳外部系统接入，从当前规模看是合理的
- 不必为了文档模板强行拆成十几个能力包，除非依赖边界已经稳定
  7.2 典型过渡层
- core/
- features/
- shared/
  落后
- 这三块是文档最终架构里最不该长期保留的旧/中过渡层
- 尤其 packages/core/usecases/\* 仍承载真实业务逻辑，直接和文档“services 是业务能力主载体”冲突
- services/README.md 也明确写了“不应继续往 packages/core/usecases/ 加逻辑，历史遗留待清空”
  领先
- features/ 把 trait 能力按特性定义，这在迁移期很有价值
- shared/ 至少把技术工具集中管理，避免散落
  更好
- 在重构期保留 features/ 作为 service 契约桥接层是可以理解的
- 但 core/ 继续承载业务实现，长期不宜
  7.3 sdk/
  落后
- 文档把 packages/sdk/\* 设为前端消费真理源
- 当前 packages/sdk/rust、packages/sdk/typescript 基本还是占位
- 前端实际消费的是 apps/web/src/lib/generated/api/\*
  领先
- 已经把目录预留出来
  更好
- 当前生成到 app 内部更直接；但从长期治理上，文档方向更优

---

8. infra/
   实际一级目录：

- docker/
- gitops/
- k3s/
- kubernetes/
- local/
- security/
- terraform/
  整体判断
- 这里不是落后，而是“实际比文档更复杂”
- 它包含了文档模板的目标项，也保留了现实演进路径
  领先
- 有 gitops/flux
- 有 security/sops
- 有本地、k3s、kubernetes、多种部署面
- 有多份 Dockerfile 和 Compose
- 安全目录还有 policy 文件，不只是 secrets
  落后
- 文档要求 infra/kubernetes/rendered/ 作为模型生成产物，但当前没有看到这一层被真正作为主交付面
- 实际存在：
    - infra/kubernetes/addons/\*
    - infra/k3s/base/\*
    - infra/k3s/overlays/\*
- 说明平台模型驱动到 Kubernetes rendered 的闭环还没有完全成为主路径
- terraform/ 目前更像预留目录
  更好
- 当前 infra/docker、infra/k3s、infra/kubernetes 并存，虽然不如文档“理想纯净”，但现实上更适合从本地到 VPS 到 K8s 的逐步演进
- 比文档模板更贴近“如何真的交付”

---

9. ops/
   实际内容证据：

- ops/migrations/runner/migrate.sh
- ops/runbooks/{README,health-checks,backup-restore}.md
- ops/observability/{vector,otel}/\*
- ops/scripts/bootstrap/vps.sh
  领先
- 已有 runbook、迁移、观测、bootstrap 脚本
- 方向符合文档
  落后
- 文档模板中的 benchmark/、resilience/、backup-restore/verification/ 等更细粒度结构未完整落地
- 仍有 shell 脚本存在，与“尽量统一入口”理想存在张力
  更好
- 真实运维动作先有 README + 脚本 + runbook，比为了模板先造空目录更好

---

10. verification/
    实际一级目录：

- contract/
- e2e/
- golden/
- performance/
- resilience/
- topology/
  领先
- 这是当前仓库另一块非常强的区域
- contract/ 已有：
    - backward compat
    - sdk roundtrip
    - event schema
- resilience/ 已有：
    - retry
    - idempotency
    - outbox
    - failover
- golden/ 已保存平台生成产物基线
- topology/single-vps 已有测试
  落后
- e2e/ 目前更多是 README 场景定义，未像 contract/resilience 那样看到大量执行代码
- performance/ 明显偏占位
- 文档模板要求多拓扑统一验证，目前 single-vps 比较清楚，k3s 方向还不够强
  更好
- 即使还没全覆盖，能先把 contract/resilience/golden 做强，比只追求 UI E2E 数量更健康

---

11. docs/
    实际一级目录：

- adr/
- ARCHITECTURE.md
- architecture/
- contracts/
- generated/
- operations/
- refactoring/
  整体判断
- 文档体系整体是领先的，但存在“局部文档已经落后于代码”的问题
  领先
- adr/ 完整，有 001 到 008
- architecture/ 有 context/container/component/sequence/deployment/topology
- contracts/、operations/、generated/ 都存在
- refactoring/ 很强，包含 roadmap、handoff、progress assessment
  落后
- 文档模板中有 platform-model/ 目录，当前 docs/ 下没有看到同级完整落位
- docs/ARCHITECTURE.md 作为最终态文档，与当前仓库存在明显结构漂移
- servers/README.md 等局部说明已经落后代码现状，例如还说 gateway 是 stub
- 根 README.md 没有承担 docs 入口作用
  更好
- docs/refactoring/ 是模板没有强调、但现实非常有价值的领域文档
- 这说明仓库不是“文档少”，而是“主架构文档需要回收现状”

---

12. fixtures/
    实际一级目录：

- seeds/
- sync-scenarios/
- tenants/
  落后
- 文档要求：
    - users/
    - settings/
    - counter/
    - authz-tuples/
- 当前只有三块，覆盖不足
- 说明 fixtures 层明显还未达到文档目标
  领先
- 至少有 seeds、tenants、sync-scenarios 三个重要起点
  更好
- 无明显“更好”。这块就是还薄

---

13. tools/
    实际一级目录：

- web3/
  落后
- 文档模板期望：
    - web3/
    - codegen/
    - loadtest/
    - diagnostics/
- 当前只落了 web3
  领先
- 无明显领先
  更好
- 当前只保留真实需要的 web3，比到处放空目录更干净；但若以文档目标看，仍属未完成

---

14. 文档外但重要的一级领域
    这部分是你要求里“实际情况和文档差距”的重点，因为它们不在最终态文档主骨架里，但现实中很重要。
    14.1 scripts/
    现状

- doctor.ts
- typegen.ts
- dev-desktop.ts
- boundary-check.ts
- e2e/\*
- deploy/generate-service.sh
- verify-generated.sh
  判断
- 这是实际工程自动化层
- 文档没有把它列为一级主目录，是架构文档的遗漏
- 不应简单视作“脏目录”
  结论
- 更好：对真实仓库是必要补充
- 落后的是文档：文档没有吸收这层现实
  14.2 justfiles/
  现状
- build.just
- clean.just
- deploy.just
- dev.just
- migrate.just
- platform.just
- processes.just
- quality.just
- setup.just
- skills.just
- test.just
  判断
- 这是根命令系统的模块化实现层
- 文档只强调 justfile，没有表达“任务模块拆分”
  结论
- 更好：这是比文档模板更成熟的实现

---

三、哪些落后，哪些领先，哪些更好
这里给你一个集中版清单。
明显落后

- 根 README.md 与仓库实际不符
- bun-workspace.yaml 严重不能反映真实工作区
- apps/mobile/ 缺失
- apps/browser-extension/ 基本占位
- packages/sdk/ 仍是占位，未成为前端真理源
- packages/core/usecases/\* 仍承载真实业务逻辑
- packages/ 未收敛到文档最终能力分层
- tenant-service、agent-service 明确尚未迁移完成
- servers/indexer 仍在错误层级残留
- servers/realtime、可能的 mobile-bff 仍占位
- BFF 未见各自 openapi.yaml
- fixtures/ 覆盖不足
- tools/ 只落了 web3/
- infra/ 尚未完全以 platform -> rendered -> deploy 的模型闭环为主
  明显领先
- platform/ 已形成 schema/model/generators/validators/catalog 全链路
- workers/ 作为一等公民已经实装出完整骨架
- verification/contract、verification/resilience、verification/golden 很强
- docs/adr、docs/architecture、docs/refactoring 非常完整
- agent/ 约束、模板、checklist 已经能支撑真实协作
- apps/desktop 已有同步与 E2E，不只是 Tauri 壳
- servers/gateway 实现度已超过部分自述文档
  实际更好
- Justfile + justfiles/ 比单文件 justfile 更成熟
- servers/bff/ 分组组织比根下平铺更可维护
- infra/docker + k3s + kubernetes + gitops 的并存，现实上比模板更可落地
- scripts/ 作为自动化层是必要补充，文档应吸收而不是否定
- AGENTS.md 作为根入口协作协议，比只在 agent/ 目录里放说明更有效
- 迁移期保留 packages/features 作为桥接层是可以接受的

---

四、最终结论
如果把当前仓库状态一句话说清楚：
当前项目不是“没做到文档”，而是“文档写的是最终态，仓库已经先把平台模型、worker、验证、文档体系做得很先进，但业务层和包分层仍处在从旧架构向最终态收敛的中后期”。
如果再更具体一点：

- 最成熟、最接近文档最终态的领域：platform/、workers/、verification/、agent/、docs/
- 最明显落后、需要继续收敛的领域：services/、packages/、fixtures/、tools/、根层文档治理
- 最需要文档回写现状的地方：根 README.md、bun-workspace.yaml、docs/ARCHITECTURE.md、servers/README.md、services/README.md
  如果你要，我下一步可以继续给你两份补充产物中的任意一份：
