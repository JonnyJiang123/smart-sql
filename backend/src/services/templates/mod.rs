use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// 模板错误类型
#[derive(Debug)]
pub enum TemplateError {
    NotFound,
    DuplicateName,
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::NotFound => write!(f, "Template not found"),
            TemplateError::DuplicateName => write!(f, "Duplicate template name"),
        }
    }
}

impl std::error::Error for TemplateError {}

// 提示词模板类型
enum TemplateType {
    SqlGeneration,
    SqlExplain,
    SqlOptimize,
}

// 提示词模板结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub variables: Vec<String>,
    pub default_variables: HashMap<String, String>,
}

// 提示词模板管理器
#[derive(Clone)]
pub struct TemplateManager {
    pub templates: HashMap<String, PromptTemplate>,
    pub default_templates: HashMap<String, String>,
}

impl TemplateManager {
    // 创建新的模板管理器实例
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            default_templates: HashMap::new(),
        };
        
        // 初始化默认模板
        manager.initialize_default_templates();
        
        manager
    }
    
    // 初始化默认提示词模板
    fn initialize_default_templates(&mut self) {
        // SQL生成模板
        self.add_template(PromptTemplate {
            template_id: "sql_generation_default".to_string(),
            name: "默认SQL生成模板".to_string(),
            description: "用于从自然语言生成SQL查询的标准模板".to_string(),
            content: include_str!("sql_generation_default.txt").to_string(),
            variables: vec!["database_type".to_string(), "database_schema".to_string()],
            default_variables: HashMap::from([
                ("database_type".to_string(), "通用SQL".to_string()),
            ]),
        });
        
        // SQL解释模板
        self.add_template(PromptTemplate {
            template_id: "sql_explain_default".to_string(),
            name: "默认SQL解释模板".to_string(),
            description: "用于解释SQL查询含义的标准模板".to_string(),
            content: include_str!("sql_explain_default.txt").to_string(),
            variables: vec!["database_type".to_string()],
            default_variables: HashMap::from([
                ("database_type".to_string(), "通用SQL".to_string()),
            ]),
        });
        
        // SQL优化模板
        self.add_template(PromptTemplate {
            template_id: "sql_optimize_default".to_string(),
            name: "默认SQL优化模板".to_string(),
            description: "用于优化SQL查询的标准模板".to_string(),
            content: include_str!("sql_optimize_default.txt").to_string(),
            variables: vec!["database_type".to_string()],
            default_variables: HashMap::from([
                ("database_type".to_string(), "通用SQL".to_string()),
            ]),
        });
        
        // 设置默认模板映射
        self.default_templates.insert("sql_generation".to_string(), "sql_generation_default".to_string());
        self.default_templates.insert("sql_explain".to_string(), "sql_explain_default".to_string());
        self.default_templates.insert("sql_optimize".to_string(), "sql_optimize_default".to_string());
    }
    
    // 添加模板
    pub fn add_template(&mut self, template: PromptTemplate) -> Result<(), TemplateError> {
        // 检查是否存在同名模板
        if self.templates.iter().any(|(_, t)| t.name == template.name && t.template_id != template.template_id) {
            return Err(TemplateError::DuplicateName);
        }
        
        self.templates.insert(template.template_id.clone(), template);
        Ok(())
    }
    
    // 更新模板
    pub fn update_template(&mut self, template: PromptTemplate) -> Result<(), TemplateError> {
        // 检查模板是否存在
        if !self.templates.contains_key(&template.template_id) {
            return Err(TemplateError::NotFound);
        }
        
        // 检查是否存在同名模板（排除当前模板）
        if self.templates.iter().any(|(_, t)| 
            t.name == template.name && t.template_id != template.template_id
        ) {
            return Err(TemplateError::DuplicateName);
        }
        
        self.templates.insert(template.template_id.clone(), template);
        Ok(())
    }
    
    // 删除模板
    pub fn delete_template(&mut self, template_id: &str) -> Result<(), TemplateError> {
        if self.templates.remove(template_id).is_some() {
            // 如果删除的是默认模板，清除默认设置
            let mut keys_to_remove = Vec::new();
            for (key, id) in &self.default_templates {
                if id == template_id {
                    keys_to_remove.push(key.clone());
                }
            }
            
            for key in keys_to_remove {
                self.default_templates.remove(&key);
            }
            
            Ok(())
        } else {
            Err(TemplateError::NotFound)
        }
    }
    
    // 设置默认模板
    pub fn set_default_template(&mut self, template_type: &str, template_id: &str) {
        self.default_templates.insert(template_type.to_string(), template_id.to_string());
    }
    
    // 获取模板
    pub fn get_template(&self, template_id: &str) -> Option<&PromptTemplate> {
        self.templates.get(template_id)
    }
    
    // 获取所有可用模板
    pub fn get_available_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }
    
    // 获取默认模板
    pub fn get_default_template(&self, template_type: &str) -> Option<&PromptTemplate> {
        if let Some(default_id) = self.default_templates.get(template_type) {
            self.get_template(default_id)
        } else {
            None
        }
    }
    
    // 渲染模板，替换变量
    pub fn render_template(&self, template_id: &str, variables: &HashMap<String, String>) -> Result<String, String> {
        // 获取模板
        let template = match self.get_template(template_id) {
            Some(t) => t,
            None => return Err(format!("模板 {} 不存在", template_id)),
        };
        
        // 渲染模板
        self.render(&template.content, variables, &template.default_variables)
    }
    
    // 渲染默认模板
    pub fn render_default_template(&self, template_type: &str, variables: &HashMap<String, String>) -> Result<String, String> {
        // 获取默认模板ID
        let default_id = match self.default_templates.get(template_type) {
            Some(id) => id,
            None => return Err(format!("未找到类型 {} 的默认模板", template_type)),
        };
        
        // 获取模板
        let template = match self.get_template(default_id) {
            Some(t) => t,
            None => return Err(format!("默认模板 {} 不存在", default_id)),
        };
        
        // 渲染模板
        self.render(&template.content, variables, &template.default_variables)
    }
    
    // 渲染内容中的变量
    fn render(&self, content: &str, variables: &HashMap<String, String>, default_variables: &HashMap<String, String>) -> Result<String, String> {
        let mut result = content.to_string();
        
        // 处理所有模板变量
        for (var_name, default_value) in default_variables {
            let placeholder = format!("{{{{{}}}}}", var_name);
            
            // 优先使用提供的变量值，如果没有则使用默认值
            let value = variables.get(var_name).unwrap_or(default_value);
            
            result = result.replace(&placeholder, value);
        }
        
        // 检查是否还有未替换的变量
        if let Some(unreplaced) = result.find("{{") {
            if let Some(end) = result[unreplaced..].find("}}") {
                let var_name = &result[unreplaced + 2..unreplaced + end];
                return Err(format!("缺少必要变量: {}", var_name));
            }
        }
        
        Ok(result)
    }
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_manager() {
        let manager = TemplateManager::new();
        
        // 测试获取默认模板
        let sql_gen_template = manager.get_default_template("sql_generation");
        assert!(sql_gen_template.is_some());
        
        // 测试渲染模板
        let mut variables = HashMap::new();
        variables.insert("database_type".to_string(), "PostgreSQL".to_string());
        
        let rendered = manager.render_default_template("sql_generation", &variables);
        assert!(rendered.is_ok());
        assert!(rendered.unwrap().contains("PostgreSQL"));
    }
}