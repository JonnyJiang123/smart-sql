# SQL Favorites API 测试脚本
# 功能验证

# 基础URL
$BASE_URL = "http://localhost:8080/api"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "SQL 收藏夹 API 功能验证" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. 创建收藏
Write-Host "`n[测试1] 创建SQL收藏..." -ForegroundColor Yellow
$createFavoriteBody = @{
    name = "获取所有用户"
    sql_text = "SELECT * FROM users LIMIT 10"
    description = "查询用户表的前10条记录"
    category = "查询示例"
    connection_id = 1
} | ConvertTo-Json

try {
    $response = Invoke-WebRequest -Uri "$BASE_URL/favorites" -Method POST -Body $createFavoriteBody -ContentType "application/json" -ErrorAction Stop
    $result = $response.Content | ConvertFrom-Json
    Write-Host "✓ 创建成功！" -ForegroundColor Green
    Write-Host "响应: $($result | ConvertTo-Json -Depth 2)" -ForegroundColor Gray
    $favoriteId = $result.data.id
} catch {
    Write-Host "✗ 创建失败！" -ForegroundColor Red
    Write-Host "错误: $_" -ForegroundColor Red
}

# 2. 获取所有收藏
Write-Host "`n[测试2] 获取所有收藏..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "$BASE_URL/favorites" -Method GET -ContentType "application/json" -ErrorAction Stop
    $result = $response.Content | ConvertFrom-Json
    Write-Host "✓ 获取成功！" -ForegroundColor Green
    Write-Host "收藏数: $($result.count)" -ForegroundColor Gray
} catch {
    Write-Host "✗ 获取失败！" -ForegroundColor Red
    Write-Host "错误: $_" -ForegroundColor Red
}

# 3. 获取分组列表
Write-Host "`n[测试3] 获取分组列表..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "$BASE_URL/favorites/categories" -Method GET -ContentType "application/json" -ErrorAction Stop
    $result = $response.Content | ConvertFrom-Json
    Write-Host "✓ 获取成功！" -ForegroundColor Green
    Write-Host "分组: $($result.data -join ', ')" -ForegroundColor Gray
} catch {
    Write-Host "✗ 获取失败！" -ForegroundColor Red
    Write-Host "错误: $_" -ForegroundColor Red
}

if ($favoriteId) {
    # 4. 更新收藏
    Write-Host "`n[测试4] 更新收藏..." -ForegroundColor Yellow
    $updateFavoriteBody = @{
        category = "常用查询"
    } | ConvertTo-Json
    
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/favorites/$favoriteId" -Method PUT -Body $updateFavoriteBody -ContentType "application/json" -ErrorAction Stop
        $result = $response.Content | ConvertFrom-Json
        Write-Host "✓ 更新成功！" -ForegroundColor Green
        Write-Host "新分组: $($result.data.category)" -ForegroundColor Gray
    } catch {
        Write-Host "✗ 更新失败！" -ForegroundColor Red
        Write-Host "错误: $_" -ForegroundColor Red
    }
    
    # 5. 增加使用次数
    Write-Host "`n[测试5] 增加使用次数..." -ForegroundColor Yellow
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/favorites/$favoriteId/use" -Method POST -ContentType "application/json" -ErrorAction Stop
        $result = $response.Content | ConvertFrom-Json
        Write-Host "✓ 增加成功！" -ForegroundColor Green
        Write-Host "使用次数: $($result.data.usage_count)" -ForegroundColor Gray
    } catch {
        Write-Host "✗ 增加失败！" -ForegroundColor Red
        Write-Host "错误: $_" -ForegroundColor Red
    }
    
    # 6. 删除收藏
    Write-Host "`n[测试6] 删除收藏..." -ForegroundColor Yellow
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/favorites/$favoriteId" -Method DELETE -ContentType "application/json" -ErrorAction Stop
        $result = $response.Content | ConvertFrom-Json
        Write-Host "✓ 删除成功！" -ForegroundColor Green
    } catch {
        Write-Host "✗ 删除失败！" -ForegroundColor Red
        Write-Host "错误: $_" -ForegroundColor Red
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "API 功能验证完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
