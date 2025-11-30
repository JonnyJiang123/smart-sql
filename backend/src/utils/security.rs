use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;

// SQL注入检测结果
enum SqlInjectionResult {
    Safe,
    Unsafe(String),
}

// SQL注入保护工具
pub struct SqlInjectionProtection;

impl SqlInjectionProtection {
    // 检测SQL注入攻击
    pub fn detect_injection(sql: &str) -> Result<(), String> {
        let sql_lower = sql.to_lowercase();
        
        // 检查常见的SQL注入模式
        match Self::check_dangerous_patterns(&sql_lower) {
            SqlInjectionResult::Unsafe(reason) => Err(reason),
            SqlInjectionResult::Safe => Ok(()),
        }
    }
    
    // 检查危险的SQL模式
    fn check_dangerous_patterns(sql: &str) -> SqlInjectionResult {
        lazy_static! {
            // 危险的SQL关键字组合
            static ref DANGEROUS_PATTERNS: Vec<(Regex, &'static str)> = vec![
                // UNION + SELECT 注入（更广泛的检测）
                (Regex::new(r"(?i)(;|--|\||\s)union\s+select").unwrap(), "UNION SELECT注入尝试"),
                // DROP TABLE 操作
                (Regex::new(r"(?i)drop\s+table\s+").unwrap(), "DROP TABLE操作不允许"),
                // ALTER TABLE 操作
                (Regex::new(r"(?i)alter\s+table\s+").unwrap(), "ALTER TABLE操作不允许"),
                // CREATE/DROP DATABASE
                (Regex::new(r"(?i)(create|drop)\s+database").unwrap(), "数据库操作不允许"),
                // 多语句执行（排除单个语句末尾的分号）
                (Regex::new(r";\s*\w+").unwrap(), "多语句执行不允许"),
                // 危险函数调用
                (Regex::new(r"(?i)exec\s*\(|sp_executesql|xp_cmdshell").unwrap(), "危险函数调用不允许"),
                // COMMENT注入
                (Regex::new(r"(?i)(\s|^)(--|#|/\*).*(\*/|$)").unwrap(), "注释注入尝试"),
            ];
        }
        
        // 检查是否包含危险模式
        for (pattern, reason) in &*DANGEROUS_PATTERNS {
            if pattern.is_match(sql) {
                return SqlInjectionResult::Unsafe(reason.to_string());
            }
        }
        
        // 检查常见的SQL元字符组合
        if Self::contains_dangerous_metacharacters(sql) {
            return SqlInjectionResult::Unsafe("检测到可疑的SQL元字符组合".to_string());
        }
        
        SqlInjectionResult::Safe
    }
    
    // 检查危险的元字符组合
    fn contains_dangerous_metacharacters(sql: &str) -> bool {
        lazy_static! {
            static ref DANGEROUS_METACHAR_COMBINATIONS: HashSet<&'static str> = {
                let mut set = HashSet::new();
                // 添加一些常见的可疑组合
                set.insert("' or ");
                set.insert(" or '");
                set.insert("'--");
                set.insert("' /*");
                set.insert("*/ '");
                set.insert("'= '");
                set.insert("'like '");
                set.insert("1=1");
                set.insert("' union ");
                set.insert("' and ");
                set.insert(" and '");
                set.insert(") or (");
                set.insert("' or ''='");
                set.insert("' or 1=1");
                set.insert(") or 1=1--");
                set
            };
        }
        
        for &comb in DANGEROUS_METACHAR_COMBINATIONS.iter() {
            if sql.contains(comb) {
                return true;
            }
        }
        
        false
    }
    
    // 清理SQL输入，移除潜在的危险字符
    #[allow(dead_code)]
    pub fn sanitize_sql(sql: &str) -> String {
        // 这是一个基本的清理实现，实际应用中可能需要更复杂的规则
        // 注意：最好的防护是使用参数化查询，而不是字符串清理
        let mut sanitized = sql.to_string();
        
        // 移除注释
        if let Some(index) = sanitized.find("--") {
            sanitized.truncate(index);
        }
        
        // 移除多语句分隔符
        sanitized = sanitized.replace(";", "");
        
        // 移除危险的函数
        lazy_static! {
            static ref DANGEROUS_FUNCTIONS: Vec<&'static str> = vec![
                "xp_cmdshell", "exec", "sp_executesql", "bulk insert",
                "openrowset", "opendatasource", "execute"
            ];
        }
        
        for &func in &*DANGEROUS_FUNCTIONS {
            sanitized = sanitized.replace(func, "");
            sanitized = sanitized.replace(&func.to_uppercase(), "");
        }
        
        sanitized.trim().to_string()
    }
}

// 请求速率限制器
#[allow(dead_code)]
pub struct RateLimiter {
    // 在实际应用中，这里应该有更复杂的数据结构来存储请求记录
    // 比如使用Redis或内存中的LRU缓存
}

impl RateLimiter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
    
    #[allow(dead_code)]
    pub async fn check_rate_limit(&self, _ip_address: &str) -> Result<(), String> {
        // 简单的速率限制实现
        // 实际应用中应该检查单位时间内的请求次数
        // 这里只是一个示例实现，总是允许请求
        Ok(())
    }
}

// 为 RateLimiter 添加 Default 实现
impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// API密钥验证
#[allow(dead_code)]
pub fn validate_api_key(api_key: &str) -> bool {
    // 简单的API密钥验证
    // 实际应用中应该从安全的存储中获取有效的API密钥
    !api_key.is_empty() && api_key.len() >= 16
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_injection() {
        // 安全的SQL
        assert!(SqlInjectionProtection::detect_injection("SELECT * FROM users WHERE id = 1").is_ok());
        
        // 不安全的SQL
        assert!(SqlInjectionProtection::detect_injection("SELECT * FROM users WHERE id = 1 OR 1=1").is_err());
        assert!(SqlInjectionProtection::detect_injection("SELECT * FROM users WHERE username = '' OR ''='").is_err());
        assert!(SqlInjectionProtection::detect_injection("SELECT * FROM users; DROP TABLE users").is_err());
    }
    
    #[test]
    fn test_sanitize_sql() {
        let sql = "SELECT * FROM users; -- 注释部分\nDROP TABLE test";
        let sanitized = SqlInjectionProtection::sanitize_sql(sql);
        assert!(!sanitized.contains(";"));
        assert!(!sanitized.contains("DROP TABLE"));
    }
}