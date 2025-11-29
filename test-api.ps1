# Smart SQL 后端API测试脚本
# 使用方法: .\test-api.ps1

Write-Host "================================" -ForegroundColor Green
Write-Host "Smart SQL 后端API测试" -ForegroundColor Green
Write-Host "================================`n" -ForegroundColor Green

$baseUrl = "http://localhost:8080"

# 1. 健康检查
Write-Host "1. 健康检查..." -ForegroundColor Cyan
try {
    $health = Invoke-RestMethod -Uri "$baseUrl/health" -Method Get
    Write-Host "✅ 健康检查成功!" -ForegroundColor Green
    $health | ConvertTo-Json
} catch {
    Write-Host "❌ 健康检查失败: $_" -ForegroundColor Red
    exit 1
}

# 2. 获取连接列表
Write-Host "`n2. 获取连接列表..." -ForegroundColor Cyan
try {
    $connections = Invoke-RestMethod -Uri "$baseUrl/api/connections" -Method Get
    Write-Host "✅ 连接列表获取成功! 共 $($connections.Count) 个连接" -ForegroundColor Green
    $connections | ConvertTo-Json -Depth 5
} catch {
    Write-Host "❌ 获取连接列表失败: $_" -ForegroundColor Red
}

# 3. 创建新连接
Write-Host "`n3. 创建新连接 (MySQL示例)..." -ForegroundColor Cyan
$newConnection = @{
    name = "测试MySQL-" + (Get-Date -Format "HHmmss")
    db_type = "mysql"
    host = "localhost"
    port = 3306
    database_name = "test"
    username = "root"
    password = "password"
} | ConvertTo-Json

try {
    $created = Invoke-RestMethod -Uri "$baseUrl/api/connections" -Method Post -Body $newConnection -ContentType "application/json"
    Write-Host "✅ 连接创建成功! ID: $($created.id)" -ForegroundColor Green
    $connectionId = $created.id
    $created | ConvertTo-Json -Depth 5
} catch {
    Write-Host "⚠️  连接创建失败 (可能已存在): $_" -ForegroundColor Yellow
    $connectionId = 1 # 使用已有连接
}

# 4. 测试连接
Write-Host "`n4. 测试数据库连接 (SQLite)..." -ForegroundColor Cyan
$testConnection = @{
    db_type = "sqlite"
    file_path = ":memory:"
} | ConvertTo-Json

try {
    $testResult = Invoke-RestMethod -Uri "$baseUrl/api/connections/test" -Method Post -Body $testConnection -ContentType "application/json"
    if ($testResult.success) {
        Write-Host "✅ 连接测试成功! 响应时间: $($testResult.response_time_ms)ms" -ForegroundColor Green
    } else {
        Write-Host "❌ 连接测试失败: $($testResult.message)" -ForegroundColor Red
    }
    $testResult | ConvertTo-Json
} catch {
    Write-Host "❌ 测试连接失败: $_" -ForegroundColor Red
}

# 5. 更新连接
if ($connectionId) {
    Write-Host "`n5. 更新连接 (ID=$connectionId)..." -ForegroundColor Cyan
    $updateData = @{
        name = "更新后的连接-" + (Get-Date -Format "HHmmss")
        db_type = "mysql"
        host = "localhost"
        port = 3307
        database_name = "test_updated"
        username = "admin"
    } | ConvertTo-Json

    try {
        $updated = Invoke-RestMethod -Uri "$baseUrl/api/connections/$connectionId" -Method Put -Body $updateData -ContentType "application/json"
        Write-Host "✅ 连接更新成功!" -ForegroundColor Green
        $updated | ConvertTo-Json -Depth 5
    } catch {
        Write-Host "❌ 更新连接失败: $_" -ForegroundColor Red
    }
}

# 6. 获取单个连接
if ($connectionId) {
    Write-Host "`n6. 获取单个连接 (ID=$connectionId)..." -ForegroundColor Cyan
    try {
        $conn = Invoke-RestMethod -Uri "$baseUrl/api/connections/$connectionId" -Method Get
        Write-Host "✅ 获取连接成功!" -ForegroundColor Green
        $conn | ConvertTo-Json -Depth 5
    } catch {
        Write-Host "❌ 获取连接失败: $_" -ForegroundColor Red
    }
}

# 7. 删除连接
if ($connectionId -and $connectionId -gt 1) {
    Write-Host "`n7. 删除连接 (ID=$connectionId)..." -ForegroundColor Cyan
    try {
        Invoke-RestMethod -Uri "$baseUrl/api/connections/$connectionId" -Method Delete
        Write-Host "✅ 连接删除成功!" -ForegroundColor Green
    } catch {
        Write-Host "❌ 删除连接失败: $_" -ForegroundColor Red
    }
}

# 最终结果
Write-Host "`n================================" -ForegroundColor Green
Write-Host "测试完成!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green

Write-Host "`n最终连接列表:" -ForegroundColor Cyan
try {
    $finalConnections = Invoke-RestMethod -Uri "$baseUrl/api/connections" -Method Get
    Write-Host "共 $($finalConnections.Count) 个连接" -ForegroundColor Green
    $finalConnections | ConvertTo-Json -Depth 5
} catch {
    Write-Host "❌ 获取最终连接列表失败: $_" -ForegroundColor Red
}
