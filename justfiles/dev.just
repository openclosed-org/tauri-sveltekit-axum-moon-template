# Development — all dev server commands

# 启动全栈开发（web-bff + admin-bff + web）
dev:
    moon run repo:dev-fullstack

# 启动 Web 开发
dev-web:
    moon run repo:dev-web

# 启动 API 开发（web-bff）
dev-api:
    moon run repo:dev-api

# 启动 Admin BFF 开发
dev-admin-bff:
    moon run repo:dev-admin-bff

# 启动桌面应用开发
dev-desktop:
    moon run repo:dev-desktop

# 启动 Tauri 桌面应用（手动调试用）
dev-tauri:
    cd apps/client/native/src-tauri && cargo tauri dev
