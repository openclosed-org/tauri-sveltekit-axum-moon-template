@echo off
REM ============================================
REM Visual Studio Build Tools 环境配置脚本
REM 用于配置 MSVC 工具链到 PATH
REM ============================================

echo === 配置 Visual Studio Build Tools 环境 ===

REM 设置 Build Tools 路径
set "VS_BUILDTOOLS_PATH=D:\dev-storage\Microsoft Visual Studio\18\BuildTools"

REM 检查 vcvarsall.bat 是否存在
if exist "%VS_BUILDTOOLS_PATH%\VC\Auxiliary\Build\vcvarsall.bat" (
    echo ✅ 找到 vcvarsall.bat
) else (
    echo ❌ 未找到 vcvarsall.bat
    echo 路径: %VS_BUILDTOOLS_PATH%\VC\Auxiliary\Build\vcvarsall.bat
    pause
    exit /b 1
)

REM 调用 vcvarsall.bat 配置 x64 环境
echo 📦 正在配置 x64 编译环境...
call "%VS_BUILDTOOLS_PATH%\VC\Auxiliary\Build\vcvarsall.bat" x64

echo ✅ MSVC 工具链已配置完成
echo 📍 工具链路径: %VS_BUILDTOOLS_PATH%
echo 📍 MSVC 版本: 14.50.35717

REM 验证 cl.exe 是否可用
where cl.exe >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ cl.exe 已在 PATH 中
    cl.exe /Bv 2>&1 | findstr "Version"
) else (
    echo ⚠️  cl.exe 未在 PATH 中
)

REM 验证 link.exe 是否可用
where link.exe >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ link.exe 已在 PATH 中
) else (
    echo ⚠️  link.exe 未在 PATH 中
)

echo.
echo === 环境配置完成 ===
echo 💡 提示: 此脚本会为当前终端会话配置环境
echo    关闭终端后需要重新运行此脚本
echo.

REM 启动一个新的子 shell（保持环境）
cmd /k "echo 已配置 MSVC 环境。可以运行 bun、cargo、wdio 等命令。"
