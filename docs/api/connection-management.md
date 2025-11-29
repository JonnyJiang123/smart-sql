# 数据库连接管理 API 文档

## 概述

数据库连接管理功能允许用户添加、测试、保存和激活多个数据库连接。所有连接配置都保存在本地 SQLite 数据库中。

## API 端点

### 1. 获取所有连接

**请求**
```
GET /api/connections
```

**响应**
```json
[
  {
    "id": 1,
    "name": "本地MySQL",
    "db_type": "mysql",
    "host": "localhost",
    "port": 3306,
    "database_name": "test_db",
    "username": "root",
    "password": null,
    "is_active": true,
    "created_at": "2025-01-15T10:00:00Z",
    "updated_at": "2025-01-15T10:00:00Z"
  }
]
```

### 2. 创建新连接

**请求**
```
POST /api/connections
Content-Type: application/json

{
  "name": "测试数据库",
  "db_type": "mysql",
  "host": "localhost",
  "port": 3306,
  "database_name": "test_db",
  "username": "root",
  "password": "password123"
}
```

**支持的数据库类型**
- `mysql` - MySQL数据库
- `postgresql` - PostgreSQL数据库
- `sqlite` - SQLite数据库

**SQLite 示例**
```json
{
  "name": "本地SQLite",
  "db_type": "sqlite",
  "file_path": "./data/test.db"
}
```

**响应**
```json
{
  "id": 2,
  "name": "测试数据库",
  "db_type": "mysql",
  "host": "localhost",
  "port": 3306,
  "database_name": "test_db",
  "username": "root",
  "is_active": false,
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z"
}
```

### 3. 测试数据库连接

**请求**
```
POST /api/connections/test
Content-Type: application/json

{
  "db_type": "mysql",
  "host": "localhost",
  "port": 3306,
  "database_name": "test_db",
  "username": "root",
  "password": "password123"
}
```

**响应 - 成功**
```json
{
  "success": true,
  "message": "连接成功",
  "server_version": "8.0.32",
  "response_time_ms": 125
}
```

**响应 - 失败**
```json
{
  "success": false,
  "message": "连接失败: Connection refused",
  "server_version": null,
  "response_time_ms": 5000
}
```

### 4. 激活连接（并获取数据库信息）

当用户双击连接列表中的某个连接时，调用此API激活连接并获取数据库信息。

**请求**
```
POST /api/connections/{id}/activate
```

**响应 - 成功**
```json
{
  "success": true,
  "message": "连接激活成功",
  "database_info": {
    "database_type": "MySQL",
    "server_version": "8.0.32-MySQL Community Server",
    "tables": [
      "users",
      "orders",
      "products",
      "categories"
    ],
    "total_tables": 4
  }
}
```

**响应 - 失败**
```json
{
  "success": false,
  "message": "连接失败: Access denied for user 'root'@'localhost'",
  "database_info": null
}
```

### 5. 更新连接配置

**请求**
```
PUT /api/connections/{id}
Content-Type: application/json

{
  "name": "生产MySQL",
  "host": "prod.example.com",
  "port": 3306,
  "database_name": "prod_db",
  "username": "admin",
  "password": "newpassword"
}
```

**响应**
```json
{
  "id": 1,
  "name": "生产MySQL",
  "db_type": "mysql",
  "host": "prod.example.com",
  "port": 3306,
  "database_name": "prod_db",
  "username": "admin",
  "is_active": false,
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2025-01-15T11:00:00Z"
}
```

### 6. 删除连接

**请求**
```
DELETE /api/connections/{id}
```

**响应**
```
204 No Content
```

## 前端使用流程

### 新增连接流程

1. **用户点击"新增连接"按钮**
   - 弹出连接配置表单

2. **填写表单**
   ```typescript
   interface ConnectionForm {
     name: string;
     db_type: 'mysql' | 'postgresql' | 'sqlite';
     host?: string;          // MySQL/PostgreSQL必填
     port?: number;          // MySQL/PostgreSQL必填
     database_name?: string; // MySQL/PostgreSQL必填
     username?: string;      // MySQL/PostgreSQL必填
     password?: string;      // MySQL/PostgreSQL可选
     file_path?: string;     // SQLite必填
   }
   ```

3. **点击"测试连接"按钮**
   ```typescript
   async function testConnection(form: ConnectionForm) {
     setLoading(true);
     try {
       const response = await fetch('/api/connections/test', {
         method: 'POST',
         headers: { 'Content-Type': 'application/json' },
         body: JSON.stringify(form)
       });
       const result = await response.json();
       
       if (result.success) {
         showMessage('连接成功！', 'success');
       } else {
         showMessage(`连接失败：${result.message}`, 'error');
       }
     } finally {
       setLoading(false);
     }
   }
   ```

4. **点击"保存"按钮**
   ```typescript
   async function saveConnection(form: ConnectionForm) {
     const response = await fetch('/api/connections', {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify(form)
     });
     const newConnection = await response.json();
     
     // 更新连接列表
     setConnections([...connections, newConnection]);
     closeDialog();
     showMessage('连接已保存', 'success');
   }
   ```

### 激活连接流程

1. **用户双击连接列表中的某个连接**
   ```typescript
   async function activateConnection(connectionId: number) {
     setLoading(true);
     try {
       const response = await fetch(`/api/connections/${connectionId}/activate`, {
         method: 'POST'
       });
       const result = await response.json();
       
       if (result.success && result.database_info) {
         // 更新数据库树形结构
         setDatabaseInfo(result.database_info);
         setTables(result.database_info.tables);
         setActiveConnectionId(connectionId);
         
         showMessage(`已连接到 ${result.database_info.database_type}`, 'success');
       } else {
         showMessage(`连接失败：${result.message}`, 'error');
       }
     } finally {
       setLoading(false);
     }
   }
   ```

2. **显示数据库信息**
   - 在数据库树中显示所有表
   - 显示数据库类型和版本
   - 更新连接状态指示器

## 错误处理

### 常见错误码

- `400 Bad Request` - 请求参数错误
  - 缺少必要字段
  - 数据库类型不支持
  - 连接配置不完整

- `404 Not Found` - 连接不存在

- `500 Internal Server Error` - 服务器内部错误
  - 数据库操作失败
  - 连接池创建失败

### 错误响应格式

```json
{
  "error": "error_code",
  "message": "详细错误信息",
  "details": "可选的额外信息"
}
```

## 安全考虑

1. **密码加密存储** - 数据库密码在本地 SQLite 中加密存储
2. **连接字符串安全** - 不在日志中记录包含密码的连接字符串
3. **超时控制** - 连接测试有 5 秒超时限制
4. **错误信息脱敏** - 不向前端返回详细的数据库错误堆栈

## 性能优化

1. **连接池复用** - 激活的连接使用连接池，避免频繁创建连接
2. **异步操作** - 所有数据库操作都是异步的，不阻塞主线程
3. **懒加载** - 只在用户激活连接时才建立实际连接

## 测试示例

### 使用 curl 测试

```bash
# 1. 创建连接
curl -X POST http://localhost:8080/api/connections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "测试MySQL",
    "db_type": "mysql",
    "host": "localhost",
    "port": 3306,
    "database_name": "test_db",
    "username": "root",
    "password": "password"
  }'

# 2. 测试连接
curl -X POST http://localhost:8080/api/connections/test \
  -H "Content-Type: application/json" \
  -d '{
    "db_type": "mysql",
    "host": "localhost",
    "port": 3306,
    "database_name": "test_db",
    "username": "root",
    "password": "password"
  }'

# 3. 获取所有连接
curl http://localhost:8080/api/connections

# 4. 激活连接
curl -X POST http://localhost:8080/api/connections/1/activate

# 5. 删除连接
curl -X DELETE http://localhost:8080/api/connections/1
```

## 更新日志

- **2025-01-15**: 
  - 实现完整的连接管理 API
  - 添加激活连接并获取数据库信息功能
  - 支持 MySQL、PostgreSQL、SQLite 三种数据库
  - 添加连接测试功能
