# Process management — cross-platform (Windows/macOS/Linux)
# Uses just's built-in os_family() for reliable platform detection

# 查看本项目相关进程状态
ps:
    @echo "=== 项目进程状态 ==="
    @echo ""
    @echo "--- Web BFF Server (port 3010) ---"
    @if [ "{{os_family()}}" = "windows" ]; then \
        powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 3010 -ErrorAction SilentlyContinue | Select-Object OwningProcess | Where-Object OwningProcess -gt 0 | ForEach-Object { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,StartTime }" 2>/dev/null || echo "  [STOPPED]"; \
    else \
        lsof -i :3010 2>/dev/null | grep LISTEN || echo "  [STOPPED]"; \
    fi
    @echo ""
    @echo "--- 孤儿/无响应进程 ---"
    @if [ "{{os_family()}}" = "windows" ]; then \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.Responding -eq $false } | Select-Object Id,ProcessName,Responding" 2>/dev/null || echo "  [NONE]"; \
    else \
        ps -axo user=,pid=,stat=,start=,time=,command= | awk '$3 ~ /^Z/ { print }' || echo "  [NONE]"; \
    fi

# 停止所有本项目启动的开发进程
stop:
    @echo "正在停止所有开发进程..."
    @if [ "{{os_family()}}" = "windows" ]; then \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.CommandLine -match 'cargo run' -or $_.ProcessName -match 'web-bff' } | Stop-Process -Force -ErrorAction SilentlyContinue" 2>/dev/null; echo "  [BFF servers stopped]"; \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.CommandLine -match 'moon run repo:dev' } | Stop-Process -Force -ErrorAction SilentlyContinue" 2>/dev/null; echo "  [moon tasks stopped]"; \
    else \
        pkill -f "cargo run -p web-bff" 2>/dev/null && echo "  ✓ Web BFF stopped" || echo "  - Web BFF not running"; \
        pkill -f "web-bff" 2>/dev/null && echo "  ✓ web-bff stopped" || true; \
        pkill -f "moon run repo:dev" 2>/dev/null && echo "  ✓ moon tasks stopped" || true; \
    fi
    @echo "清理完成"

# 停止指定端口的进程
stop-port PORT='':
    @test -n "{{PORT}}" || (echo "Usage: just stop-port 3010" && exit 1)
    @if [ "{{os_family()}}" = "windows" ]; then \
        powershell -NoProfile -Command "$conn = Get-NetTCPConnection -LocalPort {{PORT}} -ErrorAction SilentlyContinue; if ($conn) { Stop-Process -Id $conn.OwningProcess -Force; echo '  ✓ Port {{PORT}} freed' } else { echo '  - Nothing on port {{PORT}}' }"; \
    else \
        PIDS=$(lsof -ti :{{PORT}} 2>/dev/null || true); \
        if [ -n "$PIDS" ]; then \
            kill -9 $PIDS 2>/dev/null && echo "  ✓ Port {{PORT}} freed" || echo "  - Failed to free port {{PORT}}"; \
        else \
            echo "  - Nothing on port {{PORT}}"; \
        fi; \
    fi

# 清理所有孤儿/无响应进程（当前用户的）
clean-orphans:
    @echo "正在清理当前用户的孤儿/无响应进程..."
    @if [ "{{os_family()}}" = "windows" ]; then \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.Responding -eq $false -and $_.SessionId -eq (Get-Process -Id $PID).SessionId } | Stop-Process -Force -ErrorAction SilentlyContinue; echo '  ✓ Non-responding processes cleaned'"; \
    else \
        PIDS=$(ps -axo pid=,stat= | awk '$2 ~ /^Z/ {print $1}'); \
        if [ -n "$PIDS" ]; then \
            kill -9 $PIDS 2>/dev/null && echo "  ✓ Zombie processes cleaned" || echo "  - Failed to clean zombie processes"; \
        else \
            echo "  - No zombie processes found"; \
        fi; \
    fi
    @echo "完成"

# 显示端口占用详情
ports:
    @echo "=== 项目端口占用 ==="
    @echo ""
    @echo "Port 3010 (Web BFF):"
    @if [ "{{os_family()}}" = "windows" ]; then \
        powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 3010 -ErrorAction SilentlyContinue | ForEach-Object { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue | Select-Object Id,ProcessName }" 2>/dev/null || echo "  [FREE]"; \
    else \
        lsof -i :3010 2>/dev/null | grep -v COMMAND || echo "  [FREE]"; \
    fi

# ── Worker Management ─────────────────────────────────────────

# 启动所有 workers（后台运行）
dev-workers:
    @echo "Starting workers..."
    @echo "  - outbox-relay-worker (health: :3030)"
    @echo "  - indexer-worker (health: :3031)"
    @echo "  - projector-worker (health: :3032)"
    @echo "  - scheduler-worker (health: :3033)"
    @echo "  - sync-reconciler-worker (health: :3034)"
    @echo ""
    @echo "Workers run in background. Use 'just stop-workers' to stop."
    cargo run -p outbox-relay-worker &
    cargo run -p indexer-worker &
    cargo run -p projector-worker &
    cargo run -p scheduler-worker &
    cargo run -p sync-reconciler-worker &
    @echo "All workers started."

# 停止所有 workers
stop-workers:
    @echo "Stopping workers..."
    @pkill -f "outbox-relay-worker" 2>/dev/null && echo "  ✓ outbox-relay-worker stopped" || echo "  - outbox-relay-worker not running"
    @pkill -f "indexer-worker" 2>/dev/null && echo "  ✓ indexer-worker stopped" || echo "  - indexer-worker not running"
    @pkill -f "projector-worker" 2>/dev/null && echo "  ✓ projector-worker stopped" || echo "  - projector-worker not running"
    @pkill -f "scheduler-worker" 2>/dev/null && echo "  ✓ scheduler-worker stopped" || echo "  - scheduler-worker not running"
    @pkill -f "sync-reconciler-worker" 2>/dev/null && echo "  ✓ sync-reconciler-worker stopped" || echo "  - sync-reconciler-worker not running"
    @echo "Done."

# 启动单个 worker
run-worker WORKER='':
    @test -n "{{WORKER}}" || (echo "Usage: just run-worker WORKER=outbox-relay" && exit 1)
    @echo "Starting {{WORKER}}-worker..."
    cargo run -p {{WORKER}}-worker

# 查看 workers 健康状态
health-workers:
    @echo "=== Worker Health ==="
    @echo ""
    @echo "outbox-relay-worker (:3030):"
    @curl -s http://localhost:3030/healthz 2>/dev/null || echo "  [UNREACHABLE]"
    @echo ""
    @echo "indexer-worker (:3031):"
    @curl -s http://localhost:3031/healthz 2>/dev/null || echo "  [UNREACHABLE]"
    @echo ""
    @echo "projector-worker (:3032):"
    @curl -s http://localhost:3032/healthz 2>/dev/null || echo "  [UNREACHABLE]"
    @echo ""
    @echo "scheduler-worker (:3033):"
    @curl -s http://localhost:3033/healthz 2>/dev/null || echo "  [UNREACHABLE]"
    @echo ""
    @echo "sync-reconciler-worker (:3034):"
    @curl -s http://localhost:3034/healthz 2>/dev/null || echo "  [UNREACHABLE]"
    @echo ""
