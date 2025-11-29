-- 为数据库连接表添加环境标签字段
ALTER TABLE connections ADD COLUMN environment TEXT DEFAULT 'development';

-- 添加索引提升查询性能
CREATE INDEX IF NOT EXISTS idx_connections_environment ON connections(environment);

-- 添加一些示例环境标签数据
UPDATE connections SET environment = 'development' WHERE environment IS NULL;

-- 添加环境标签的约束检查（可选）
-- CREATE TRIGGER validate_environment_tag 
-- BEFORE INSERT ON connections
-- FOR EACH ROW
-- BEGIN
--     SELECT CASE 
--         WHEN NEW.environment NOT IN ('development', 'testing', 'staging', 'production') 
--         THEN RAISE(ABORT, 'Invalid environment tag')
--     END;
-- END;