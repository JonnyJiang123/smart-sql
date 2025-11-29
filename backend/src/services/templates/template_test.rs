use super::mod::{TemplateManager, TemplateType};

#[test]
fn test_template_manager_creation() {
    // 测试模板管理器创建
    let template_dir = "./src/services/templates";
    let manager = TemplateManager::new(template_dir);
    
    assert!(manager.is_ok());
}

#[test]
fn test_template_manager_get_template() {
    // 测试获取模板
    let template_dir = "./src/services/templates";
    let manager = TemplateManager::new(template_dir).unwrap();
    
    // 获取默认SQL生成模板
    let template = manager.get_template(TemplateType::SqlGeneration, "default");
    assert!(template.is_ok());
    assert!(!template.unwrap().is_empty());
    
    // 尝试获取不存在的模板
    let template = manager.get_template(TemplateType::SqlGeneration, "non_existent");
    assert!(template.is_err());
}

#[test]
fn test_template_manager_add_template() {
    // 测试添加新模板
    let template_dir = "./src/services/templates";
    let mut manager = TemplateManager::new(template_dir).unwrap();
    
    // 添加新模板
    let template_content = "This is a test template";
    let result = manager.add_template(
        TemplateType::SqlGeneration,
        "test_template",
        template_content
    );
    
    assert!(result.is_ok());
    
    // 验证模板是否添加成功
    let template = manager.get_template(TemplateType::SqlGeneration, "test_template");
    assert!(template.is_ok());
    assert_eq!(template.unwrap(), template_content);
}

#[test]
fn test_template_manager_update_template() {
    // 测试更新模板
    let template_dir = "./src/services/templates";
    let mut manager = TemplateManager::new(template_dir).unwrap();
    
    // 先添加一个模板
    let template_name = "update_test_template";
    manager.add_template(
        TemplateType::SqlGeneration,
        template_name,
        "Original content"
    ).unwrap();
    
    // 更新模板
    let new_content = "Updated content";
    let result = manager.update_template(
        TemplateType::SqlGeneration,
        template_name,
        new_content
    );
    
    assert!(result.is_ok());
    
    // 验证更新是否成功
    let template = manager.get_template(TemplateType::SqlGeneration, template_name);
    assert!(template.is_ok());
    assert_eq!(template.unwrap(), new_content);
}

#[test]
fn test_template_manager_delete_template() {
    // 测试删除模板
    let template_dir = "./src/services/templates";
    let mut manager = TemplateManager::new(template_dir).unwrap();
    
    // 先添加一个模板
    let template_name = "delete_test_template";
    manager.add_template(
        TemplateType::SqlGeneration,
        template_name,
        "Content to be deleted"
    ).unwrap();
    
    // 删除模板
    let result = manager.delete_template(TemplateType::SqlGeneration, template_name);
    
    assert!(result.is_ok());
    
    // 验证模板是否已删除
    let template = manager.get_template(TemplateType::SqlGeneration, template_name);
    assert!(template.is_err());
}