# 智能SQLer测试规范

## 🎯 测试目标

- **测试覆盖率**: 100% (严格要求)
- **执行成功率**: 100% (所有测试必须通过)
- **开发模式**: TDD测试驱动开发
- **验收标准**: 没有失败的测试 = 功能未完成

## 🔧 TDD开发流程

### 1. Red-Green-Refactor 循环

```
编写失败测试 (RED) → 实现最小代码 (GREEN) → 重构优化 (REFACTOR)
       ↑                                                   ↓
       └───────────────────── 重复 ────────────────────────┘
```

**工作流程示例**:

```rust
// Step 1: 编写失败的测试 (RED)
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_adapter_connection() {
        // 这个测试先会失败，因为我们还没有实现
        let adapter = SQLiteAdapter::new(":memory:").await.unwrap();
        assert!(adapter.connect().await.is_ok());
    }
}

// Step 2: 实现最小可工作代码 (GREEN)
impl SQLiteAdapter {
    pub async fn new(path: &str) -> Result<Self, DatabaseError> {
        let pool = sqlx::SqlitePool::connect(path).await?;
        Ok(Self {
            connection: Arc::new(pool),
        })
    }

    pub async fn connect(&self) -> Result<(), DatabaseError> {
        // 最小实现 - 只测试连接是否可用
        sqlx::query("SELECT 1").execute(&*self.connection).await?;
        Ok(())
    }
}

// Step 3: 重构优化 (REFACTOR)
impl SQLiteAdapter {
    // 添加更多功能，保持测试通过
}
```

### 2. 测试优先原则

**规则**:
1. **新功能**: 必须先写测试，后写实现
2. **Bug修复**: 先写复现测试，再修复代码
3. **重构**: 必须有测试保护网
4. **提交代码**: 所有测试必须通过

## 📝 测试分层策略

### 1. 单元测试 (Unit Tests)

**目标**: 测试最小可测试单元（函数、方法）

**Rust单元测试**:

```rust
// src/adapters/sqlite.rs

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tempfile::NamedTempFile;

    /// 创建测试数据库
    async fn create_test_db() -> SQLiteAdapter {
        let temp_file = NamedTempFile::new().unwrap();
        SQLiteAdapter::new(temp_file.path().to_str().unwrap()).await.unwrap()
    }

    #[tokio::test]
    async fn test_empty_database_schema() {
        let db = create_test_db().await;
        let schema = db.get_schema().await.unwrap();

        // 验证空数据库的Schema
        assert!(schema.tables.is_empty());
        assert_eq!(schema.database_type, "SQLite");
    }

    #[tokio::test]
    async fn test_table_creation_and_schema() {
        let db = create_test_db().await;

        // 创建测试表
        db.execute(r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT,
                age INTEGER CHECK(age >= 0),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#).await.unwrap();

        // 验证Schema
        let schema = db.get_schema().await.unwrap();
        assert_eq!(schema.tables.len(), 1);

        let users_table = &schema.tables[0];
        assert_eq!(users_table.name, "users");
        assert_eq!(users_table.columns.len(), 5);

        // 验证列信息
        let id_column = users_table.columns.iter().find(|c| c.name == "id").unwrap();
        assert!(id_column.is_primary_key);
        assert_eq!(id_column.data_type, "INTEGER");
        assert!(!id_column.is_nullable);

        let username_column = users_table.columns.iter().find(|c| c.name == "username").unwrap();
        assert!(!username_column.is_nullable);
    }

    #[tokio::test]
    async fn test_query_execution_and_results() {
        let db = create_test_db().await;

        // 准备测试数据
        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)").await.unwrap();
        db.execute("INSERT INTO users (name, age) VALUES ('张三', 25)").await.unwrap();
        db.execute("INSERT INTO users (name, age) VALUES ('李四', 30)").await.unwrap();

        // 执行查询
        let result = db.execute_query("SELECT * FROM users WHERE age > 20").await.unwrap();

        // 验证结果
        assert_eq!(result.columns.len(), 3); // id, name, age
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][1], "张三"); // name字段
        assert_eq!(result.execution_time > 0, true);
    }

    #[tokio::test]
    async fn test_invalid_sql_handling() {
        let db = create_test_db().await;

        let result = db.execute_query("INVALID SQL COMMAND").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DatabaseError::InvalidSQL(msg) => {
                assert!(!msg.is_empty());
            },
            _ => panic!("应该返回InvalidSQL错误"),
        }
    }
}
```

**Svelte/TypeScript单元测试**:

```typescript
// src/lib/services/__tests__/api-client.test.ts

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { apiClient, ApiError } from '$lib/services/api-client';

describe('ApiClient', () => {
    beforeEach(() => {
        // Setup test environment
        fetchMock.resetMocks();
    });

    afterEach(() => {
        // Cleanup
        vi.clearAllMocks();
    });

    it('should execute query successfully', async () => {
        const mockResult = {
            columns: ['id', 'name'],
            rows: [[1, 'Alice'], [2, 'Bob']],
            executionTime: 100,
            rowCount: 2
        };

        fetchMock.mockResponseOnce(JSON.stringify({
            success: true,
            data: mockResult
        }));

        const result = await apiClient.executeQuery('SELECT * FROM users');

        expect(result.columns).toEqual(['id', 'name']);
        expect(result.rows.length).toBe(2);
        expect(result.executionTime).toBe(100);
    });

    it('should handle query errors', async () => {
        fetchMock.mockResponseOnce(JSON.stringify({
            success: false,
            error: 'Invalid SQL syntax'
        }), { status: 400 });

        await expect(
            apiClient.executeQuery('INVALID SQL')
        ).rejects.toThrow('Invalid SQL syntax');
    });

    it('should handle network errors', async () => {
        fetchMock.mockRejectOnce(new Error('Network error'));

        await expect(
            apiClient.executeQuery('SELECT * FROM users')
        ).rejects.toThrow('Network error');
    });
});
```

### 2. 集成测试 (Integration Tests)

**Rust集成测试**:

```rust
// tests/api_integration_tests.rs

use axum::body::Body;
use axum_test::TestServer;
use smart_sql::app;

#[tokio::test]
async fn test_ai_sql_generation_endpoint() {
    let app = app::create_app().await;
    let server = TestServer::new(app).unwrap();

    // 准备测试数据
    let request_body = serde_json::json!({
        "query": "查询所有年龄大于18岁的用户",
        "database_id": "test_db"
    });

    // 发送请求
    let response = server
        .post("/api/ai/generate-sql")
        .json(&request_body)
        .await;

    // 验证响应
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["success"].as_bool().unwrap_or(false));
    assert!(body["data"]["sql"].as_str().is_some());
    assert!(body["data"]["confidence"].as_f64().unwrap() > 0.8);
}

#[tokio::test]
async fn test_query_execution_endpoint() {
    let app = app::create_app_with_test_db().await;
    let server = TestServer::new(app).unwrap();

    // 创建测试表
    server
        .post("/api/database/execute")
        .json(&serde_json::json!({
            "sql": "CREATE TABLE test_users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)"
        }))
        .await
        .assert_status_ok();

    // 插入测试数据
    server
        .post("/api/database/execute")
        .json(&serde_json::json!({
            "sql": "INSERT INTO test_users (name, age) VALUES ('Alice', 25), ('Bob', 30)"
        }))
        .await
        .assert_status_ok();

    // 执行查询
    let response = server
        .post("/api/database/execute")
        .json(&serde_json::json!({
            "sql": "SELECT * FROM test_users WHERE age > 20"
        }))
        .await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["columns"].as_array().unwrap().len(), 3);
    assert_eq!(body["data"]["rows"].as_array().unwrap().len(), 2);
}
```

**Svelte前端集成测试**:

```typescript
// tests/sql-editor.test.ts

describe('SqlEditor Integration', () => {
    it('should successfully generate and execute SQL', async () => {
        // 挂载组件
        const { container } = render(SqlEditor);

        // 生成SQL
        const aiInput = screen.getByPlaceholderText(/输入自然语言查询/i);
        await userEvent.type(aiInput, '查询所有用户');

        const generateBtn = screen.getByRole('button', { name: /生成SQL/i });
        await userEvent.click(generateBtn);

        // 等待AI返回
        await waitFor(() => {
            expect(screen.getByDisplayValue(/SELECT \* FROM users/i)).toBeInTheDocument();
        });

        // 执行查询
        const executeBtn = screen.getByRole('button', { name: /执行查询/i });
        await userEvent.click(executeBtn);

        // 验证结果
        await waitFor(() => {
            expect(screen.getByText(/查询结果: 10 行/i)).toBeInTheDocument();
        });
    });
});
```

### 3. E2E测试 (End-to-End Tests)

```typescript
// tests/e2e/full-workflow.spec.ts

describe('Full Workflow E2E', () => {
    it('should complete a full user workflow', async () => {
        // 1. 访问应用
        await page.goto('http://localhost:5173');

        // 2. 连接到数据库
        await page.click('button:has-text("连接数据库")');
        await page.fill('input[name="name"]', 'Test DB');
        await page.fill('input[name="path"]', ':memory:');
        await page.click('button:has-text("连接")');

        // 3. 通过AI生成SQL
        await page.fill('textarea[placeholder*="自然语言"]', '查询所有年龄大于18的用户');
        await page.click('button:has-text("AI生成SQL")');

        // 等待SQL生成
        await page.waitForSelector('textarea[value*="SELECT"]');

        // 4. 执行查询
        await page.click('button:has-text("执行查询")');

        // 5. 验证结果
        await page.waitForSelector('table');
        const rows = await page.$$('table tbody tr');
        expect(rows.length).toBeGreaterThan(0);
    });
});
```

## 🎯 核心功能测试重点

### 1. AI功能测试

```rust
#[cfg(test)]
mod ai_functionality_tests {
    use super::*;

    #[tokio::test]
    async fn test_natural_language_to_sql_conversion() {
        let ai_service = setup_ai_service();

        let test_cases = vec![
            ("查询所有用户", "SELECT * FROM users"),
            ("查找年龄大于18岁的用户", "SELECT * FROM users WHERE age > 18"),
            ("统计每个部门的员工数量", "SELECT department, COUNT(*) FROM employees GROUP BY department"),
            ("查询2024年1月的订单", "SELECT * FROM orders WHERE order_date >= '2024-01-01' AND order_date < '2024-02-01'"),
        ];

        for (input, expected_pattern) in test_cases {
            let result = ai_service.generate_sql(input, &test_schema()).await.unwrap();

            // 验证返回的SQL包含预期模式
            assert_sql_contains_pattern(&result.sql, expected_pattern);

            // 验证置信度足够高
            assert!(result.confidence > 0.85, "Confidence too low for input: {}", input);
        }
    }

    #[tokio::test]
    async fn test_sql_injection_prevention() {
        let ai_service = setup_ai_service();

        let malicious_inputs = vec![
            "'; DROP TABLE users; --",
            "1' OR '1'='1",
            "'; INSERT INTO users VALUES('hacker'); --",
            "'; DELETE FROM users; --",
            "'; UPDATE users SET admin = true; --",
        ];

        for malicious_input in malicious_inputs {
            let result = ai_service.generate_sql(malicious_input, &test_schema()).await;

            // AI应该拒绝生成恶意SQL或生成安全的替代方案
            if let Ok(generated) = result {
                let sql_upper = generated.sql.to_uppercase();

                // 验证没有危险操作
                assert!(!sql_upper.contains("DROP"), "Should not contain DROP: {}", generated.sql);
                assert!(!sql_upper.contains("DELETE"), "Should not contain DELETE: {}", generated.sql);
                assert!(!sql_upper.contains("INSERT"), "Should not contain INSERT: {}", generated.sql);
                assert!(!sql_upper.contains("UPDATE"), "Should not contain UPDATE: {}", generated.sql);
            }
        }
    }

    #[tokio::test]
    async fn test_schema_aware_generation() {
        let ai_service = setup_ai_service();
        let schema = test_schema();

        // 使用schema上下文生成SQL
        let result = ai_service
            .generate_sql_with_context("查询用户订单", &schema)
            .await
            .unwrap();

        // 验证生成的SQL使用了正确的表名和列名
        assert!(result.sql.contains("users") || result.sql.contains("orders"));
        assert!(result.confidence > 0.9);
    }
}

fn assert_sql_contains_pattern(sql: &str, pattern: &str) {
    let normalized_sql = sql.replace("\n", " ").replace("\t", " ").to_lowercase();
    let normalized_pattern = pattern.to_lowercase();

    assert!(
        normalized_sql.contains(&normalized_pattern),
        "SQL '{}' should contain pattern '{}'", sql, pattern
    );
}
```

### 2. 数据库操作测试

```rust
#[cfg(test)]
mod database_operation_tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_management() {
        let adapter = SQLiteAdapter::new(":memory:").await.unwrap();

        // 模拟多连接请求
        let tasks = (0..10).map(|_| {
            let adapter_clone = adapter.clone();
            tokio::spawn(async move {
                adapter_clone.execute("SELECT 1").await.unwrap();
            })
        });

        let results = futures::future::join_all(tasks).await;
        for result in results {
            result.unwrap();
        }
    }

    #[tokio::test]
    async fn test_large_result_set_handling() {
        let db = create_test_db().await;

        // 创建表并插入大量数据
        db.execute("CREATE TABLE test_data (id INTEGER PRIMARY KEY, value TEXT)").await.unwrap();

        let mut tx = db.begin_transaction().await.unwrap();
        for i in 0..10000 {
            tx.execute("INSERT INTO test_data (value) VALUES (?)", &[&format!("value-{}", i)])
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();

        // 执行查询
        let result = db.execute_query("SELECT * FROM test_data").await.unwrap();

        // 验证结果
        assert_eq!(result.rows.len(), 10000);
        assert!(result.execution_time < 5000); // 5秒内完成
    }

    #[tokio::test]
    async fn test_concurrent_query_execution() {
        let db = create_test_db().await;

        db.execute("CREATE TABLE counter (value INTEGER)").await.unwrap();
        db.execute("INSERT INTO counter VALUES (0)").await.unwrap();

        // 并发更新
        let tasks = (0..10).map(|_| {
            let db_clone = db.clone();
            tokio::spawn(async move {
                db_clone.execute("UPDATE counter SET value = value + 1").await.unwrap();
            })
        });

        futures::future::join_all(tasks).await;

        let result = db.execute_query("SELECT value FROM counter").await.unwrap();
        assert_eq!(result.rows[0][0], 10);
    }
}
```

### 3. 错误处理测试

```rust
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection_errors() {
        // 测试连接不存在的文件
        let result = SQLiteAdapter::new("/nonexistent/path/db.sqlite").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DatabaseError::ConnectionError(_) => {},
            _ => panic!("应该返回ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_invalid_sql_errors() {
        let db = create_test_db().await;

        // 语法错误
        let result = db.execute_query("SELEC * FROM users").await;
        assert!(result.is_err());

        // 表不存在
        let result = db.execute_query("SELECT * FROM nonexistent_table").await;
        assert!(result.is_err());

        // 列不存在
        db.execute("CREATE TABLE test (id INTEGER)").await.unwrap();
        let result = db.execute_query("SELECT nonexistent_column FROM test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ai_api_errors() {
        let ai_service = AIService::new("invalid_api_key");

        let result = ai_service.generate_sql("测试查询", &test_schema()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AIServiceError::APIError(_) | AIServiceError::AuthenticationError => {
                // 预期的错误类型
            },
            _ => panic!("应该返回API相关错误"),
        }
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let db = create_test_db().await;

        // 创建大量数据的表
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT)").await.unwrap();

        // 插入耗时查询
        db.execute("INSERT INTO test (data) WITH RECURSIVE c(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM c WHERE x < 1000000) SELECT random() FROM c").await.unwrap();

        // 应该能够正确处理长查询
        let result = db.execute_query("SELECT COUNT(*) FROM test").await;
        assert!(result.is_ok());
    }
}
```

## 🚀 测试工具和配置

### 1. Rust测试配置

```toml
# Cargo.toml

[dev-dependencies]
# 测试框架
tokio-test = "0.4"
axum-test = "15.0"
tempfile = "3.8"

# Mock工具
mockall = "0.12"
wiremock = "0.5"
mockito = "1.2"

# 断言增强
pretty_assertions = "1.4"
claims = "0.7"

# 测试覆盖率
cargo-tarpaulin = "0.27"

# 性能测试
criterion = "0.5"

# 属性测试 (Property-based testing)
proptest = "1.4"
quickcheck = "1.0"

# 测试并发
futures = "0.3"
```

### 2. TypeScript测试配置

```json
// package.json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest run --coverage"
  },
  "devDependencies": {
    "@testing-library/svelte": "^4.0.0",
    "@testing-library/jest-dom": "^6.0.0",
    "@testing-library/user-event": "^14.0.0",
    "vitest": "^1.0.0",
    "@vitest/ui": "^1.0.0",
    "@vitest/coverage-v8": "^1.0.0",
    "jsdom": "^23.0.0",
    "msw": "^2.0.0"
  }
}
```

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
    plugins: [sveltekit()],
    test: {
        environment: 'jsdom',
        globals: true,
        setupFiles: ['./tests/setup.ts'],
        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'html'],
            exclude: [
                'node_modules/**',
                'tests/**',
                '**/*.config.{js,ts}',
                '**/*.d.ts'
            ],
            thresholds: {
                lines: 100,
                functions: 100,
                branches: 100,
                statements: 100
            }
        }
    }
});
```

### 3. Playwright E2E配置

```typescript
// playwright.config.ts
import type { PlaywrightTestConfig } from '@playwright/test';

const config: PlaywrightTestConfig = {
    webServer: {
        command: 'npm run build && npm run preview',
        port: 4173
    },
    testDir: 'tests',
    testMatch: '**/*.e2e.ts',
    use: {
        baseURL: 'http://localhost:4173',
        screenshot: 'only-on-failure',
        video: 'retain-on-failure'
    },
    projects: [
        {
            name: 'chromium',
            use: { browserName: 'chromium' }
        }
    ]
};

export default config;
```

## 📋 CI/CD集成

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Run tests
        run: |
          cargo test --all-features --verbose
          cargo clippy --all-targets --all-features -- -D warnings
          cargo fmt -- --check

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir ./coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
          flags: backend

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Run tests
        run: |
          npm test -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info
          flags: frontend

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies
        run: |
          # 安装浏览器
          npx playwright install-deps chromium

      - name: Run E2E tests
        run: |
          npm run test:e2e

      - name: Upload test results
        uses: actions/upload-artifact@v3
        if: failure()
        with:
          name: test-results
          path: test-results/
```

## 🎯 运行时机 (测试触发时机)

1. **开发过程中**:
   - 每次修改代码后立即运行相关测试
   - 使用 `cargo watch` 或 `vitest watch` 模式

2. **提交代码前**:
   - 运行完整测试套件: `cargo test` + `npm test`
   - 检查覆盖率
   - 运行代码格式化

3. **推送代码时**:
   - CI自动运行所有测试
   - 检查代码质量
   - 生成覆盖率报告

4. **创建Pull Request时**:
   - 所有测试必须通过
   - 代码覆盖率 >= 100%
   - 代码审查通过

5. **发布前**:
   - 完整测试套件 (单元 + 集成 + E2E)
   - 性能测试
   - 安全扫描
   - 手动测试验证

记住:**测试不是可选的，是开发的基础。没有通过的测试 = 功能未完成。**
