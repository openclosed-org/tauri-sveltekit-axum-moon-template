# Process management — cross-platform (Windows/macOS/Linux)

# 查看本项目相关进程状态
ps:
    @echo "=== 项目进程状态 ==="
    @echo ""
    @echo "--- API Server (port 3001) ---"
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 3001 -ErrorAction SilentlyContinue | Select-Object OwningProcess | Where-Object OwningProcess -gt 0 | ForEach-Object { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,StartTime }" 2>/dev/null || echo "  [STOPPED]"; \
    else \
        lsof -i :3001 2>/dev/null | grep LISTEN || echo "  [STOPPED]"; \
    fi
    @echo ""
    @echo "--- Vite Dev Server (port 5173) ---"
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 5173 -ErrorAction SilentlyContinue | Select-Object OwningProcess | Where-Object OwningProcess -gt 0 | ForEach-Object { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,StartTime }" 2>/dev/null || echo "  [STOPPED]"; \
    else \
        lsof -i :5173 2>/dev/null | grep LISTEN || echo "  [STOPPED]"; \
    fi
    @echo ""
    @echo "--- Tauri Desktop App ---"
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-Process -Name 'native-tauri' -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,StartTime" 2>/dev/null || echo "  [STOPPED]"; \
    else \
        pgrep -la "native-tauri" 2>/dev/null || echo "  [STOPPED]"; \
    fi
    @echo ""
    @echo "--- 孤儿/无响应进程 ---"
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.Responding -eq $false } | Select-Object Id,ProcessName,Responding" 2>/dev/null || echo "  [NONE]"; \
    else \
        ps aux | grep -E "Z|defunct" | grep -v grep || echo "  [NONE]"; \
    fi

# 停止所有本项目启动的开发进程
stop:
    @echo "正在停止所有开发进程..."
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.CommandLine -match 'cargo run' -or $_.ProcessName -match 'runtime_server' } | Stop-Process -Force -ErrorAction SilentlyContinue" 2>/dev/null; echo "  [API server stopped]"; \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.CommandLine -match 'cargo tauri dev' } | Stop-Process -Force -ErrorAction SilentlyContinue" 2>/dev/null; echo "  [Tauri dev stopped]"; \
        powershell -NoProfile -Command "Get-Process -Name 'native-tauri' -ErrorAction SilentlyContinue | Stop-Process -Force" 2>/dev/null; echo "  [native-tauri stopped]"; \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.CommandLine -match 'vite dev' -or $_.CommandLine -match 'bun run dev' } | Stop-Process -Force -ErrorAction SilentlyContinue" 2>/dev/null; echo "  [Vite dev stopped]"; \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.CommandLine -match 'moon run repo:dev' } | Stop-Process -Force -ErrorAction SilentlyContinue" 2>/dev/null; echo "  [moon tasks stopped]"; \
    else \
        pkill -f "cargo run -p runtime_server" 2>/dev/null && echo "  ✓ API server stopped" || echo "  - API server not running"; \
        pkill -f "runtime_server" 2>/dev/null && echo "  ✓ runtime_server stopped" || true; \
        pkill -f "cargo tauri dev" 2>/dev/null && echo "  ✓ Tauri dev stopped" || echo "  - Tauri dev not running"; \
        pkill -f "native-tauri" 2>/dev/null && echo "  ✓ native-tauri stopped" || echo "  - native-tauri not running"; \
        pkill -f "cd ../web/app && bun run dev" 2>/dev/null && echo "  ✓ Vite (Tauri child) stopped" || true; \
        pkill -f "cd apps/client/web/app && bun run dev" 2>/dev/null && echo "  ✓ Vite (standalone) stopped" || true; \
        pkill -f "vite dev" 2>/dev/null && echo "  ✓ vite dev stopped" || true; \
        pkill -f "moon run repo:dev" 2>/dev/null && echo "  ✓ moon tasks stopped" || true; \
    fi
    @echo "清理完成"

# 停止指定端口的进程
stop-port PORT='':
    @test -n "{{PORT}}" || (echo "Usage: just stop-port PORT=3001" && exit 1)
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "$conn = Get-NetTCPConnection -LocalPort {{PORT}} -ErrorAction SilentlyContinue; if ($conn) { Stop-Process -Id $conn.OwningProcess -Force; echo '  ✓ Port {{PORT}} freed' } else { echo '  - Nothing on port {{PORT}}' }"; \
    else \
        lsof -ti :{{PORT}} 2>/dev/null | xargs kill -9 2>/dev/null && echo "  ✓ Port {{PORT}} freed" || echo "  - Nothing on port {{PORT}}"; \
    fi

# 清理所有孤儿/无响应进程（当前用户的）
clean-orphans:
    @echo "正在清理当前用户的孤儿/无响应进程..."
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-Process | Where-Object { $_.Responding -eq $false -and $_.SessionId -eq (Get-Process -Id $PID).SessionId } | Stop-Process -Force -ErrorAction SilentlyContinue; echo '  ✓ Non-responding processes cleaned'"; \
    else \
        ps aux | awk '$8 ~ /Z/ {print $2}' | xargs -r kill -9 2>/dev/null && echo "  ✓ Zombie processes cleaned" || echo "  - No zombie processes found"; \
    fi
    @echo "完成"

# 显示端口占用详情
ports:
    @echo "=== 项目端口占用 ==="
    @echo ""
    @echo "Port 3001 (API):"
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 3001 -ErrorAction SilentlyContinue | ForEach-Object { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue | Select-Object Id,ProcessName }" 2>/dev/null || echo "  [FREE]"; \
    else \
        lsof -i :3001 2>/dev/null | grep -v COMMAND || echo "  [FREE]"; \
    fi
    @echo ""
    @echo "Port 5173 (Vite):"
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 5173 -ErrorAction SilentlyContinue | ForEach-Object { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue | Select-Object Id,ProcessName }" 2>/dev/null || echo "  [FREE]"; \
    else \
        lsof -i :5173 2>/dev/null | grep -v COMMAND || echo "  [FREE]"; \
    fi
    @echo ""
    @echo "Port 1420 (Tauri fallback):"
    @if uname -s | grep -qiE 'mingw|msys|cygwin'; then \
        powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue | ForEach-Object { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue | Select-Object Id,ProcessName }" 2>/dev/null || echo "  [FREE]"; \
    else \
        lsof -i :1420 2>/dev/null | grep -v COMMAND || echo "  [FREE]"; \
    fi
