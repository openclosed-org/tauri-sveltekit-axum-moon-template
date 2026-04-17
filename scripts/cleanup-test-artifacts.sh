#!/usr/bin/env bash
# 截断 .tmp 中超过 10MB 的日志文件，保留最后 5 万行
# 用于防止测试日志无限增长
set -euo pipefail

MAX_SIZE=$((10 * 1024 * 1024))  # 10MB
KEEP_LINES=50000

find .tmp -name "*.log" -type f 2>/dev/null | while read -r f; do
  size=$(stat -f%z "$f" 2>/dev/null || stat -c%s "$f" 2>/dev/null || echo 0)
  if [ "$size" -gt "$MAX_SIZE" ]; then
    echo "  截断 $f (当前 ${size} bytes)"
    tail -n "$KEEP_LINES" "$f" > "${f}.tmp" && mv "${f}.tmp" "$f"
  fi
done
