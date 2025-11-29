# 测试指南

本文档提供了智能SQLer项目的测试指南，包括如何运行单元测试、集成测试以及检查测试覆盖率。

## 运行单元测试

在backend目录下，使用以下命令运行单元测试：

```bash
cd backend
cargo test
```

要运行特定模块的测试：

```bash
cargo test utils::security
```

要运行被忽略的测试（如需要实际数据库连接的测试）：

```bash
cargo test -- --include-ignored
```

## 运行集成测试

集成测试位于backend/tests目录下，使用以下命令运行：

```bash
cargo test --test api_integration_test
```

## 检查测试覆盖率

要检查测试覆盖率，需要安装cargo-tarpaulin工具：

```bash
cargo install cargo-tarpaulin
```

然后运行覆盖率检查：

```bash
cargo tarpaulin --features coverage --out Html
```

这将生成HTML格式的覆盖率报告，保存在`tarpaulin-report.html`文件中。

## 测试命名规范

- 单元测试文件应命名为`module_name_test.rs`，放在与被测试模块相同的目录下
- 集成测试文件应放在`tests`目录下，命名为`*_test.rs`
- 测试函数应使用`test_`前缀，并描述性地命名测试内容

## 测试最佳实践

1. **隔离测试**：每个测试应该是独立的，不依赖于其他测试的状态
2. **模拟依赖**：对于外部依赖（如数据库、API），使用模拟或mock
3. **测试边界条件**：确保测试覆盖正常情况、边缘情况和错误情况
4. **测试性能**：对于关键路径，添加性能基准测试
5. **保持测试简洁**：每个测试应该只测试一个概念

## 常见问题排查

1. **测试失败**：检查错误消息，确保测试环境配置正确
2. **连接错误**：对于数据库测试，确保数据库服务正在运行且配置正确
3. **API密钥错误**：对于AI服务测试，确保设置了有效的API密钥环境变量