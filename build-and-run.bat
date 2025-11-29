@echo off
setlocal enabledelayedexpansion

echo =======================================
echo 智能SQLer - 一键编译启动脚本
echo =======================================

:: 1. 清理旧构建
echo [1/5] 清理旧构建...
if exist "frontend\dist" rmdir /s /q "frontend\dist"
if exist "backend\target" rmdir /s /q "backend\target"
if exist "frontend\src-tauri\target" rmdir /s /q "frontend\src-tauri\target"
echo 清理完成

:: 2. 编译前端
echo [2/5] 编译前端...
cd frontend
call npm run build
if %errorlevel% neq 0 (
    echo 前端编译失败！
    pause
    exit /b 1
)
echo 前端编译成功

:: 3. 编译后端
echo [3/5] 编译后端...
cd ..\backend
call cargo build --release
if %errorlevel% neq 0 (
    echo 后端编译失败！
    pause
    exit /b 1
)
echo 后端编译成功

:: 4. 构建Tauri桌面应用
echo [4/5] 构建Tauri桌面应用...
cd ..\frontend
call npm run tauri build
if %errorlevel% neq 0 (
    echo Tauri构建失败！
    pause
    exit /b 1
)
echo Tauri构建成功

:: 5. 启动应用
echo [5/5] 启动应用...
set EXE_PATH=src-tauri\target\release\smart-sql.exe
if exist "!EXE_PATH!" (
    echo 启动应用：!EXE_PATH!
    start "" "!EXE_PATH!"
) else (
    echo 可执行文件不存在：!EXE_PATH!
    pause
    exit /b 1
)

echo =======================================
echo 脚本执行完成！
echo =======================================
pause
