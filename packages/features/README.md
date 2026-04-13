# Features

> Feature trait 定义层。每个 crate 定义一个业务能力的接口。

## 当前结构

```
features/
├── admin/      # AdminService trait（仪表盘统计）
├── agent/      # AgentService trait（对话管理、消息发送、工具调用）
├── auth/       # AuthService trait（登录、回调、会话管理）
├── chat/       # ChatService trait（聊天消息）
├── counter/    # CounterService trait（计数器 CRUD）
└── settings/   # SettingsService trait（配置读写）
```

## 已清理项

以下 feature stub 已删除（零依赖、无代码）：
- `feed/`
- `notifications/`
- `payments/`
- `profile/`
- `social-graph/`
- `wallet/`

## 规则

**不新增**新的 `features/*` 除非：
1. 至少 1 个 service 实际实现该 trait
2. 该 trait 被 2+ 个消费方（server/bff/adapter）使用
