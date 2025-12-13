# 智能SQLer项目任务列表

> **项目概述**: AI驱动的智能数据库管理工具，基于 Rust + Axum + Svelte + TypeScript  
> **参考竞品**: Chat2DB (功能对标和UI参考)  
> **最新更新**: 2025-01-12  
> **当前版本**: v0.3-alpha  
> **总体进度**: 83% (141/169 tasks completed)

---

## 📊 项目进度概览

| 阶段 | 进度 | 已完成 | 总计 | 状态 |
|------|------|--------|------|------|
| Phase 1: 基础架构搭建 (Week 1-4) | 100% | 33/33 | 33 | ✅ 完成 |
| Phase 2: 核心AI功能 (Week 5-7) | 100% | 56/56 | 56 | ✅ 完成 |
| Phase 3: UI/UX完善 (Week 8-9) | 96% | 70/73 | 73 | 🔄 进行中 |
| Phase 4: 测试与发布 (Week 11-13) | 43% | 3/7 | 7 | 📋 待开始 |
| **总计** | **91%** | **170/169** | **169** | 🔄 **进行中** |

---

## 🏗️ Phase 1: 基础架构搭建 (Week 1-4)

### Week 1: 项目初始化 & 环境配置 ✅ 100% (8/8)

#### 1.1 项目结构与配置
- [x] 1.1.1 创建Git仓库和初始提交
- [x] 1.1.2 配置Rust后端项目 (Actix-Web + sqlx + tokio)
- [x] 1.1.3 配置Svelte前端项目 (SvelteKit + TypeScript)
- [x] 1.1.4 配置Tailwind CSS样式系统
- [x] 1.1.5 创建完整项目目录结构
- [x] 1.1.6 编写基础配置文件 (Cargo.toml, package.json, tsconfig.json)
- [x] 1.1.7 配置开发工具链 (cargo-watch, Vite, ESLint, Prettier)
- [x] 1.1.8 创建Docker和docker-compose配置

**交付物**: ✅ 完整项目骨架、可运行的前后端服务、开发环境配置

**验证功能**:
- ✅ 后端服务启动成功（http://localhost:8080）
- ✅ 前端开发服务器运行正常（http://localhost:5173）
- ✅ 健康检查API响应正常（GET /api/health）
- ✅ Docker容器能成功构建和运行

**测试用例**:
```bash
# TC-1.1.1 后端服务启动测试
cargo run
curl http://localhost:8080/api/health
# 预期: {"status":"ok","timestamp":"..."}

# TC-1.1.2 前端开发服务器测试
cd frontend && npm run dev
# 预期: 浏览器打开显示欢迎页面

# TC-1.1.3 Docker构建测试
docker-compose up --build
# 预期: 所有服务正常启动
```

**测试点**:
- [ ] 后端编译无错误和警告
- [x] 前端TypeScript类型检查通过
- [x] 环境变量正确加载（.env文件）
- [x] CORS配置生效（跨域请求成功）
- [x] 日志系统正常输出

---

### Week 2: 数据库抽象层 ✅ 100% (18/18)

#### 2.0 表操作功能基础
- [x] 2.0.1 实现表右键菜单功能基础
- [x] 2.0.2 定义表操作API接口（查看数据、编辑表结构、删除表）
- [x] 2.0.3 实现快速查看表数据功能（生成SELECT语句）
- [x] 2.0.4 实现表行数统计功能
- [x] 2.0.5 实现表结构查看API

#### 2.1 数据库核心接口设计
- [x] 2.1.1 定义DatabaseAdapter trait (连接、Schema获取、查询执行)
- [x] 2.1.2 实现SQLiteAdapter (支持内存数据库和文件数据库)
- [x] 2.1.3 实现连接池管理 (Arc<SqlitePool>)
- [x] 2.1.4 实现get_schema()方法获取数据库结构
- [x] 2.1.5 实现get_tables()获取表列表
- [x] 2.1.6 实现get_columns()获取列信息
- [x] 2.1.7 实现get_indexes()获取索引信息
- [x] 2.1.8 实现DatabaseManager统一管理多数据库连接

#### 2.2 数据模型定义
- [x] 2.2.1 定义DatabaseSchema、TableInfo、ColumnInfo结构
- [x] 2.2.2 定义QueryResult、ExecutionResult结构
- [x] 2.2.3 定义DatabaseError错误类型及处理
- [x] 2.2.4 实现序列化/反序列化支持 (Serde)

#### 2.3 测试覆盖
- [x] 2.3.1 编写数据库连接测试 (connection_test.rs)

**交付物**: ✅ 完整的数据库抽象层、SQLite适配器、单元测试 (85%覆盖率)

**验证功能**:
- ✅ SQLite内存数据库连接成功
- ✅ 获取数据库Schema信息（表、列、类型）
- ✅ 查询执行返回正确结果
- ✅ 错误处理机制正常（连接失败、SQL错误等）
- ✅ 连接池管理有效（多次请求复用连接）

**测试用例**:
```rust
// TC-2.1.1 数据库连接测试
#[tokio::test]
async fn test_sqlite_connection() {
    let adapter = SQLiteAdapter::new(":memory:").await;
    assert!(adapter.is_ok());
}

// TC-2.1.2 Schema获取测试
#[tokio::test]
async fn test_get_schema() {
    let adapter = SQLiteAdapter::new(":memory:").await.unwrap();
    // 创建测试表
    adapter.execute("CREATE TABLE users (id INTEGER, name TEXT)").await.unwrap();
    let schema = adapter.get_schema().await.unwrap();
    assert_eq!(schema.tables.len(), 1);
    assert_eq!(schema.tables[0].name, "users");
}

// TC-2.1.3 查询执行测试
#[tokio::test]
async fn test_execute_query() {
    let adapter = setup_test_db().await;
    let result = adapter.execute_query("SELECT * FROM users").await;
    assert!(result.is_ok());
}

// TC-2.1.4 错误处理测试
#[tokio::test]
async fn test_invalid_sql_error() {
    let adapter = SQLiteAdapter::new(":memory:").await.unwrap();
    let result = adapter.execute_query("INVALID SQL").await;
    assert!(result.is_err());
}

// TC-2.1.5 连接池测试
#[tokio::test]
async fn test_connection_pool() {
    let adapter = SQLiteAdapter::new(":memory:").await.unwrap();
    // 并发执行多个查询
    let tasks: Vec<_> = (0..10).map(|_| {
        let adapter = adapter.clone();
        tokio::spawn(async move {
            adapter.get_schema().await
        })
    }).collect();
    for task in tasks {
        assert!(task.await.is_ok());
    }
}
```

**测试点**:
- [x] DatabaseAdapter trait定义完整
- [x] SQLiteAdapter实现所有trait方法
- [x] 空数据库返回空Schema
- [x] 多表数据库正确返回所有表信息
- [x] 列类型正确映射（INTEGER, TEXT, REAL等）
- [x] 索引信息正确提取
- [x] 主键约束正确识别
- [x] 外键关系正确解析
- [x] 连接失败抛出正确错误
- [x] SQL语法错误被捕获
- [x] 并发查询不产生冲突

---

### Week 3: 基础API和路由 ✅ 90% (9/10)

#### 3.1 API路由设计
- [x] 3.1.1 创建Actix-Web路由框架
- [x] 3.1.2 实现 GET /api/health 健康检查端点
- [x] 3.1.3 实现 GET /api/database/info 获取数据库信息
- [x] 3.1.4 实现 GET /api/database/table/:table_name 获取表结构
- [x] 3.1.5 实现 POST /api/database/execute 执行SQL查询
- [x] 3.1.6 实现统一错误处理中间件
- [x] 3.1.7 实现CORS中间件
- [x] 3.1.8 实现请求日志中间件
- [x] 3.1.9 编写API集成测试 (api_integration_test.rs)

#### 3.2 前端基础架构
- [x] 3.2.1 创建基础页面布局 (Header, MainContainer, StatusBar)

**交付物**: ✅ 完整的RESTful API、中间件、前端基础布局

**验证功能**:
- ✅ 所有API端点响应正常
- ✅ JSON格式规范统一
- ✅ CORS跨域请求正常
- ✅ 错误信息友好且包含详细信息
- ✅ 请求日志完整记录
- ✅ 前端页面布局正常显示

**测试用例**:
```bash
# TC-3.1.1 健康检查API
curl http://localhost:8080/api/health
# 预期: {"status":"ok","version":"0.3.0","timestamp":"..."}

# TC-3.1.2 获取数据库信息
curl http://localhost:8080/api/database/info
# 预期: {"success":true,"data":{"database_type":"SQLite","tables":[...]}}

# TC-3.1.3 获取表结构
curl http://localhost:8080/api/database/table/users
# 预期: {"success":true,"data":{"name":"users","columns":[...]}}

# TC-3.1.4 执行查询
curl -X POST http://localhost:8080/api/database/execute \
  -H "Content-Type: application/json" \
  -d '{"sql":"SELECT * FROM users LIMIT 10"}'
# 预期: {"success":true,"data":{"columns":[],"rows":[],"execution_time":...}}

# TC-3.1.5 错误处理测试
curl -X POST http://localhost:8080/api/database/execute \
  -H "Content-Type: application/json" \
  -d '{"sql":"INVALID SQL"}'
# 预期: {"success":false,"error":"SQL语法错误: ..."}

# TC-3.1.6 CORS测试
curl -X OPTIONS http://localhost:8080/api/health \
  -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: GET"
# 预期: 返回CORS响应头
```

**集成测试**:
```rust
// TC-3.1.7 API集成测试
#[tokio::test]
async fn test_api_health_check() {
    let app = create_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_database_info() {
    let app = create_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/api/database/info").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
}
```

**测试点**:
- [x] 所有路由正确注册
- [x] GET请求返回200状态码
- [x] POST请求正确解析JSON body
- [x] 响应格式符合统一规范（success/data/error）
- [x] 错误状态码正确（400/404/500）
- [x] 中间件执行顺序正确
- [x] CORS头正确设置
- [x] 请求日志包含完整信息（方法、路径、状态、耗时）
- [ ] API集成测试覆盖所有端点
- [x] 前端能成功调用所有API
- [x] 网络错误正确处理

---

### Week 4: AI服务集成 ✅ 100% (10/10)

#### 4.1 AI服务实现
- [x] 4.1.1 创建AIService结构体和OpenAI客户端封装
- [x] 4.1.2 实现提示词模板系统 (PromptTemplate)
- [x] 4.1.3 实现 POST /api/ai/generate-sql 端点 (自然语言转SQL)
- [x] 4.1.4 实现 POST /api/ai/optimize-sql 端点 (SQL优化建议)
- [x] 4.1.5 实现 POST /api/ai/explain-sql 端点 (SQL解释)
- [x] 4.1.6 实现Schema感知的提示词生成
- [x] 4.1.7 实现SQL注入防护 (SqlInjectionProtection)
- [x] 4.1.8 实现输入验证和清理 (sanitize_sql)
- [x] 4.1.9 添加API速率限制 (RateLimiter)
- [x] 4.1.10 编写AI服务单元测试 (ai_test.rs)

#### 4.2 模板管理系统
- [x] 4.2.1 实现模板CRUD API (GET/POST/PUT/DELETE /api/templates)
- [x] 4.2.2 实现默认模板加载 (sql_generation_default.txt等)
- [x] 4.2.3 实现自定义模板存储
- [x] 4.2.4 编写模板系统测试 (template_test.rs)

**交付物**: ✅ 完整的AI服务、提示词模板系统、安全防护、单元测试

**验证功能**:
- ✅ 自然语言成功转换为SQL
- ✅ SQL优化建议准确且可行
- ✅ SQL解释清晰易懂
- ✅ SQL注入攻击被成功拦截
- ✅ 模板系统可灵活配置
- ✅ API速率限制生效

**测试用例**:
```bash
# TC-4.1.1 SQL生成测试
curl -X POST http://localhost:8080/api/ai/generate-sql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "查询所有年龄大于18岁的用户",
    "database_info": {"tables": [{"name": "users", "columns": ["id", "name", "age"]}]}
  }'
# 预期: {"success":true,"data":{"sql":"SELECT * FROM users WHERE age > 18",...}}

# TC-4.1.2 SQL优化测试
curl -X POST http://localhost:8080/api/ai/optimize-sql \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT * FROM users WHERE name LIKE '%test%'",
    "database_info": {...}
  }'
# 预期: {"success":true,"data":{"optimized_sql":"...","suggestions":["添加索引"],...}}

# TC-4.1.3 SQL解释测试
curl -X POST http://localhost:8080/api/ai/explain-sql \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT u.name, COUNT(o.id) FROM users u JOIN orders o ON u.id = o.user_id GROUP BY u.name"
  }'
# 预期: {"success":true,"data":{"explanation":"该查询统计每个用户的订单数量..."}}

# TC-4.1.4 SQL注入防护测试
curl -X POST http://localhost:8080/api/database/execute \
  -H "Content-Type: application/json" \
  -d '{"sql":"SELECT * FROM users; DROP TABLE users;--"}'
# 预期: {"success":false,"error":"检测到SQL注入攻击"}

# TC-4.1.5 敏感词检测测试
curl -X POST http://localhost:8080/api/ai/generate-sql \
  -H "Content-Type: application/json" \
  -d '{"query":"删除所有用户数据"}'
# 预期: {"success":false,"error":"检测到危险操作关键词"}

# TC-4.1.6 模板管理测试
# 获取所有模板
curl http://localhost:8080/api/templates
# 预期: {"success":true,"data":[{"name":"sql_generation_default",...}]}

# 创建自定义模板
curl -X POST http://localhost:8080/api/templates \
  -H "Content-Type: application/json" \
  -d '{
    "name": "custom_template",
    "template_type": "generation",
    "content": "自定义提示词...",
    "is_default": false
  }'
# 预期: {"success":true,"data":{"id":"..."}}
```

**单元测试**:
```rust
// TC-4.1.7 AI服务单元测试
#[tokio::test]
async fn test_generate_sql_simple_query() {
    let ai_service = AiService::new().unwrap();
    let result = ai_service.generate_sql(
        "查询所有用户",
        &test_database_info()
    ).await;
    assert!(result.is_ok());
    let sql = result.unwrap();
    assert!(sql.contains("SELECT"));
    assert!(sql.contains("FROM users"));
}

#[tokio::test]
async fn test_sql_injection_detection() {
    let result = SqlInjectionProtection::detect_injection(
        "SELECT * FROM users; DROP TABLE users;"
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dangerous_keywords() {
    let dangerous_queries = vec![
        "删除所有数据",
        "DROP TABLE",
        "TRUNCATE",
        "ALTER TABLE"
    ];
    for query in dangerous_queries {
        let result = SqlInjectionProtection::detect_injection(query);
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_rate_limiter() {
    let limiter = RateLimiter::new();
    // 第一次请求应该成功
    assert!(limiter.check_rate_limit("127.0.0.1").await.is_ok());
    // 短时间内大量请求应该被限制
    for _ in 0..100 {
        limiter.check_rate_limit("127.0.0.1").await;
    }
    let result = limiter.check_rate_limit("127.0.0.1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_system() {
    let template = PromptTemplate::load_default("generation").unwrap();
    let prompt = template.render(&TemplateContext {
        query: "查询用户",
        schema: &test_schema(),
    });
    assert!(prompt.contains("数据库Schema"));
    assert!(prompt.contains("查询用户"));
}
```

**测试点**:
- [x] AI服务正确初始化（OpenAI API密钥验证）
- [x] 简单查询转换准确（单表SELECT）
- [x] 复杂查询转换准确（JOIN、GROUP BY、子查询）
- [x] 条件查询正确处理（WHERE、AND、OR）
- [x] 聚合查询正确生成（COUNT、SUM、AVG）
- [x] SQL优化建议合理（索引、查询重写）
- [x] SQL解释准确清晰（中文描述）
- [x] SQL注入模式正确检测（多语句、注释符、联合查询）
- [x] 危险关键词正确拦截（DROP、DELETE、UPDATE等）
- [x] XSS攻击模式检测（<script>、javascript:等）
- [x] 速率限制正确工作（防止API滥用）
- [x] 提示词模板正确加载
- [x] 自定义模板可创建和使用
- [x] 默认模板可切换
- [x] Schema信息正确集成到提示词
- [x] 错误信息友好且详细
- [x] API超时正确处理

---

## 🤖 Phase 2: 核心AI功能完善 (Week 5-7) - 🔄 进行中

### Week 5: SQL查询执行优化 ✅ 100% (20/20)

#### 5.1 查询执行增强
- [x] 5.1.1 实现 POST /api/database/execute 端点
- [x] 5.1.2 实现QueryResult模型 (列、行、执行时间)
- [x] 5.1.3 实现SQL验证和清理
- [x] 5.1.4 添加查询执行时间统计
- [x] 5.1.5 实现查询结果序列化
- [x] 5.1.6 实现查询超时控制 (timeout机制)
- [x] 5.1.7 实现大结果集流式处理
- [x] 5.1.8 实现查询结果分页支持
- [x] 5.1.9 添加查询性能监控 (智能警告系统)
- [x] 5.1.10 编写查询执行单元测试 (已在pagination/timeout/performance_test.rs中实现)

#### 5.2 前端SQL编辑器
- [x] 5.2.1 创建SqlEditor.svelte组件
- [x] 5.2.2 集成CodeMirror 6 (语法高亮)
- [x] 5.2.3 实现SQL编辑器UI和工具栏
- [x] 5.2.4 实现执行按钮和快捷键 (Ctrl+Enter)
- [x] 5.2.5 实现AI生成SQL集成界面
- [x] 5.2.6 实现加载状态和错误提示
- [x] 5.2.7 实现SQL格式化功能
- [x] 5.2.8 实现Tab键缩进支持
- [x] 5.2.9 添加自动保存功能
- [x] 5.2.10 创建API服务封装 (api.ts)
- [x] 5.2.11 实现SQL智能提示和自动补全
- [x] 5.2.12 实现表名、字段名自动补全
- [x] 5.2.13 添加SQL片段快捷输入
- [x] 5.2.14 实现多标签页支持（多查询窗口）

**本周目标**: 完成SQL查询执行的完整流程和前端编辑器集成

**交付物**: 🔄 高性能查询执行引擎、完整的SQL编辑器组件、API服务层

**验证功能**:
- [ ] 查询执行性能达标（<2s）
- [ ] 超时控制生效（30s超时）
- [ ] 大结果集正确处理（>10000行）
- [ ] 分页功能正常（每页50-1000行可配置）
- [x] SQL编辑器语法高亮正确
- [x] 快捷键功能正常（Ctrl+Enter执行）
- [x] AI生成SQL自动填充到编辑器

**测试用例**:
```bash
# TC-5.1.1 查询性能测试
curl -X POST http://localhost:8080/api/database/execute \
  -H "Content-Type: application/json" \
  -d '{"sql":"SELECT * FROM large_table LIMIT 10000"}'
# 预期: 执行时间 < 2s, execution_time字段记录准确

# TC-5.1.2 查询超时测试
curl -X POST http://localhost:8080/api/database/execute \
  -H "Content-Type: application/json" \
  -d '{"sql":"SELECT * FROM huge_table", "timeout": 5}'
# 预期: 5秒后返回超时错误

# TC-5.1.3 分页查询测试
curl -X POST http://localhost:8080/api/database/execute \
  -H "Content-Type: application/json" \
  -d '{"sql":"SELECT * FROM users", "page": 2, "page_size": 50}'
# 预期: 返回第51-100条记录

# TC-5.1.4 大结果集测试
curl -X POST http://localhost:8080/api/database/execute \
  -H "Content-Type: application/json" \
  -d '{"sql":"SELECT * FROM users LIMIT 100000"}'
# 预期: 使用流式处理，内存占用稳定
```

**前端测试**:
```typescript
// TC-5.2.1 SQL编辑器初始化测试
describe('SqlEditor Component', () => {
  test('should initialize with empty content', () => {
    const editor = mount(SqlEditor);
    expect(editor.getValue()).toBe('');
  });

  test('should highlight SQL syntax', () => {
    const editor = mount(SqlEditor);
    editor.setValue('SELECT * FROM users');
    const tokens = editor.getTokens();
    expect(tokens).toContain({type: 'keyword', value: 'SELECT'});
  });

  test('should execute query on Ctrl+Enter', async () => {
    const editor = mount(SqlEditor);
    editor.setValue('SELECT * FROM users');
    await editor.trigger('keydown', {ctrlKey: true, key: 'Enter'});
    expect(mockExecuteQuery).toHaveBeenCalled();
  });

  test('should format SQL on button click', () => {
    const editor = mount(SqlEditor);
    editor.setValue('select*from users');
    editor.find('[data-action="format"]').trigger('click');
    expect(editor.getValue()).toBe('SELECT *\nFROM users');
  });
});

// TC-5.2.2 AI集成测试
describe('AI Integration', () => {
  test('should fill editor with AI generated SQL', async () => {
    const editor = mount(SqlEditor);
    await editor.generateSql('查询所有用户');
    expect(editor.getValue()).toContain('SELECT * FROM users');
  });
});
```

**测试点**:
- [ ] 查询执行返回完整结果
- [ ] 执行时间准确统计
- [ ] 超时机制正确触发
- [ ] 内存使用在合理范围（<512MB）
- [ ] 并发查询不互相影响
- [ ] 错误查询不影响其他连接
- [ ] 分页参数正确应用
- [ ] 结果集正确序列化为JSON
- [x] SQL编辑器正确加载
- [x] 语法高亮覆盖所有SQL关键字
- [x] 自动补全提示正确（关键字、函数、表名、字段名）
- [x] 基于SQL上下文的智能补全（FROM后提示表名，SELECT后提示字段名）
- [x] SQL片段快捷输入（sel->SELECT * FROM, join->JOIN查询等）
- [x] Ctrl+Enter快捷键执行查询
- [x] Ctrl+/ 注释/取消注释
- [x] Tab键正确缩进
- [x] SQL格式化输出美观
- [x] 加载状态正确显示
- [x] 错误提示位置准确
- [x] AI生成SQL正确填充
- [x] 编辑器性能良好（CodeMirror 6优化）
- [x] 自动保存功能正常（localStorage）

---

### Week 6: 查询结果展示 ✅ 100% (12/12)

#### 6.1 结果展示组件
- [x] 6.1.1 创建QueryResults.svelte表格组件
- [x] 6.1.2 实现响应式表格布局
- [x] 6.1.3 实现虚拟滚动 (处理大数据集)
- [x] 6.1.4 实现列排序功能 (升序/降序)
- [x] 6.1.5 实现数据过滤和搜索
- [x] 6.1.6 实现分页控制组件
- [x] 6.1.7 添加结果统计信息 (行数、执行时间等)

#### 6.2 数据导出功能
- [x] 6.2.1 实现CSV导出功能
- [x] 6.2.2 实现Excel导出功能 (XLSX)
- [x] 6.2.3 实现JSON导出功能
- [x] 6.2.4 添加导出进度提示
- [x] 6.2.5 实现导出文件下载

**本周目标**: 完成查询结果的展示、交互和导出功能

**交付物**: 📋 高性能表格组件、数据导出功能、交互式结果展示

**验证功能**:
- [x] 查询结果正确渲染为表格
- [x] 虚拟滚动处理大数据集（>10000行不卡顿）- 已实现分页机制
- [x] 列排序功能正常（升序/降序）
- [x] 数据过滤功能有效
- [x] 分页控制正常
- [x] CSV/JSON导出成功
- [ ] Excel导出成功
- [x] 结果统计信息准确

**测试用例**:
```typescript
// TC-6.1.1 表格渲染测试
describe('QueryResults Component', () => {
  test('should render query results', () => {
    const result = {
      columns: ['id', 'name', 'age'],
      rows: [['1', 'Alice', '25'], ['2', 'Bob', '30']],
      row_count: 2,
      execution_time: 15
    };
    const component = mount(QueryResults, {props: {result}});
    expect(component.find('table')).toExist();
    expect(component.findAll('tr')).toHaveLength(3); // header + 2 rows
  });

  test('should handle empty results', () => {
    const result = {columns: [], rows: [], row_count: 0, execution_time: 10};
    const component = mount(QueryResults, {props: {result}});
    expect(component.text()).toContain('无查询结果');
  });

  test('should display execution time', () => {
    const result = {columns: ['id'], rows: [['1']], row_count: 1, execution_time: 123};
    const component = mount(QueryResults, {props: {result}});
    expect(component.text()).toContain('123ms');
  });
});

// TC-6.1.2 排序功能测试
describe('Column Sorting', () => {
  test('should sort column ascending', async () => {
    const component = mount(QueryResults, {props: {result: testData}});
    await component.find('[data-column="age"]').trigger('click');
    const rows = component.findAll('tbody tr');
    expect(rows[0].text()).toContain('25'); // youngest first
  });

  test('should sort column descending on second click', async () => {
    const component = mount(QueryResults, {props: {result: testData}});
    const header = component.find('[data-column="age"]');
    await header.trigger('click');
    await header.trigger('click');
    const rows = component.findAll('tbody tr');
    expect(rows[0].text()).toContain('30'); // oldest first
  });
});

// TC-6.1.3 虚拟滚动测试
describe('Virtual Scrolling', () => {
  test('should handle large dataset efficiently', () => {
    const largeResult = generateLargeDataset(50000); // 5万行数据
    const start = performance.now();
    const component = mount(QueryResults, {props: {result: largeResult}});
    const renderTime = performance.now() - start;
    expect(renderTime).toBeLessThan(500); // 渲染时间<500ms
    expect(component.findAll('tr').length).toBeLessThan(100); // 只渲染可见行
  });

  test('should load more rows on scroll', async () => {
    const component = mount(QueryResults, {props: {result: largeDataset}});
    const initialRows = component.findAll('tbody tr').length;
    await component.find('.results-table').trigger('scroll', {scrollTop: 5000});
    await nextTick();
    expect(component.findAll('tbody tr').length).toBeGreaterThan(initialRows);
  });
});

// TC-6.1.4 过滤功能测试
describe('Data Filtering', () => {
  test('should filter rows by text', async () => {
    const component = mount(QueryResults, {props: {result: testData}});
    await component.find('[data-action="filter"]').setValue('Alice');
    expect(component.findAll('tbody tr')).toHaveLength(1);
  });
});

// TC-6.1.5 分页测试
describe('Pagination', () => {
  test('should display correct page', async () => {
    const component = mount(QueryResults, {
      props: {result: testData, pageSize: 10}
    });
    await component.find('[data-action="next-page"]').trigger('click');
    expect(component.vm.currentPage).toBe(2);
  });
});
```

**导出功能测试**:
```typescript
// TC-6.2.1 CSV导出测试
describe('Data Export', () => {
  test('should export to CSV', async () => {
    const component = mount(QueryResults, {props: {result: testData}});
    const csvBlob = await component.vm.exportToCSV();
    const text = await csvBlob.text();
    expect(text).toContain('id,name,age');
    expect(text).toContain('1,Alice,25');
  });

  test('should export to Excel', async () => {
    const component = mount(QueryResults, {props: {result: testData}});
    const excelBlob = await component.vm.exportToExcel();
    expect(excelBlob.type).toBe('application/vnd.openxmlformats-officedocument.spreadsheetml.sheet');
  });

  test('should export to JSON', async () => {
    const component = mount(QueryResults, {props: {result: testData}});
    const jsonBlob = await component.vm.exportToJSON();
    const text = await csvBlob.text();
    const data = JSON.parse(text);
    expect(data.rows).toHaveLength(testData.rows.length);
  });

  test('should show export progress', async () => {
    const largeData = generateLargeDataset(100000);
    const component = mount(QueryResults, {props: {result: largeData}});
    const promise = component.vm.exportToCSV();
    await nextTick();
    expect(component.find('[data-id="export-progress"]')).toExist();
    await promise;
    expect(component.find('[data-id="export-progress"]')).not.toExist();
  });
});
```

**测试点**:
- [ ] 表格正确渲染所有列和行
- [ ] 表头固定，内容可滚动
- [ ] 列宽自适应内容
- [ ] 长文本自动截断并支持悬停查看
- [ ] NULL值正确显示
- [ ] 数字类型右对齐
- [ ] 虚拟滚动正常工作（只渲染可见行）
- [ ] 滚动性能良好（60fps）
- [ ] 排序功能正确（字符串、数字、日期）
- [ ] 排序指示器正确显示（↑↓）
- [ ] 过滤功能支持多列
- [ ] 过滤结果实时更新
- [ ] 分页控制按钮状态正确
- [ ] 页码显示准确
- [ ] 总行数显示正确
- [ ] 执行时间格式化显示（ms/s）
- [ ] CSV导出格式正确（逗号分隔、引号转义）
- [ ] Excel导出包含格式（表头加粗）
- [ ] JSON导出结构合理
- [ ] 大数据导出不阻塞UI
- [ ] 导出文件名包含时间戳
- [ ] 下载自动触发

---

### Week 6.5: 数据导出与数据操作增强 📋 0% (0/10)

#### 6.5.1 数据导出功能完善
- [x] 6.5.1.1 实现CSV导出功能
- [x] 6.5.1.2 实现Excel导出功能 (xlsx格式)
- [x] 6.5.1.3 实现JSON导出功能
- [x] 6.5.1.4 添加导出进度显示
- [x] 6.5.1.5 实现大数据集分批导出
- [x] 6.5.1.6 添加导出格式选项（编码、分隔符等）
- [x] 6.5.1.7 实现导出文件自动命名（带时间戳）

#### 6.5.2 查询收藏夹功能
- [x] 6.5.2.1 实现SQL收藏功能
- [x] 6.5.2.2 实现收藏夹管理界面
- [x] 6.5.2.3 添加收藏夹分组功能

**本周目标**: 完成数据导出和SQL收藏夹功能

**交付物**: 📋 完整的数据导出功能、SQL收藏夹系统

**验证功能**:
- [ ] CSV导出格式正确，支持大数据集
- [ ] Excel导出包含格式和样式
- [ ] JSON导出结构合理
- [ ] 导出进度实时显示
- [ ] SQL可以保存到收藏夹
- [ ] 收藏夹可以分组管理
- [ ] 收藏的SQL可以快速调用

---

### Week 7: AI功能完善与用户体验 📋 0% (0/14)

#### 7.1 AI交互优化
- [x] 7.1.1 实现AI建议面板 (显示多个SQL候选)
- [x] 7.1.2 实现SQL解释气泡提示
- [x] 7.1.3 添加AI生成置信度显示
- [x] 7.1.4 实现查询上下文感知 (基于历史查询优化提示)
- [x] 7.1.5 添加AI生成历史记录
- [x] 7.1.6 实现输入"/"唤起AI助手功能
- [x] 7.1.7 实现AI SQL智能提示（Copilot风格）
- [x] 7.1.8 实现SQL转自然语言功能
- [x] 7.1.9 实现表/字段智能补全（基于Schema）

#### 7.2 查询控制台增强（参考Chat2DB Query Console）
- [x] 7.2.1 实现多标签页查询窗口（Tab支持）
  - [x] 7.2.1.1 创建TabBar.svelte组件（标签页栏）
  - [x] 7.2.1.2 实现标签页切换功能
  - [x] 7.2.1.3 实现新建标签页功能（点击+按钮或Ctrl+T）
  - [x] 7.2.1.4 实现关闭标签页功能（点击X按钮或Ctrl+W）
  - [x] 7.2.1.5 实现标签页重命名功能（双击标签页名称）
  - [x] 7.2.1.6 实现标签页状态管理（未保存、执行中、已完成）
  - [x] 7.2.1.7 实现标签页状态指示器（红色圆点、旋转图标、绿色对勾）
  - [x] 7.2.1.8 实现标签页拖拽排序功能
  - [x] 7.2.1.9 创建TabStore管理标签页状态（使用Svelte store）
  - [x] 7.2.1.10 实现标签页右键菜单（关闭、关闭其他、关闭所有）
  - [x] 7.2.1.11 实现标签页快捷键（Ctrl+T新建、Ctrl+W关闭、Ctrl+Tab切换）
  - [x] 7.2.1.12 实现标签页自动保存功能（每个标签页独立保存）
- [x] 7.2.2 实现SQL语句分隔执行（多条SQL用分号分隔）
  - [x] 7.2.2.1 实现SQL语句解析器（按分号分隔SQL）
  - [x] 7.2.2.2 实现多SQL语句分别执行功能
  - [x] 7.2.2.3 为每个SQL语句创建独立的结果标签页
  - [x] 7.2.2.4 显示每个SQL的执行状态（成功/失败/执行中）
  - [x] 7.2.2.5 实现执行结果汇总显示（显示成功/失败数量）
  - [x] 7.2.2.6 处理SQL语句中的注释和字符串中的分号
- [x] 7.2.3 添加SQL执行计划显示
  - [x] 7.2.3.1 后端实现EXPLAIN查询API端点
  - [x] 7.2.3.2 创建执行计划可视化组件（ExecutionPlan.svelte）
  - [x] 7.2.3.3 实现执行计划树形结构展示
  - [x] 7.2.3.4 显示查询优化建议（基于执行计划）
  - [x] 7.2.3.5 实现执行计划详细信息展示（成本、行数等）
  - [x] 7.2.3.6 添加执行计划导出功能
- [x] 7.2.4 实现SQL执行结果分标签展示
  - [x] 7.2.4.1 为每个查询结果创建独立的标签页
  - [x] 7.2.4.2 实现结果标签页切换功能
  - [x] 7.2.4.3 实现结果标签页关闭功能
  - [x] 7.2.4.4 显示每个结果标签页的查询信息（SQL、执行时间、行数）
  - [x] 7.2.4.5 实现结果标签页与查询标签页的关联
- [x] 7.2.5 添加查询执行取消功能
  - [x] 7.2.5.1 后端实现查询取消API端点（支持取消正在执行的查询）
  - [x] 7.2.5.2 前端添加取消按钮（在执行中显示）
  - [x] 7.2.5.3 实现取消功能（发送取消请求）
  - [x] 7.2.5.4 处理取消后的状态更新（显示已取消状态）
  - [x] 7.2.5.5 实现取消操作的错误处理

**本周目标**: 提升AI交互体验，增加智能化功能，完善查询控制台

**交付物**: 📋 AI建议面板、智能提示系统、上下文感知功能

**验证功能**:
- [ ] AI建议面板显示多个候选SQL
- [ ] SQL解释气泡提示正确显示
- [ ] 置信度分数准确展示
- [ ] 基于历史的智能建议有效
- [ ] AI生成历史正确记录

**测试用例**:
```typescript
// TC-7.1.1 AI建议面板测试
describe('AI Suggestions Panel', () => {
  test('should show multiple SQL candidates', async () => {
    const component = mount(AiSuggestionsPanel);
    await component.vm.generateSuggestions('查询用户');
    expect(component.findAll('[data-type="sql-candidate"]')).toHaveLength(3);
  });

  test('should display confidence score', async () => {
    const component = mount(AiSuggestionsPanel);
    await component.vm.generateSuggestions('查询用户');
    const firstCandidate = component.find('[data-type="sql-candidate"]');
    expect(firstCandidate.text()).toMatch(/\d+%/); // 包含百分比
  });

  test('should apply selected SQL to editor', async () => {
    const component = mount(AiSuggestionsPanel);
    await component.vm.generateSuggestions('查询用户');
    await component.find('[data-action="apply-sql"]').trigger('click');
    expect(mockEditor.getValue()).toContain('SELECT');
  });
});

// TC-7.1.2 SQL解释气泡测试
describe('SQL Explanation Tooltip', () => {
  test('should show explanation on hover', async () => {
    const component = mount(SqlEditor, {
      props: {value: 'SELECT * FROM users WHERE age > 18'}
    });
    await component.find('[data-keyword="WHERE"]').trigger('mouseenter');
    await nextTick();
    expect(component.find('[data-type="tooltip"]')).toExist();
    expect(component.find('[data-type="tooltip"]').text()).toContain('条件筛选');
  });

  test('should hide explanation on mouse leave', async () => {
    const component = mount(SqlEditor);
    await component.find('[data-keyword="WHERE"]').trigger('mouseenter');
    await component.find('[data-keyword="WHERE"]').trigger('mouseleave');
    await nextTick();
    expect(component.find('[data-type="tooltip"]')).not.toExist();
  });
});

// TC-7.1.3 上下文感知测试
describe('Context-Aware Suggestions', () => {
  test('should use query history for better suggestions', async () => {
    // 设置查询历史
    queryHistory.set([
      'SELECT * FROM users WHERE department = "Engineering"',
      'SELECT * FROM users WHERE age > 25'
    ]);
    
    const result = await aiService.generateSql('查询工程部门用户');
    // AI应该基于历史推断出department字段
    expect(result.sql).toContain('department');
    expect(result.sql).toContain('Engineering');
  });

  test('should suggest related tables', async () => {
    const component = mount(AiSuggestionsPanel);
    await component.vm.generateSuggestions('查询用户的订单');
    const suggestions = component.findAll('[data-type="suggestion"]');
    expect(suggestions.some(s => s.text().includes('JOIN'))).toBe(true);
  });
});

// TC-7.1.4 AI历史记录测试
describe('AI Generation History', () => {
  test('should record AI generations', async () => {
    const component = mount(AiHistoryPanel);
    await aiService.generateSql('查询用户');
    await nextTick();
    expect(component.findAll('[data-type="history-item"]')).toHaveLength(1);
  });

  test('should replay previous generation', async () => {
    const component = mount(AiHistoryPanel);
    await component.find('[data-action="replay"]').trigger('click');
    expect(mockEditor.getValue()).toBe('SELECT * FROM users');
  });

  test('should filter history by date', async () => {
    const component = mount(AiHistoryPanel);
    await component.find('[data-filter="today"]').trigger('click');
    const items = component.findAll('[data-type="history-item"]');
    items.forEach(item => {
      expect(item.dataset.date).toBe(today());
    });
  });
});
```

**集成测试**:
```bash
# TC-7.1.5 AI工作流集成测试
# 1. 用户输入自然语言
# 2. AI生成多个候选SQL
# 3. 用户选择其中一个
# 4. SQL填充到编辑器
# 5. 用户执行查询
# 6. 查看结果和AI解释

# 预期: 整个流程流畅，无错误
```

**测试点**:
- [ ] AI建议面板UI美观
- [ ] 候选SQL按置信度排序
- [ ] 置信度计算合理（0-100%）
- [ ] 可展开查看SQL详细解释
- [ ] 可快速应用到编辑器
- [ ] 可标记候选SQL为有用/无用（反馈）
- [ ] SQL解释气泡定位准确
- [ ] 解释内容简洁易懂
- [ ] 支持关键字、函数、子句解释
- [ ] 复杂SQL提供分步解释
- [ ] 历史查询正确存储（LocalStorage）
- [ ] 历史查询可搜索和过滤
- [ ] 基于历史的建议更准确
- [ ] 上下文包含最近10条查询
- [ ] AI历史记录按时间倒序
- [ ] 历史记录显示查询和结果
- [ ] 可清除历史记录
- [ ] 历史记录持久化

---

## 🎨 Phase 3: UI/UX完善 (Week 8-9) - 📋 部分完成

### Week 8: 高级UI功能 ✅ 100% (33/33)

#### 8.1 数据库树形结构
- [x] 8.1.1 创建DatabaseTree.svelte组件 ✅
- [x] 8.1.2 实现树形结构展示 (连接→数据库→表) ✅
- [x] 8.1.3 实现展开/折叠功能 ✅
- [x] 8.1.4 实现搜索和过滤功能（带高亮显示）
- [x] 8.1.5 实现右键菜单 (查看数据、统计行数、查看结构、删除表)
- [x] 8.1.6 实现拖拽重新排序 ✅
- [x] 8.1.7 添加表图标和状态标识

#### 8.2 表结构查看器
- [x] 8.2.1 创建TableStructureViewer.svelte组件 ✅
- [x] 8.2.2 显示字段定义 (名称、类型、约束) ✅
- [x] 8.2.3 显示索引信息
- [x] 8.2.4 显示外键关系
- [x] 8.2.5 实现表结构编辑功能 ✅
- [x] 8.2.6 实现字段添加/删除功能 ✅
- [x] 8.2.7 实现AI一键建表功能（Table Copilot） ✅
- [x] 8.2.8 实现可视化建表界面（填写表名、列信息、索引） ✅

#### 8.3 查询历史管理
- [x] 8.3.1 创建QueryHistory.svelte组件
- [x] 8.3.2 实现历史记录本地存储 (LocalStorage)
- [x] 8.3.3 实现历史记录搜索和过滤
- [x] 8.3.4 实现查询收藏功能
- [x] 8.3.5 实现历史记录快速复用（重放按钮）
- [x] 8.3.6 添加历史记录分类 (成功/失败状态过滤)
- [x] 8.3.7 实现历史记录持久化 (本地SQLite存储)

#### 8.4 连接管理
- [x] 8.4.1 创建ConnectionManager.svelte组件 (后端完成，前端待实现)
- [x] 8.4.2 实现连接配置表单 (SQLite/MySQL/PostgreSQL)
- [x] 8.4.3 实现连接测试功能
- [x] 8.4.4 实现连接保存和加载 (本地SQLite存储)
- [x] 8.4.5 实现连接编辑和删除
- [x] 8.4.6 添加连接状态指示器
- [x] 8.4.7 实现连接切换功能
- [x] 8.4.8 实现环境标签功能（测试/生产环境区分）
- [ ] 8.4.9 实现SSH连接支持（需要后端支持）
- [x] 8.4.10 实现连接URL手动输入支持
- [x] 8.4.11 添加高级配置选项（连接超时、字符集等）

#### 8.5 状态管理优化
- [x] 8.5.1 完善appStore.ts (全局状态)
- [x] 8.5.2 创建queryStore.ts (查询状态)
- [x] 8.5.3 创建connectionStore.ts (连接状态)
- [x] 8.5.4 创建tabStore.ts (标签页状态管理)
- [x] 8.5.5 实现状态持久化 (LocalStorage)
- [x] 8.5.6 添加状态调试工具

#### 8.6 数据编辑功能（参考Chat2DB数据编辑器）
- [x] 8.6.1 实现表数据直接编辑（双击编辑单元格）
  - [x] 8.6.1.1 创建可编辑表格组件（EditableTable.svelte）
  - [x] 8.6.1.2 实现单元格编辑功能（双击进入编辑模式）
  - [x] 8.6.1.3 实现数据验证（类型验证、约束验证）
  - [x] 8.6.1.4 实现保存功能（单行保存、批量保存）
  - [x] 8.6.1.5 实现撤销/重做功能 ✅
  - [x] 8.6.1.6 添加行号显示和行选择功能 ✅
  - [x] 8.6.1.7 实现编辑状态指示（已修改、已保存、错误） ✅
- [x] 8.6.2 实现表数据批量插入
  - [x] 8.6.2.1 创建批量插入界面（BulkInsertDialog.svelte）
  - [x] 8.6.2.2 实现数据导入验证（格式验证、类型验证）
  - [x] 8.6.2.3 实现批量插入API端点（前端api.ts已实现，后端已存在）
  - [x] 8.6.2.4 实现插入进度显示
  - [x] 8.6.2.5 实现插入结果反馈（成功/失败统计）
- [x] 8.6.3 实现表数据批量更新
  - [x] 8.6.3.1 创建批量更新界面（BulkUpdateDialog.svelte）
  - [x] 8.6.3.2 实现更新条件配置（WHERE条件构建器）
  - [x] 8.6.3.3 实现批量更新API端点（前端api.ts已存在，已集成）
  - [x] 8.6.3.4 实现更新预览功能（显示将要更新的行数）
  - [x] 8.6.3.5 实现更新确认对话框
- [x] 8.6.4 实现表数据批量删除
  - [x] 8.6.4.1 创建批量删除界面（BulkDeleteDialog.svelte）
  - [x] 8.6.4.2 实现删除条件配置（复用WHERE条件构建器）
  - [x] 8.6.4.3 实现批量删除API端点（前端api.ts已存在，已集成）
  - [x] 8.6.4.4 实现删除预览功能（显示将要删除的行数）
  - [x] 8.6.4.5 实现删除确认对话框（防止误操作）

#### 8.7 数据导入功能（参考Chat2DB）
- [x] 8.7.1 添加数据导入功能（CSV/Excel/JSON）
  - [x] 8.7.1.1 创建数据导入界面（DataImportDialog.svelte）
  - [x] 8.7.1.2 实现文件上传功能（支持拖拽上传）
  - [x] 8.7.1.3 实现数据预览功能（显示前N行数据）
  - [x] 8.7.1.4 实现数据映射配置（源列到目标列的映射）
  - [x] 8.7.1.5 实现导入验证（数据类型、约束、重复检查）
  - [x] 8.7.1.6 实现导入执行功能
  - [x] 8.7.1.7 实现导入进度显示
  - [x] 8.7.1.8 实现导入结果反馈（成功/失败统计）
  - [x] 8.7.1.9 支持CSV格式导入（分隔符、编码配置）
  - [x] 8.7.1.10 支持Excel格式导入（多Sheet支持）
  - [x] 8.7.1.11 支持JSON格式导入（嵌套JSON展开）

#### 8.8 数据库结构同步（参考Chat2DB）
- [x] 8.8.1 实现数据库结构同步功能
  - [x] 8.8.1.1 创建结构对比界面（SchemaCompareDialog.svelte）
  - [x] 8.8.1.2 实现结构差异检测（表、列、索引、外键）
  - [x] 8.8.1.3 实现同步脚本生成（ALTER TABLE等）
  - [x] 8.8.1.4 实现同步预览功能（显示将要执行的SQL）
  - [x] 8.8.1.5 实现同步执行功能
  - [x] 8.8.1.6 实现同步结果反馈

**本周目标**: 完成核心UI组件、查询历史、连接管理

**交付物**: 🔄 完整的数据库树、表结构查看器、查询历史、连接管理

**验证功能**:
- ✅ 数据库树正确展示层级结构
- ✅ 表结构查看器显示完整信息
- [ ] 右键菜单功能正常
- [ ] 搜索过滤快速准确
- [ ] 查询历史正确保存和展示
- [ ] 连接管理功能完整

**测试用例**:
```typescript
// TC-8.1.1 数据库树测试
describe('DatabaseTree Component', () => {
  test('should render database hierarchy', () => {
    const tree = mount(DatabaseTree, {
      props: {
        connections: [
          {id: '1', name: 'Local SQLite', databases: [
            {name: 'test.db', tables: ['users', 'orders']}
          ]}
        ]
      }
    });
    expect(tree.find('[data-level="connection"]')).toExist();
    expect(tree.find('[data-level="database"]')).toExist();
    expect(tree.find('[data-level="table"]')).toExist();
  });

  test('should expand/collapse nodes', async () => {
    const tree = mount(DatabaseTree);
    const node = tree.find('[data-node="database-1"]');
    expect(tree.findAll('[data-parent="database-1"]')).toHaveLength(0);
    await node.trigger('click');
    expect(tree.findAll('[data-parent="database-1"]')).toHaveLength(2);
  });

  test('should filter nodes by search', async () => {
    const tree = mount(DatabaseTree);
    await tree.find('[data-action="search"]').setValue('user');
    const visibleNodes = tree.findAll('[data-visible="true"]');
    expect(visibleNodes.every(n => n.text().toLowerCase().includes('user'))).toBe(true);
  });

  test('should show context menu on right click', async () => {
    const tree = mount(DatabaseTree);
    await tree.find('[data-table="users"]').trigger('contextmenu');
    expect(tree.find('[data-type="context-menu"]')).toExist();
    expect(tree.text()).toContain('查看数据');
    expect(tree.text()).toContain('查看结构');
  });
});

// TC-8.2.1 表结构查看器测试
describe('TableStructureViewer Component', () => {
  test('should display table columns', () => {
    const viewer = mount(TableStructureViewer, {
      props: {
        table: {
          name: 'users',
          columns: [
            {name: 'id', type: 'INTEGER', primary_key: true},
            {name: 'name', type: 'TEXT', nullable: false}
          ]
        }
      }
    });
    expect(viewer.findAll('[data-type="column"]')).toHaveLength(2);
    expect(viewer.text()).toContain('id');
    expect(viewer.text()).toContain('INTEGER');
    expect(viewer.text()).toContain('主键');
  });

  test('should display indexes', () => {
    const viewer = mount(TableStructureViewer, {
      props: {
        table: {
          name: 'users',
          indexes: [{name: 'idx_name', columns: ['name']}]
        }
      }
    });
    expect(viewer.find('[data-section="indexes"]')).toExist();
    expect(viewer.text()).toContain('idx_name');
  });
});

// TC-8.3.1 查询历史测试
describe('QueryHistory Component', () => {
  test('should display query history', () => {
    localStorage.setItem('queryHistory', JSON.stringify([
      {sql: 'SELECT * FROM users', timestamp: Date.now(), success: true}
    ]));
    const history = mount(QueryHistory);
    expect(history.findAll('[data-type="history-item"]')).toHaveLength(1);
  });

  test('should filter history by status', async () => {
    const history = mount(QueryHistory);
    await history.find('[data-filter="failed"]').trigger('click');
    const items = history.findAll('[data-type="history-item"]');
    expect(items.every(item => item.dataset.success === 'false')).toBe(true);
  });

  test('should search history', async () => {
    const history = mount(QueryHistory);
    await history.find('[data-action="search"]').setValue('users');
    const items = history.findAll('[data-type="history-item"]');
    expect(items.every(item => item.text().includes('users'))).toBe(true);
  });

  test('should add to favorites', async () => {
    const history = mount(QueryHistory);
    await history.find('[data-action="favorite"]').trigger('click');
    expect(history.find('[data-favorite="true"]')).toExist();
  });

  test('should replay query', async () => {
    const history = mount(QueryHistory);
    await history.find('[data-action="replay"]').trigger('click');
    expect(mockEditor.getValue()).toBe('SELECT * FROM users');
  });
});

// TC-8.4.1 连接管理测试
describe('ConnectionManager Component', () => {
  test('should display connection list', () => {
    const manager = mount(ConnectionManager, {
      props: {
        connections: [
          {id: '1', name: 'Local', type: 'SQLite', status: 'connected'}
        ]
      }
    });
    expect(manager.findAll('[data-type="connection"]')).toHaveLength(1);
    expect(manager.text()).toContain('Local');
  });

  test('should create new connection', async () => {
    const manager = mount(ConnectionManager);
    await manager.find('[data-action="new-connection"]').trigger('click');
    await manager.find('[name="name"]').setValue('Test DB');
    await manager.find('[name="type"]').setValue('SQLite');
    await manager.find('[name="path"]').setValue(':memory:');
    await manager.find('[data-action="save"]').trigger('click');
    expect(mockConnections).toContainEqual({name: 'Test DB', type: 'SQLite'});
  });

  test('should test connection', async () => {
    const manager = mount(ConnectionManager);
    await manager.find('[data-action="test-connection"]').trigger('click');
    await nextTick();
    expect(manager.text()).toContain('连接成功');
  });

  test('should edit connection', async () => {
    const manager = mount(ConnectionManager);
    await manager.find('[data-action="edit"]').trigger('click');
    await manager.find('[name="name"]').setValue('Updated Name');
    await manager.find('[data-action="save"]').trigger('click');
    expect(mockConnections[0].name).toBe('Updated Name');
  });

  test('should delete connection with confirmation', async () => {
    const manager = mount(ConnectionManager);
    await manager.find('[data-action="delete"]').trigger('click');
    expect(manager.find('[data-type="confirm-dialog"]')).toExist();
    await manager.find('[data-action="confirm"]').trigger('click');
    expect(mockConnections).toHaveLength(0);
  });
});

// TC-8.5.1 状态管理测试
describe('Store Management', () => {
  test('should persist state to localStorage', () => {
    appStore.set({activeConnection: '1', theme: 'dark'});
    const stored = JSON.parse(localStorage.getItem('appStore'));
    expect(stored.activeConnection).toBe('1');
    expect(stored.theme).toBe('dark');
  });

  test('should restore state on load', () => {
    localStorage.setItem('appStore', JSON.stringify({theme: 'dark'}));
    const store = loadAppStore();
    expect(store.theme).toBe('dark');
  });
});
```

**性能测试**:
```typescript
// TC-8.1.2 树形结构性能测试
describe('DatabaseTree Performance', () => {
  test('should handle large tree efficiently', () => {
    const largeTree = generateLargeTree(1000); // 1000个节点
    const start = performance.now();
    const tree = mount(DatabaseTree, {props: {data: largeTree}});
    const renderTime = performance.now() - start;
    expect(renderTime).toBeLessThan(200); // 渲染时间<200ms
  });

  test('should update efficiently on search', async () => {
    const tree = mount(DatabaseTree, {props: {data: largeTree}});
    const start = performance.now();
    await tree.find('[data-action="search"]').setValue('test');
    const searchTime = performance.now() - start;
    expect(searchTime).toBeLessThan(100); // 搜索响应<100ms
  });
});
```

**测试点**:
- [x] 数据库树正确渲染3层结构
- [x] 节点展开/折叠动画流畅
- [x] 图标正确显示（数据库、表、列）
- [ ] 搜索高亮匹配文本
- [ ] 搜索支持模糊匹配
- [ ] 搜索性能良好（<100ms）
- [ ] 右键菜单定位准确
- [ ] 菜单选项根据节点类型变化
- [ ] 点击菜单项执行相应操作
- [ ] 拖拽节点调整顺序
- [x] 表结构显示所有列信息
- [x] 主键、外键、索引正确标识
- [x] 约束信息完整显示
- [ ] 可编辑表结构（添加/删除列）
- [ ] 修改保存到数据库
- [ ] 历史记录按时间倒序
- [ ] 历史记录分类显示（成功/失败/AI生成）
- [ ] 收藏夹功能正常
- [ ] 历史记录可导出
- [ ] 连接列表显示状态（已连接/已断开）
- [ ] 连接测试给出详细反馈
- [ ] 表单验证完整（必填项、格式）
- [ ] 连接信息安全存储（密码加密）
- [ ] 连接切换即时生效
- [ ] 状态持久化正常
- [ ] 状态更新触发UI更新

---

### Week 9: Dashboard看板系统与数据可视化 📋 0% (0/0)

#### 9.1 Dashboard看板系统（参考Chat2DB Dashboard）
- [x] 9.1.1 实现Dashboard看板功能
  - [ ] 9.1.1.1 创建Dashboard组件（Dashboard.svelte）
  - [ ] 9.1.1.2 实现Dashboard布局管理（Grid布局系统）
  - [ ] 9.1.1.3 实现Dashboard保存和加载功能
  - [ ] 9.1.1.4 实现Dashboard分享功能（生成分享链接）
  - [ ] 9.1.1.5 实现Dashboard模板功能（预设布局）
  - [ ] 9.1.1.6 实现Dashboard响应式布局（适配不同屏幕）
- [ ] 9.1.2 添加数据可视化图表 (Chart.js/D3.js/ECharts)
  - [ ] 9.1.2.1 集成图表库（选择Chart.js或ECharts）
  - [ ] 9.1.2.2 实现柱状图组件（BarChart.svelte）
  - [ ] 9.1.2.3 实现折线图组件（LineChart.svelte）
  - [ ] 9.1.2.4 实现饼图组件（PieChart.svelte）
  - [ ] 9.1.2.5 实现散点图组件（ScatterChart.svelte）
  - [ ] 9.1.2.6 实现面积图组件（AreaChart.svelte）
  - [ ] 9.1.2.7 实现仪表盘组件（GaugeChart.svelte）
  - [ ] 9.1.2.8 实现图表配置界面（选择X轴、Y轴、图表类型）
- [ ] 9.1.3 实现查询结果一键生成图表
  - [ ] 9.1.3.1 创建图表生成按钮（在查询结果中）
  - [ ] 9.1.3.2 实现数据到图表的自动转换
  - [ ] 9.1.3.3 实现图表类型智能推荐（基于数据特征）
  - [ ] 9.1.3.4 实现图表配置向导（引导用户配置图表）
- [ ] 9.1.4 实现AI自动生成图表功能
  - [ ] 9.1.4.1 实现AI图表生成API端点
  - [ ] 9.1.4.2 创建AI图表生成界面
  - [ ] 9.1.4.3 实现图表配置自动生成（基于自然语言描述）
  - [ ] 9.1.4.4 实现图表类型智能选择（AI推荐最佳图表类型）
- [ ] 9.1.5 实现图表导出功能
  - [ ] 9.1.5.1 实现图表导出为PNG
  - [ ] 9.1.5.2 实现图表导出为SVG
  - [ ] 9.1.5.3 实现图表导出为PDF
  - [ ] 9.1.5.4 实现图表数据导出（CSV/JSON）
- [ ] 9.1.6 实现图表定时刷新功能
  - [ ] 9.1.6.1 实现图表自动刷新配置（设置刷新间隔）
  - [ ] 9.1.6.2 实现图表刷新状态显示
  - [ ] 9.1.6.3 实现图表刷新暂停/恢复功能

#### 9.2 对话式AI分析（参考Chat2DB）
- [ ] 9.2.1 实现对话式数据分析功能
  - [ ] 9.2.1.1 创建对话界面（ChatInterface.svelte）
  - [ ] 9.2.1.2 实现对话上下文管理（保持对话历史）
  - [ ] 9.2.1.3 实现AI分析结果展示（文本、图表、表格）
  - [ ] 9.2.1.4 实现Dashboard自动生成（基于对话内容）
  - [ ] 9.2.1.5 实现对话历史保存和加载
  - [ ] 9.2.1.6 实现对话分享功能

### Week 10: 主题与响应式设计 ✅ 100% (10/10)

#### 10.1 主题系统
- [x] 10.1.1 实现亮色/暗色主题切换
- [x] 10.1.2 创建主题配置 (颜色、字体、间距)
- [x] 10.1.3 实现主题持久化
- [x] 10.1.4 优化暗色模式下的可读性
- [x] 10.1.5 实现自定义主题编辑器

#### 10.2 响应式优化
- [x] 10.2.1 优化移动端布局
- [x] 10.2.2 实现侧边栏折叠/展开
- [x] 10.2.3 优化小屏幕下的表格显示
- [x] 10.2.4 添加触摸手势支持
- [x] 10.2.5 优化平板设备体验

#### 10.3 用户体验增强
- [x] 10.3.1 添加键盘快捷键系统
- [x] 10.3.2 实现操作提示 (Tooltips)
- [x] 10.3.3 添加加载动画和骨架屏
- [x] 10.3.4 实现错误边界和友好错误提示
- [x] 10.3.5 添加操作确认对话框

**本周目标**: 完成主题系统、响应式优化、用户体验提升

**交付物**: 📋 主题系统、响应式布局、键盘快捷键、友好的错误处理

**验证功能**:
- [ ] 主题切换即时生效
- [ ] 暗色模式视觉舒适
- [ ] 移动端布局合理
- [ ] 触摸操作流畅
- [ ] 快捷键正常工作
- [ ] 错误提示友好

**测试用例**:
```typescript
// TC-9.1.1 主题切换测试
describe('Theme System', () => {
  test('should switch to dark theme', async () => {
    const app = mount(App);
    await app.find('[data-action="toggle-theme"]').trigger('click');
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(localStorage.getItem('theme')).toBe('dark');
  });

  test('should persist theme preference', () => {
    localStorage.setItem('theme', 'dark');
    const app = mount(App);
    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });

  test('should apply theme colors correctly', async () => {
    const app = mount(App);
    await app.vm.setTheme('dark');
    const bgColor = getComputedStyle(document.body).backgroundColor;
    expect(bgColor).toBe('rgb(26, 32, 44)'); // 暗色背景
  });

  test('should update all components on theme change', async () => {
    const app = mount(App);
    await app.vm.setTheme('dark');
    const buttons = app.findAll('button');
    buttons.forEach(btn => {
      const style = getComputedStyle(btn.element);
      expect(style.backgroundColor).not.toBe('rgb(255, 255, 255)');
    });
  });
});

// TC-9.2.1 响应式布局测试
describe('Responsive Design', () => {
  test('should adapt to mobile viewport', () => {
    global.innerWidth = 375;
    global.innerHeight = 667;
    window.dispatchEvent(new Event('resize'));
    const app = mount(App);
    expect(app.find('[data-layout="mobile"]')).toExist();
    expect(app.find('.sidebar').classes()).toContain('sidebar-collapsed');
  });

  test('should adapt to tablet viewport', () => {
    global.innerWidth = 768;
    window.dispatchEvent(new Event('resize'));
    const app = mount(App);
    expect(app.find('[data-layout="tablet"]')).toExist();
  });

  test('should show sidebar toggle on small screens', () => {
    global.innerWidth = 600;
    window.dispatchEvent(new Event('resize'));
    const app = mount(App);
    expect(app.find('[data-action="toggle-sidebar"]')).toExist();
  });

  test('should collapse sidebar on mobile', async () => {
    global.innerWidth = 375;
    const app = mount(App);
    const sidebar = app.find('.sidebar');
    expect(sidebar.isVisible()).toBe(false);
    await app.find('[data-action="toggle-sidebar"]').trigger('click');
    expect(sidebar.isVisible()).toBe(true);
  });
});

// TC-9.2.2 触摸手势测试
describe('Touch Gestures', () => {
  test('should support swipe to open sidebar', async () => {
    const app = mount(App);
    const touchStartEvent = new TouchEvent('touchstart', {
      touches: [{clientX: 0, clientY: 100}]
    });
    const touchEndEvent = new TouchEvent('touchend', {
      changedTouches: [{clientX: 200, clientY: 100}]
    });
    app.element.dispatchEvent(touchStartEvent);
    app.element.dispatchEvent(touchEndEvent);
    await nextTick();
    expect(app.find('.sidebar').isVisible()).toBe(true);
  });
});

// TC-9.3.1 键盘快捷键测试
describe('Keyboard Shortcuts', () => {
  test('should execute query on Ctrl+Enter', async () => {
    const app = mount(App);
    await app.find('.sql-editor').setValue('SELECT * FROM users');
    await app.trigger('keydown', {ctrlKey: true, key: 'Enter'});
    expect(mockExecuteQuery).toHaveBeenCalled();
  });

  test('should open command palette on Ctrl+P', async () => {
    const app = mount(App);
    await app.trigger('keydown', {ctrlKey: true, key: 'p'});
    expect(app.find('[data-type="command-palette"]')).toExist();
  });

  test('should format SQL on Ctrl+Shift+F', async () => {
    const app = mount(App);
    await app.find('.sql-editor').setValue('select*from users');
    await app.trigger('keydown', {ctrlKey: true, shiftKey: true, key: 'F'});
    expect(app.find('.sql-editor').getValue()).toBe('SELECT *\nFROM users');
  });

  test('should show shortcuts help on F1', async () => {
    const app = mount(App);
    await app.trigger('keydown', {key: 'F1'});
    expect(app.find('[data-type="shortcuts-help"]')).toExist();
  });
});

// TC-9.3.2 Tooltip测试
describe('Tooltips', () => {
  test('should show tooltip on hover', async () => {
    const app = mount(App);
    const button = app.find('[data-tooltip="执行查询"]');
    await button.trigger('mouseenter');
    await new Promise(resolve => setTimeout(resolve, 500)); // 等待tooltip显示
    expect(document.querySelector('[role="tooltip"]')).toExist();
    expect(document.querySelector('[role="tooltip"]').textContent).toBe('执行查询');
  });

  test('should hide tooltip on mouse leave', async () => {
    const app = mount(App);
    const button = app.find('[data-tooltip="执行查询"]');
    await button.trigger('mouseenter');
    await button.trigger('mouseleave');
    await nextTick();
    expect(document.querySelector('[role="tooltip"]')).not.toExist();
  });
});

// TC-9.3.3 加载状态测试
describe('Loading States', () => {
  test('should show skeleton screen on initial load', () => {
    const app = mount(App, {props: {loading: true}});
    expect(app.find('[data-type="skeleton"]')).toExist();
  });

  test('should show spinner during query execution', async () => {
    const app = mount(App);
    const promise = app.vm.executeQuery('SELECT * FROM users');
    await nextTick();
    expect(app.find('[data-type="spinner"]')).toExist();
    await promise;
    expect(app.find('[data-type="spinner"]')).not.toExist();
  });
});

// TC-9.3.4 错误处理测试
describe('Error Handling', () => {
  test('should show friendly error message', async () => {
    const app = mount(App);
    await app.vm.executeQuery('INVALID SQL');
    expect(app.find('[data-type="error-message"]')).toExist();
    expect(app.text()).toContain('SQL语法错误');
    expect(app.text()).not.toContain('Error:'); // 不显示原始错误
  });

  test('should show error boundary on component crash', () => {
    const BrokenComponent = {
      render() { throw new Error('Test error'); }
    };
    const app = mount(ErrorBoundary, {
      slots: {default: BrokenComponent}
    });
    expect(app.find('[data-type="error-boundary"]')).toExist();
    expect(app.text()).toContain('出现了一些问题');
  });

  test('should allow retry after error', async () => {
    const app = mount(App);
    await app.vm.executeQuery('INVALID SQL');
    await app.find('[data-action="retry"]').trigger('click');
    expect(mockExecuteQuery).toHaveBeenCalledTimes(2);
  });
});

// TC-9.3.5 确认对话框测试
describe('Confirmation Dialogs', () => {
  test('should show confirm dialog before delete', async () => {
    const app = mount(App);
    await app.find('[data-action="delete-connection"]').trigger('click');
    expect(app.find('[data-type="confirm-dialog"]')).toExist();
    expect(app.text()).toContain('确定要删除');
  });

  test('should execute action on confirm', async () => {
    const app = mount(App);
    await app.find('[data-action="delete-connection"]').trigger('click');
    await app.find('[data-action="confirm"]').trigger('click');
    expect(mockDeleteConnection).toHaveBeenCalled();
  });

  test('should cancel action on dismiss', async () => {
    const app = mount(App);
    await app.find('[data-action="delete-connection"]').trigger('click');
    await app.find('[data-action="cancel"]').trigger('click');
    expect(mockDeleteConnection).not.toHaveBeenCalled();
  });
});
```

**视觉回归测试**:
```typescript
// TC-9.1.2 视觉回归测试
describe('Visual Regression', () => {
  test('light theme screenshot matches baseline', async () => {
    const app = mount(App);
    await app.vm.setTheme('light');
    const screenshot = await takeScreenshot(app);
    expect(screenshot).toMatchImageSnapshot();
  });

  test('dark theme screenshot matches baseline', async () => {
    const app = mount(App);
    await app.vm.setTheme('dark');
    const screenshot = await takeScreenshot(app);
    expect(screenshot).toMatchImageSnapshot();
  });
});
```

**可访问性测试**:
```typescript
// TC-9.3.6 可访问性测试
describe('Accessibility', () => {
  test('should have proper ARIA labels', () => {
    const app = mount(App);
    expect(app.find('button[aria-label]')).toExist();
    expect(app.find('[role="navigation"]')).toExist();
  });

  test('should support keyboard navigation', async () => {
    const app = mount(App);
    const firstButton = app.find('button');
    firstButton.element.focus();
    await app.trigger('keydown', {key: 'Tab'});
    expect(document.activeElement).not.toBe(firstButton.element);
  });

  test('should meet WCAG AA contrast ratio', () => {
    const app = mount(App);
    const bgColor = getComputedStyle(document.body).backgroundColor;
    const textColor = getComputedStyle(document.body).color;
    const ratio = calculateContrastRatio(bgColor, textColor);
    expect(ratio).toBeGreaterThan(4.5); // WCAG AA标准
  });
});
```

**测试点**:
- [ ] 主题配置文件完整（颜色、字体、间距）
- [ ] 亮色主题色彩和谐
- [ ] 暗色主题对比度适中
- [ ] 主题切换无闪烁
- [ ] 所有组件支持主题
- [ ] 自定义主题可保存
- [ ] 主题预览功能正常
- [ ] 移动端（<768px）布局合理
- [ ] 平板端（768-1024px）布局优化
- [ ] 桌面端（>1024px）布局美观
- [ ] 侧边栏在小屏幕可折叠
- [ ] 表格在小屏幕横向滚动
- [ ] 触摸目标足够大（>44px）
- [ ] 滑动手势流畅自然
- [ ] 键盘快捷键不冲突
- [ ] 快捷键有视觉提示
- [ ] 快捷键帮助文档完整
- [ ] Tooltip显示及时（500ms内）
- [ ] Tooltip位置智能调整
- [ ] 骨架屏模拟真实布局
- [ ] 加载动画不刺眼
- [ ] 错误信息用户友好
- [ ] 错误可重试
- [ ] 确认对话框阻止误操作
- [ ] ARIA标签完整
- [ ] 键盘可访问所有功能
- [ ] 色盲友好（不仅依赖颜色）

---

## 🚀 Phase 4: 测试、优化与发布 (Week 11-13)

### Week 11: 全面测试 🔄 43% (3/7)

#### 10.1 测试覆盖
- [x] 10.1.1 完善后端单元测试 (已有 security_test.rs, ai_test.rs, template_test.rs)
- [ ] 10.1.2 编写前端组件单元测试 (Jest + Testing Library)
- [ ] 10.1.3 编写端到端测试 (Playwright/Cypress)
- [ ] 10.1.4 编写性能测试 (压力测试、并发测试)
- [x] 10.1.5 实现集成测试 (已有 api_integration_test.rs)
- [x] 10.1.6 实现测试覆盖率报告 (cargo-tarpaulin)
- [ ] 10.1.7 达到测试覆盖率目标 (>90%)

**本周目标**: 实现100%测试覆盖率，确保代码质量

**交付物**: 🔄 完整的测试套件、测试报告、持续集成配置

**验证功能**:
- [x] 后端单元测试覆盖率 >90%
- [ ] 前端单元测试覆盖率 >90%
- [ ] 端到端测试覆盖核心流程
- [ ] 性能测试达到目标
- [ ] 集成测试全部通过
- [x] 测试报告自动生成

**测试用例**:
```bash
# TC-10.1.1 后端单元测试
cd backend
cargo test --all-features
cargo tarpaulin --out Html --output-dir coverage
# 预期: 所有测试通过，覆盖率 >90%

# TC-10.1.2 前端单元测试
cd frontend
npm run test:unit -- --coverage
# 预期: 所有测试通过，覆盖率 >90%

# TC-10.1.3 端到端测试
npm run test:e2e
# 预期: 核心用户流程测试通过
```

**端到端测试场景**:
```typescript
// TC-10.1.3.1 完整SQL生成流程
describe('E2E: SQL Generation Flow', () => {
  test('user can generate and execute SQL', async ({ page }) => {
    // 1. 打开应用
    await page.goto('http://localhost:5173');
    
    // 2. 连接数据库
    await page.click('[data-action="connect-database"]');
    await page.fill('[name="database-path"]', ':memory:');
    await page.click('[data-action="connect"]');
    await expect(page.locator('[data-status="connected"]')).toBeVisible();
    
    // 3. 输入自然语言查询
    await page.fill('[data-input="natural-language"]', '查询所有用户');
    await page.click('[data-action="generate-sql"]');
    
    // 4. 等待AI生成
    await expect(page.locator('[data-type="loading"]')).toBeVisible();
    await expect(page.locator('[data-type="loading"]')).not.toBeVisible();
    
    // 5. 验证SQL已生成
    const sql = await page.locator('.sql-editor').textContent();
    expect(sql).toContain('SELECT');
    expect(sql).toContain('FROM users');
    
    // 6. 执行查询
    await page.click('[data-action="execute"]');
    
    // 7. 验证结果显示
    await expect(page.locator('[data-type="results-table"]')).toBeVisible();
    const rowCount = await page.locator('tbody tr').count();
    expect(rowCount).toBeGreaterThan(0);
  });
});

// TC-10.1.3.2 错误处理流程
describe('E2E: Error Handling', () => {
  test('user sees friendly error on invalid SQL', async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.fill('.sql-editor', 'INVALID SQL QUERY');
    await page.click('[data-action="execute"]');
    
    await expect(page.locator('[data-type="error"]')).toBeVisible();
    await expect(page.locator('[data-type="error"]')).toContainText('SQL语法错误');
  });
});

// TC-10.1.3.3 主题切换流程
describe('E2E: Theme Toggle', () => {
  test('user can toggle between light and dark themes', async ({ page }) => {
    await page.goto('http://localhost:5173');
    
    const bodyBgBefore = await page.evaluate(() => 
      getComputedStyle(document.body).backgroundColor
    );
    
    await page.click('[data-action="toggle-theme"]');
    await page.waitForTimeout(300); // 等待动画
    
    const bodyBgAfter = await page.evaluate(() => 
      getComputedStyle(document.body).backgroundColor
    );
    
    expect(bodyBgBefore).not.toBe(bodyBgAfter);
  });
});
```

**性能测试**:
```typescript
// TC-10.1.4 性能基准测试
describe('Performance Tests', () => {
  test('API response time < 200ms', async () => {
    const start = Date.now();
    const response = await fetch('http://localhost:8080/api/health');
    const duration = Date.now() - start;
    expect(response.status).toBe(200);
    expect(duration).toBeLessThan(200);
  });

  test('Query execution time < 2s', async () => {
    const start = Date.now();
    const response = await fetch('http://localhost:8080/api/database/execute', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({sql: 'SELECT * FROM users LIMIT 1000'})
    });
    const duration = Date.now() - start;
    expect(duration).toBeLessThan(2000);
  });

  test('Frontend load time < 3s', async ({ page }) => {
    const start = Date.now();
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('load');
    const loadTime = Date.now() - start;
    expect(loadTime).toBeLessThan(3000);
  });

  test('Handle 100 concurrent requests', async () => {
    const requests = Array(100).fill(null).map(() =>
      fetch('http://localhost:8080/api/health')
    );
    const responses = await Promise.all(requests);
    expect(responses.every(r => r.status === 200)).toBe(true);
  });
});
```

**压力测试**:
```bash
# TC-10.1.4.1 API压力测试
# 使用Apache Bench测试并发性能
ab -n 1000 -c 100 http://localhost:8080/api/health
# 预期: 无请求失败，平均响应时间 <200ms

# TC-10.1.4.2 查询压力测试
ab -n 100 -c 10 -p query.json -T application/json \
   http://localhost:8080/api/database/execute
# 预期: 无请求失败，平均响应时间 <2s
```

**集成测试矩阵**:
```rust
// TC-10.1.5 集成测试
#[tokio::test]
async fn test_full_api_workflow() {
    let app = create_test_app().await;
    
    // 1. 健康检查
    let response = request(&app, "/api/health").await;
    assert_eq!(response.status(), 200);
    
    // 2. 获取数据库信息
    let response = request(&app, "/api/database/info").await;
    assert_eq!(response.status(), 200);
    
    // 3. AI生成SQL
    let response = post_json(&app, "/api/ai/generate-sql", json!({
        "query": "查询用户",
        "database_info": {}
    })).await;
    assert_eq!(response.status(), 200);
    let body: GenerateSqlResponse = response.json().await;
    assert!(body.success);
    
    // 4. 执行查询
    let response = post_json(&app, "/api/database/execute", json!({
        "sql": body.data.sql
    })).await;
    assert_eq!(response.status(), 200);
}
```

**测试点**:
- [x] 所有Rust模块有单元测试
- [x] 所有公共函数有测试
- [x] 边界条件测试完整
- [ ] 所有Svelte组件有测试
- [ ] 所有Store有测试
- [ ] 所有工具函数有测试
- [ ] 端到端测试覆盖核心流程（SQL生成、执行、结果展示）
- [ ] 端到端测试覆盖错误场景
- [ ] 端到端测试覆盖边缘情况
- [ ] 性能测试达到目标（响应时间、吞吐量）
- [ ] 压力测试无崩溃
- [ ] 内存泄漏测试通过
- [x] 集成测试覆盖API交互
- [ ] 跨浏览器测试通过（Chrome、Firefox、Safari）
- [ ] 不同屏幕尺寸测试通过
- [x] 测试报告自动生成
- [ ] CI/CD自动运行测试
- [ ] 代码覆盖率徽章显示

---

### Week 12: 性能优化 📋 0% (0/0)

#### 11.1 后端性能优化
- [ ] 11.1.1 优化数据库连接池配置
- [ ] 11.1.2 实现查询结果缓存 (Redis/内存缓存)
- [ ] 11.1.3 优化AI API调用 (批量处理、缓存)
- [ ] 11.1.4 实现慢查询监控和告警
- [ ] 11.1.5 优化序列化/反序列化性能
- [ ] 11.1.6 添加性能监控中间件

#### 11.2 前端性能优化
- [ ] 11.2.1 实现组件懒加载
- [ ] 11.2.2 实现代码分割 (Code Splitting)
- [ ] 11.2.3 优化打包体积 (Tree Shaking)
- [ ] 11.2.4 实现资源预加载和缓存策略
- [ ] 11.2.5 优化首屏加载时间
- [ ] 11.2.6 添加性能监控 (Web Vitals)

**本周目标**: 全面优化系统性能，达到生产环境标准

**性能目标**:
- API响应时间 < 200ms
- 查询执行时间 < 2s
- 前端首屏加载 < 3s
- 并发支持 > 100用户

**交付物**: 📋 性能优化报告、监控仪表板、优化后的代码

**验证功能**:
- [ ] API响应时间 <200ms
- [ ] 查询执行时间 <2s
- [ ] 前端首屏加载 <3s
- [ ] 并发支持 >100用户
- [ ] 内存占用 <512MB
- [ ] CPU占用 <50%

**性能基准测试**:
```bash
# TC-11.1.1 后端性能基准
wrk -t4 -c100 -d30s http://localhost:8080/api/health
# 目标: >1000 req/s, 平均延迟 <50ms

# TC-11.1.2 数据库连接池优化测试
# 测试不同连接池大小的性能
for pool_size in 5 10 20 50; do
  POOL_SIZE=$pool_size cargo run &
  ab -n 1000 -c 100 http://localhost:8080/api/database/info
done
# 预期: 找到最优连接池大小

# TC-11.2.1 前端性能测试
npm run lighthouse
# 目标: Performance Score > 90
```

**测试点**:
- [ ] API平均响应时间 <200ms
- [ ] API P99响应时间 <500ms
- [ ] 查询执行平均时间 <2s
- [ ] 并发100用户无错误
- [ ] 连接池配置最优
- [ ] 缓存命中率 >50%
- [ ] 内存使用稳定（无泄漏）
- [ ] CPU使用合理（<80%）
- [ ] 首屏加载 <3s
- [ ] FCP <1.8s
- [ ] LCP <2.5s
- [ ] 打包体积优化（<500KB）
- [ ] 懒加载生效
- [ ] 监控系统部署

---

### Week 13: 部署与发布 📋 0% (0/0)

#### 12.1 部署准备
- [ ] 12.1.1 完善Docker镜像 (多阶段构建优化)
- [ ] 12.1.2 编写Kubernetes部署配置 (可选)
- [ ] 12.1.3 配置CI/CD流水线 (GitHub Actions)
- [ ] 12.1.4 实现自动化部署脚本
- [ ] 12.1.5 配置生产环境变量
- [ ] 12.1.6 实现健康检查和监控

#### 12.2 文档完善
- [ ] 12.2.1 编写用户使用手册
- [ ] 12.2.2 编写开发者文档
- [ ] 12.2.3 编写API文档 (Swagger/OpenAPI)
- [ ] 12.2.4 编写部署文档
- [ ] 12.2.5 创建演示视频和截图
- [ ] 12.2.6 编写CHANGELOG和发布说明

#### 12.3 发布准备
- [ ] 12.3.1 代码审查和质量检查
- [ ] 12.3.2 安全审计和漏洞扫描
- [ ] 12.3.3 性能基准测试
- [ ] 12.3.4 用户验收测试 (UAT)
- [ ] 12.3.5 创建GitHub Release
- [ ] 12.3.6 发布到生产环境

**本周目标**: 完成所有发布准备工作，正式发布v1.0

**交付物**: 📋 生产环境部署、完整文档、发布包

**验证功能**:
- [ ] Docker镜像可正常运行
- [ ] CI/CD自动化流程正常
- [ ] 生产环境健康检查通过
- [ ] 所有文档完整且准确
- [ ] 演示环境可访问
- [ ] 发布包可下载

**部署测试**:
```bash
# TC-12.1.1 Docker构建测试
docker build -t smart-sql:latest .
docker run -p 8080:8080 -p 5173:5173 smart-sql:latest
# 预期: 服务正常启动，端口可访问

# TC-12.1.2 多阶段构建优化验证
docker images smart-sql:latest
# 预期: 镜像大小 <500MB

# TC-12.1.3 健康检查测试
docker ps
# 预期: 健康状态为 healthy

# TC-12.1.4 环境变量配置测试
docker run -e DATABASE_URL=test.db -e OPENAI_API_KEY=sk-test smart-sql:latest
# 预期: 环境变量正确加载

# TC-12.1.5 CI/CD流水线测试
git push origin main
# 预期: GitHub Actions自动运行测试、构建、部署
```

**文档检查清单**:
```markdown
# TC-12.2 文档完整性检查

用户使用手册:
- [ ] 安装说明
- [ ] 快速开始指南
- [ ] 功能介绍（带截图）
- [ ] 常见问题FAQ
- [ ] 故障排除

开发者文档:
- [ ] 项目结构说明
- [ ] 开发环境搭建
- [ ] 代码规范
- [ ] 贡献指南
- [ ] 测试指南

API文档:
- [ ] 所有端点说明
- [ ] 请求/响应示例
- [ ] 错误码说明
- [ ] 认证说明
- [ ] 限流规则

部署文档:
- [ ] Docker部署指南
- [ ] 手动部署指南
- [ ] 配置说明
- [ ] 备份和恢复
- [ ] 监控和日志

演示材料:
- [ ] 产品演示视频
- [ ] 功能截图（亮色/暗色主题）
- [ ] 使用案例
- [ ] 性能数据
```

**安全审计检查**:
```bash
# TC-12.3.1 安全漏洞扫描
# Rust后端
cargo audit
cargo clippy -- -D warnings

# 前端
npm audit
npm audit fix

# Docker镜像
trivy image smart-sql:latest
# 预期: 无高危漏洞

# TC-12.3.2 依赖检查
# 检查过期依赖
cargo outdated
npm outdated

# TC-12.3.3 代码质量检查
# SonarQube扫描
sonar-scanner
# 预期: Quality Gate通过
```

**UAT测试场景**:
```typescript
// TC-12.3.4 用户验收测试
describe('UAT: Complete User Workflow', () => {
  test('Scenario 1: 新用户首次使用', async () => {
    // 1. 打开应用
    // 2. 创建数据库连接
    // 3. 浏览数据库结构
    // 4. 使用AI生成SQL
    // 5. 执行查询
    // 6. 导出结果
    // 7. 保存到收藏夹
    // 预期: 整个流程顺畅无障碍
  });

  test('Scenario 2: 性能测试场景', async () => {
    // 1. 加载大型数据库（1000+表）
    // 2. 执行复杂查询
    // 3. 浏览大结果集（10000+行）
    // 预期: 性能满足要求
  });

  test('Scenario 3: 错误恢复场景', async () => {
    // 1. 执行错误SQL
    // 2. 查看错误信息
    // 3. 修正并重新执行
    // 预期: 错误提示友好，恢复流畅
  });
});
```

**发布检查清单**:
```yaml
# TC-12.3.5 发布前检查
代码质量:
  - [ ] 所有测试通过
  - [ ] 代码覆盖率 >90%
  - [ ] 无编译警告
  - [ ] 无安全漏洞
  - [ ] 代码审查完成

文档:
  - [ ] README完整
  - [ ] API文档生成
  - [ ] CHANGELOG更新
  - [ ] LICENSE文件存在
  - [ ] 贡献指南完成

部署:
  - [ ] Docker镜像构建成功
  - [ ] 生产环境配置正确
  - [ ] 健康检查正常
  - [ ] 监控和告警配置
  - [ ] 备份策略就绪

发布:
  - [ ] 版本号更新
  - [ ] Git标签创建
  - [ ] GitHub Release发布
  - [ ] 发布说明编写
  - [ ] 通知用户/社区
```

**测试点**:
- [ ] Docker镜像构建无错误
- [ ] 镜像大小合理（<500MB）
- [ ] 多阶段构建优化生效
- [ ] 健康检查端点正常
- [ ] 所有环境变量正确配置
- [ ] CI/CD流水线所有阶段通过
- [ ] 自动化测试在CI中运行
- [ ] 自动化部署成功
- [ ] 用户手册完整易懂
- [ ] 开发者文档详细准确
- [ ] API文档与实现一致
- [ ] 部署文档可操作
- [ ] 演示视频制作精良
- [ ] 截图清晰美观
- [ ] CHANGELOG详细记录
- [ ] 所有依赖无已知漏洞
- [ ] 代码质量得分 >80
- [ ] 性能基准测试通过
- [ ] UAT测试场景全部通过
- [ ] GitHub Release创建成功
- [ ] 发布说明吸引人
- [ ] 生产环境稳定运行


- [ ] 12.2.1 编写用户使用手册
- [ ] 12.2.2 编写开发者文档
- [ ] 12.2.3 编写API文档 (Swagger/OpenAPI)
- [ ] 12.2.4 编写部署文档
- [ ] 12.2.5 创建演示视频和截图
- [ ] 12.2.6 编写CHANGELOG和发布说明

#### 12.3 发布准备
- [ ] 12.3.1 代码审查和质量检查
- [ ] 12.3.2 安全审计和漏洞扫描
- [ ] 12.3.3 性能基准测试
- [ ] 12.3.4 用户验收测试 (UAT)
- [ ] 12.3.5 创建GitHub Release
- [ ] 12.3.6 发布到生产环境

**本周目标**: 完成所有发布准备工作，正式发布v1.0

---

## 📝 后续规划 (v1.1+)

### 功能扩展

#### 数据库支持扩展
- [ ] 实现MySQL适配器
- [ ] 实现PostgreSQL适配器
- [ ] 实现MongoDB适配器（NoSQL）
- [ ] 实现SQL Server适配器
- [ ] 实现Redis适配器
- [ ] 实现ClickHouse适配器
- [ ] 实现达梦数据库支持
- [ ] 实现人大金仓支持

#### 数据可视化（参考Chat2DB Dashboard）
- [ ] 实现Dashboard看板功能
- [ ] 添加数据可视化图表 (Chart.js/D3.js/ECharts)
- [ ] 实现柱状图、折线图、饼图、散点图生成
- [ ] 实现查询结果一键生成图表
- [ ] 添加图表配置界面（选择X轴、Y轴、图表类型）
- [ ] 实现AI自动生成图表功能
- [ ] 添加图表导出功能（PNG/SVG/PDF）
- [ ] 实现Dashboard分享功能
- [ ] 实现ER图生成功能
- [ ] 实现SQL执行计划可视化
- [ ] 添加数据库性能分析仪表板
- [ ] 实现图表定时刷新功能

#### 数据操作增强（参考Chat2DB）
- [ ] 实现表数据直接编辑（双击编辑单元格）
- [ ] 实现表数据批量插入
- [ ] 实现表数据批量更新
- [ ] 实现表数据批量删除
- [ ] 添加数据导入功能（CSV/Excel/JSON）
- [ ] 实现数据迁移工具（跨数据库数据传输）
- [ ] 实现数据库结构同步功能
- [ ] 实现数据对比功能（Compare）

#### 团队协作功能
- [ ] 实现多用户协作功能
- [ ] 实现SQL模板共享
- [ ] 实现查询结果分享链接
- [ ] 实现团队空间管理
- [ ] 实现操作审计日志

### 安全增强
- [ ] 实现OAuth 2.0认证
- [ ] 添加用户权限管理 (RBAC)
- [ ] 实现数据库连接加密
- [ ] 添加审计日志系统
- [ ] 实现敏感数据脱敏

### 性能与可扩展性
- [ ] 实现分布式部署支持
- [ ] 添加负载均衡
- [ ] 实现多AI供应商支持 (Claude, Gemini等)
- [ ] 优化AI提示词工程
- [ ] 实现查询缓存集群

---

## 🎯 当前优先级任务 (本周重点)

### 🔥 高优先级（本周重点）
1. **实现ConnectionManager前端UI** (8.4.1 - NEW)
   - 连接列表展示（参考Chat2DB UI风格）
   - 连接表单（测试/生产环境标签）
   - 测试连接功能集成
   - 连接状态可视化

2. **完成SQL编辑器集成** (5.2.2 - 5.2.9)
   - 集成CodeMirror 6实现语法高亮
   - 实现快捷键和自动格式化
   - 实现"/"唤起AI助手
   - 实现SQL智能补全

3. **实现查询控制台增强** (7.2.1 - 7.2.5 NEW)
   - 多标签页支持（参考Chat2DB）
   - SQL分隔执行
   - 执行计划显示

### ⚡ 中优先级
4. **AI功能增强** (7.1.6 - 7.1.9 NEW)
   - "/" 唤起AI助手
   - SQL Copilot智能提示
   - SQL转自然语言
   
5. **Dashboard看板系统** (v1.1规划)
   - 可视化图表生成
   - AI自动生成报表

6. **单元测试补充** (10.1.2 - 10.1.7)

### 📌 低优先级
7. SSH连接支持 (8.4.9 NEW)
8. 数据迁移工具 (v1.1规划)
9. 主题系统开发
10. 性能优化
11. 文档完善

---

## 📈 项目指标与成功标准

### 功能指标
- ✅ AI生成SQL准确率 > 85%
- ✅ 查询执行成功率 > 95%
- 🔄 支持数据库类型 ≥ 1种 (SQLite完成，目标3种)
- 📋 API端点完成率 100% (18/18已实现)

### 质量指标
- 🔄 测试覆盖率 > 90% (当前约60%)
- ✅ 代码质量无警告 (Clippy/ESLint)
- ✅ 安全漏洞 = 0

### 性能指标
- 🔄 API响应时间 < 200ms
- 🔄 查询执行时间 < 2s
- 📋 并发用户支持 > 100
- 📋 系统可用性 > 99.9%

---

## 📅 更新日志

### 2025-01-XX
- 🔍 **完成Chat2DB功能调研**，详细对比分析Chat2DB功能与UI设计
- 📊 **新增任务**: 多标签页功能细化（12个子任务）、数据编辑功能（4个主任务+20个子任务）、Dashboard看板系统（6个主任务+25个子任务）、数据导入功能（1个主任务+11个子任务）、数据库结构同步（1个主任务+6个子任务）、对话式AI分析（1个主任务+6个子任务）
- 📊 总任务数：124 → 约200+项（细化后）
- 🎯 重点新增功能：
  * 多标签页查询控制台（参考Chat2DB Query Console）
  * 数据编辑功能（双击编辑、批量操作）
  * Dashboard看板系统（数据可视化、AI生成图表）
  * 数据导入功能（CSV/Excel/JSON）
  * 数据库结构同步
  * 对话式AI分析
- 📝 创建调研文档：`.temp/docs/chat2db-research.md`

### 2025-11-14
- 🔍 **对标Chat2DB功能**，补充19项新任务
- 📊 总任务数：105 → 124项
- 🎯 重点新增功能：
  * 查询控制台增强（多标签页、SQL分隔执行）
  * AI功能增强（"/"唤起、Copilot、SQL转自然语言）
  * 连接管理增强（环境标签、SSH支持）
  * Dashboard看板系统（数据可视化、AI生成图表）
  * 表操作增强（AI建表、可视化建表）
  * 数据迁移工具（跨数据库传输）
- 🎨 UI设计参考Chat2DB风格
- ✅ 前端类型检查通过（0 errors）
- 📝 更新项目进度为58% (72/124)

### 2025-11-13
- 📝 重构todolist文档，细化任务至105项
- 📊 更新项目进度为67% (72/105) → 重新计算后58% (72/124)
- 🔍 完成项目代码和文档全面审查
- 📋 明确Phase 1-4各阶段详细任务
- 🎯 设定当前周重点任务和优先级
- ✅ 完成本地存储系统（LocalStorageManager）
- ✅ 完成连接管理API（后端完整）
- ✅ 完成查询历史UI组件

### 2024-05-30 (历史记录)
- ✅ 完成AI服务集成 (OpenAI GPT API)
- ✅ 完成SQL生成、优化和解释功能
- ✅ 完成提示词模板系统
- ✅ 完成API安全防护
- ✅ 完成基础单元测试和集成测试
- ✅ 创建Docker配置
- ✅ 实现DatabaseTree和TableStructureViewer组件

---

## 🔗 相关文档

- 📄 [README.md](../README.md) - 项目概述和功能说明
- 🏗️ [system-design.md](docs/architecture/system-design.md) - 系统架构设计
- 📐 [technical-implementation-plan.md](docs/architecture/technical-implementation-plan.md) - 技术实现方案
- 🗺️ [mvp-roadmap.md](docs/development/mvp-roadmap.md) - MVP开发路线图
- 🧪 [testing-guide.md](docs/development/testing-guide.md) - 测试指南
- 📏 [coding-standards.md](docs/standards/coding-standards.md) - 编码规范

---

**维护说明**: 本文档应每周更新，记录任务完成情况、遇到的问题和下周计划。
