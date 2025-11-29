-- 智能SQLer本地数据存储Schema
-- 用于保存用户连接配置、查询历史、SQL收藏等本地数据

-- 数据库连接配置表
CREATE TABLE IF NOT EXISTS connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                    -- 连接名称
    db_type TEXT NOT NULL,                 -- 数据库类型: sqlite, mysql, postgresql, etc.
    host TEXT,                             -- 主机地址 (远程数据库)
    port INTEGER,                          -- 端口号
    database_name TEXT,                    -- 数据库名称
    username TEXT,                         -- 用户名
    password TEXT,                         -- 密码 (加密存储)
    file_path TEXT,                        -- 文件路径 (SQLite)
    connection_string TEXT,                -- 完整连接串 (可选)
    is_active INTEGER DEFAULT 0,           -- 是否为当前激活连接
    last_connected_at INTEGER,             -- 最后连接时间戳
    created_at INTEGER NOT NULL,           -- 创建时间戳
    updated_at INTEGER NOT NULL,           -- 更新时间戳
    UNIQUE(name)                           -- 连接名称唯一
);

-- 查询历史表
CREATE TABLE IF NOT EXISTS query_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    connection_id INTEGER,                 -- 关联连接ID
    sql_text TEXT NOT NULL,                -- SQL语句
    executed_at INTEGER NOT NULL,          -- 执行时间戳
    execution_time_ms INTEGER,             -- 执行耗时(毫秒)
    row_count INTEGER,                     -- 返回行数
    is_success INTEGER NOT NULL DEFAULT 1, -- 是否成功
    error_message TEXT,                    -- 错误信息
    is_favorite INTEGER DEFAULT 0,         -- 是否收藏
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

-- 创建索引提升查询性能
CREATE INDEX IF NOT EXISTS idx_query_history_executed_at ON query_history(executed_at DESC);
CREATE INDEX IF NOT EXISTS idx_query_history_connection ON query_history(connection_id);
CREATE INDEX IF NOT EXISTS idx_query_history_favorite ON query_history(is_favorite) WHERE is_favorite = 1;

-- SQL收藏夹表
CREATE TABLE IF NOT EXISTS sql_favorites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                    -- 收藏名称/标签
    sql_text TEXT NOT NULL,                -- SQL语句
    description TEXT,                      -- 描述说明
    category TEXT,                         -- 分类标签
    connection_id INTEGER,                 -- 关联连接ID (可选)
    created_at INTEGER NOT NULL,           -- 创建时间戳
    updated_at INTEGER NOT NULL,           -- 更新时间戳
    usage_count INTEGER DEFAULT 0,         -- 使用次数
    last_used_at INTEGER,                  -- 最后使用时间
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_sql_favorites_category ON sql_favorites(category);
CREATE INDEX IF NOT EXISTS idx_sql_favorites_last_used ON sql_favorites(last_used_at DESC);

-- 应用设置表
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,                  -- 设置键
    value TEXT NOT NULL,                   -- 设置值(JSON)
    updated_at INTEGER NOT NULL            -- 更新时间戳
);

-- 插入默认设置
INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES
    ('theme', '"light"', strftime('%s', 'now')),
    ('default_page_size', '100', strftime('%s', 'now')),
    ('query_timeout', '30', strftime('%s', 'now')),
    ('max_history_items', '1000', strftime('%s', 'now'));
