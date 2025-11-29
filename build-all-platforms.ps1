<#
.SYNOPSIS
智能SQLer - Windows上的跨平台打包脚本

.DESCRIPTION
在Windows环境下一键打包智能SQLer为所有平台的可执行文件，包含所有依赖
支持：Windows、macOS Intel、macOS ARM、Linux x86_64、Linux ARM64

.EXAMPLE
.uild-all-platforms.ps1
#>

# 强制设置终端编码为UTF-8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8

Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "智能SQLer - 跨平台打包脚本" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan

# 检查依赖
Write-Host "[1/6] 检查依赖..." -ForegroundColor Yellow

# 检查Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "错误：未安装Rust！请先安装Rust 1.91或更高版本。" -ForegroundColor Red
    exit 1
}

# 检查Node.js
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "错误：未安装Node.js！请先安装Node.js 16.x或更高版本。" -ForegroundColor Red
    exit 1
}

# 检查npm
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Host "错误：未安装npm！请先安装npm 8.x或更高版本。" -ForegroundColor Red
    exit 1
}

Write-Host "依赖检查通过！" -ForegroundColor Green

# 定义目标平台
$targetPlatforms = @(
    @{ Name = "Windows x86_64"; Target = "x86_64-pc-windows-msvc"; Extension = ".exe" }
    @{ Name = "macOS Intel"; Target = "x86_64-apple-darwin"; Extension = "" }
    @{ Name = "macOS ARM"; Target = "aarch64-apple-darwin"; Extension = "" }
    @{ Name = "Linux x86_64"; Target = "x86_64-unknown-linux-gnu"; Extension = "" }
    @{ Name = "Linux ARM64"; Target = "aarch64-unknown-linux-gnu"; Extension = "" }
)

# 清理旧构建
Write-Host "\n[2/6] 清理旧构建..." -ForegroundColor Yellow
if (Test-Path "frontend\dist") {
    Remove-Item -Recurse -Force "frontend\dist"
    Write-Host "已清理前端构建目录" -ForegroundColor Green
}
if (Test-Path "frontend\src-tauri\target") {
    Remove-Item -Recurse -Force "frontend\src-tauri\target"
    Write-Host "已清理Tauri构建目录" -ForegroundColor Green
}

# 编译前端
Write-Host "\n[3/6] 编译前端..." -ForegroundColor Yellow
Set-Location "frontend"
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "前端编译失败！" -ForegroundColor Red
    Set-Location ".."
    exit 1
}
Write-Host "前端编译成功！" -ForegroundColor Green

# 安装Tauri CLI（如果未安装）
Write-Host "\n[4/6] 检查Tauri CLI..." -ForegroundColor Yellow
if (-not (Get-Command tauri -ErrorAction SilentlyContinue)) {
    Write-Host "Tauri CLI未安装，正在安装..." -ForegroundColor Yellow
    npm install -g @tauri-apps/cli
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Tauri CLI安装失败！" -ForegroundColor Red
        Set-Location ".."
        exit 1
    }
}
Write-Host "Tauri CLI已就绪！" -ForegroundColor Green

# 构建所有平台
Write-Host "\n[5/6] 构建所有平台..." -ForegroundColor Yellow

$buildResults = @()

foreach ($platform in $targetPlatforms) {
    Write-Host "\n正在构建 $($platform.Name)..." -ForegroundColor Cyan
    
    try {
        # 构建命令
        $buildCommand = "npm run tauri build -- --target $($platform.Target) --verbose"
        Invoke-Expression $buildCommand
        
        if ($LASTEXITCODE -eq 0) {
            # 构建成功，记录结果
            $outputDir = "src-tauri\target\$($platform.Target)\release"
            $executablePath = "$outputDir\smart-sql$($platform.Extension)"
            
            $buildResults += @{
                Platform = $platform.Name
                Target = $platform.Target
                Status = "成功"
                ExecutablePath = $executablePath
                OutputDir = $outputDir
            }
            
            Write-Host "✅ $($platform.Name) 构建成功！" -ForegroundColor Green
            Write-Host "   可执行文件：$executablePath" -ForegroundColor Green
        } else {
            # 构建失败
            $buildResults += @{
                Platform = $platform.Name
                Target = $platform.Target
                Status = "失败"
                ExecutablePath = ""
                OutputDir = ""
            }
            
            Write-Host "❌ $($platform.Name) 构建失败！" -ForegroundColor Red
        }
    } catch {
        # 异常处理
        $buildResults += @{
            Platform = $platform.Name
            Target = $platform.Target
            Status = "异常"
            ExecutablePath = ""
            OutputDir = ""
        }
        
        Write-Host "❌ $($platform.Name) 构建异常：$($_.Exception.Message)" -ForegroundColor Red
    }
}

# 返回项目根目录
Set-Location ".."

# 显示构建结果汇总
Write-Host "\n[6/6] 构建结果汇总：" -ForegroundColor Yellow
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "平台名称           | 状态 | 目标三元组                  | 可执行文件位置" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan

foreach ($result in $buildResults) {
    $statusColor = if ($result.Status -eq "成功") { "Green" } else { "Red" }
    $executablePath = if ($result.ExecutablePath) { $result.ExecutablePath } else { "-" }
    
    Write-Host "$($result.Platform.PadRight(17)) | $($result.Status.PadRight(4)) | $($result.Target.PadRight(26)) | $executablePath" -ForegroundColor $statusColor
}

Write-Host "=======================================" -ForegroundColor Cyan

# 统计结果
$successCount = ($buildResults | Where-Object { $_.Status -eq "成功" }).Count
$totalCount = $buildResults.Count

Write-Host "\n构建完成！" -ForegroundColor Green
Write-Host "成功：$successCount/$totalCount 个平台" -ForegroundColor Green

if ($successCount -eq $totalCount) {
    Write-Host "✅ 所有平台构建成功！" -ForegroundColor Green
} else {
    Write-Host "⚠️  部分平台构建失败，请检查日志。" -ForegroundColor Yellow
}

Write-Host "\n可执行文件已包含所有依赖，可以直接在目标平台运行。" -ForegroundColor Green
Write-Host "=======================================" -ForegroundColor Cyan
