use mongodb::bson::Document;
use mongodb::bson::Bson;
use serde_json::Value;
use std::collections::HashSet;

/// 危险的MongoDB操作符列表
const DANGEROUS_OPERATORS: &[&str] = &[
    "$where",
    "$eval",
    "$function",
    "$javascript",
    "$mapReduce",
    "$group.$push",
    "$group.$addToSet",
    "$out",
    "$merge",
    "$bucketAuto",
];

/// 将用户输入的 JSON 字符串转换为 BSON Document
pub fn parse_bson_query(query_json_string: &str) -> Result<Document, String> {
    // 1. JSON 格式验证
    let json_value: Value = serde_json::from_str(query_json_string)
        .map_err(|e| format!("输入不是合法的 JSON 格式: {}", e))?;

    // 2. BSON 转换验证 (检查数据类型兼容性)
    let bson_doc = mongodb::bson::to_document(&json_value)
        .map_err(|e| format!("JSON 结构无法转换为 BSON Document: {}", e))?;

    Ok(bson_doc)
}

/// 解析聚合管道
pub fn parse_aggregate_pipeline(pipeline_json_string: &str) -> Result<Vec<Document>, String> {
    // 1. JSON 格式验证
    let json_array: Vec<Value> = serde_json::from_str(pipeline_json_string)
        .map_err(|e| format!("输入不是合法的 JSON 数组格式: {}", e))?;

    // 2. BSON 转换验证 (检查数据类型兼容性)
    let pipeline: Vec<Document> = json_array
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            mongodb::bson::to_document(&value)
                .map_err(|e| format!("管道步骤 {} 无法转换为 BSON Document: {}", index + 1, e))
        })
        .collect::<Result<Vec<Document>, String>>()?;

    Ok(pipeline)
}

/// 检查 BSON Document 中是否包含危险操作符
pub fn contains_dangerous_operators(doc: &Document) -> bool {
    let dangerous_set: HashSet<&str> = DANGEROUS_OPERATORS.iter().cloned().collect();
    
    // 递归检查所有字段，将 Document 包装为 Bson::Document
    check_dangerous_operators_recursive(&Bson::Document(doc.clone()), &dangerous_set)
}

/// 递归检查 BSON Document 中是否包含危险操作符
fn check_dangerous_operators_recursive(value: &Bson, dangerous_operators: &HashSet<&str>) -> bool {
    match value {
        Bson::Document(doc) => {
            // 检查当前文档的所有键
            for key in doc.keys() {
                if dangerous_operators.contains(key.as_str()) {
                    return true;
                }
                // 递归检查值
                if check_dangerous_operators_recursive(doc.get(key).unwrap(), dangerous_operators) {
                    return true;
                }
            }
            false
        }
        Bson::Array(arr) => {
            // 递归检查数组中的所有元素
            for item in arr {
                if check_dangerous_operators_recursive(item, dangerous_operators) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// 过滤 BSON Document 中的危险操作符
pub fn filter_dangerous_operators(doc: &Document) -> Document {
    let dangerous_set: HashSet<&str> = DANGEROUS_OPERATORS.iter().cloned().collect();
    
    // 递归过滤所有字段，将 Document 包装为 Bson::Document
    let filtered_bson = filter_dangerous_operators_recursive(&Bson::Document(doc.clone()), &dangerous_set);
    
    // 将过滤后的 Bson 转换回 Document
    if let Bson::Document(filtered_doc) = filtered_bson {
        filtered_doc
    } else {
        Document::new()
    }
}

/// 递归过滤 BSON Document 中的危险操作符
fn filter_dangerous_operators_recursive(value: &Bson, dangerous_operators: &HashSet<&str>) -> Bson {
    match value {
        Bson::Document(doc) => {
            let mut filtered_doc = Document::new();
            for (key, val) in doc.iter() {
                if !dangerous_operators.contains(key.as_str()) {
                    filtered_doc.insert(key.clone(), filter_dangerous_operators_recursive(val, dangerous_operators));
                }
            }
            Bson::Document(filtered_doc)
        }
        Bson::Array(arr) => {
            let filtered_arr: Vec<Bson> = arr
                .iter()
                .map(|item| filter_dangerous_operators_recursive(item, dangerous_operators))
                .collect();
            Bson::Array(filtered_arr)
        }
        _ => value.clone(),
    }
}

/// 过滤聚合管道中的危险操作符
pub fn filter_aggregate_pipeline(pipeline: &Vec<Document>) -> Vec<Document> {
    pipeline.iter()
        .map(|stage| {
            let mut filtered_stage = Document::new();
            let dangerous_set: HashSet<&str> = DANGEROUS_OPERATORS.iter().cloned().collect();
            for (key, val) in stage.iter() {
                if !dangerous_set.contains(key.as_str()) {
                    filtered_stage.insert(key.clone(), filter_dangerous_operators_recursive(val, &dangerous_set));
                }
            }
            filtered_stage
        })
        .collect()
}

/// 为聚合管道添加或调整limit
pub fn add_or_adjust_limit(pipeline: &Vec<Document>) -> Vec<Document> {
    let mut pipeline = pipeline.clone();
    
    // 检查是否已有$limit阶段
    let has_limit = pipeline.iter().any(|stage| stage.contains_key("$limit"));
    
    if has_limit {
        // 调整现有$limit阶段的值
        for stage in &mut pipeline {
            if let Some(limit_bson) = stage.get("$limit") {
                // 处理不同类型的limit值
                let limit_value = match limit_bson {
                    Bson::Int32(val) => *val,
                    Bson::Int64(val) => *val as i32,
                    Bson::Double(val) => *val as i32,
                    _ => 200,
                };
                
                // 限制为min(1500, 实际limit)
                let adjusted_limit = limit_value.min(1500);
                stage.insert("$limit", Bson::Int32(adjusted_limit));
            }
        }
    } else {
        // 添加默认$limit阶段
        let mut limit_stage = Document::new();
        limit_stage.insert("$limit", Bson::Int32(200));
        pipeline.push(limit_stage);
    }
    
    pipeline
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_bson_query_valid() {
        let valid_query = r#"{ "age": { "$gt": 18 } }"#;
        assert!(parse_bson_query(valid_query).is_ok());
    }
    
    #[test]
    fn test_parse_bson_query_invalid_json() {
        let invalid_query = r#"{ "age": { "$gt": 18 }"#; // 缺少结束括号
        assert!(parse_bson_query(invalid_query).is_err());
    }
    
    #[test]
    fn test_parse_aggregate_pipeline_valid() {
        let valid_pipeline = r#"[{ "$match": { "age": { "$gt": 18 } } }, { "$group": { "_id": "$gender", "count": { "$sum": 1 } } }]"#;
        assert!(parse_aggregate_pipeline(valid_pipeline).is_ok());
    }
    
    #[test]
    fn test_contains_dangerous_operators() {
        let dangerous_query = r#"{ "$where": "this.age > 18" }"#;
        let doc = parse_bson_query(dangerous_query).unwrap();
        assert!(contains_dangerous_operators(&doc));
        
        let safe_query = r#"{ "age": { "$gt": 18 } }"#;
        let doc = parse_bson_query(safe_query).unwrap();
        assert!(!contains_dangerous_operators(&doc));
    }
    
    #[test]
    fn test_filter_dangerous_operators() {
        let dangerous_query = r#"{ "$where": "this.age > 18", "age": { "$gt": 18 } }"#;
        let doc = parse_bson_query(dangerous_query).unwrap();
        let filtered_doc = filter_dangerous_operators(&doc);
        assert!(!filtered_doc.contains_key("$where"));
        assert!(filtered_doc.contains_key("age"));
    }
}
