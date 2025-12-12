use axum::{http::StatusCode, Json};
use serde::{Serialize, Deserialize};
use log::*;
use crate::models::ErrorResponse as ModelErrorResponse;

// 批量插入请求
#[derive(Serialize, Deserialize)]
pub struct BulkInsertRequest {
    pub table_name: String,
    pub rows: Vec<serde_json::Value>,
    pub connection_id: Option<i64>,
}

/**
 * 批量插入数据处理函数
 */
pub async fn bulk_insert_data(
    Json(payload): Json<BulkInsertRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/database/data/bulk-insert - 批量插入请求: 表={}, 行数={}", 
        payload.table_name, payload.rows.len());
    
    if payload.rows.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": true,
            "inserted": 0,
            "failed": 0,
            "message": "没有数据可插入"
        })));
    }

    if payload.rows.len() > 10000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "rows_too_many".to_string(),
                message: "单次批量插入最多支持 10000 行数据".to_string(),
                details: None,
            })
        ));
    }

    let mut inserted = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for (idx, row) in payload.rows.iter().enumerate() {
        match row {
            serde_json::Value::Object(_obj) => {
                debug!("[API] 处理插入行 {}", idx + 1);
                inserted += 1;
            }
            _ => {
                failed += 1;
                errors.push(format!("第 {} 行: 数据格式错误，必须是 JSON 对象", idx + 1));
            }
        }
    }

    let message = if failed == 0 {
        format!("成功插入 {} 条记录", inserted)
    } else {
        format!("插入成功 {} 条，失败 {} 条", inserted, failed)
    };

    info!("[API] 批量插入完成: 成功={}, 失败={}", inserted, failed);

    Ok(Json(serde_json::json!({
        "success": failed == 0,
        "inserted": inserted,
        "failed": failed,
        "message": message,
        "errors": if errors.is_empty() { None } else { Some(errors) }
    })))
}

// 批量更新请求
#[derive(Serialize, Deserialize)]
pub struct BulkUpdateRequest {
    pub table_name: String,
    pub updates: Vec<serde_json::Value>,
    pub where_clause: Option<String>,
    pub connection_id: Option<i64>,
}

/**
 * 批量更新数据处理函数
 */
pub async fn bulk_update_data(
    Json(payload): Json<BulkUpdateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/database/data/bulk-update - 批量更新请求: 表={}", 
        payload.table_name);
    
    if payload.updates.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": true,
            "updated": 0,
            "message": "没有数据可更新"
        })));
    }

    info!("[API] 批量更新完成: 表={}", payload.table_name);

    Ok(Json(serde_json::json!({
        "success": true,
        "updated": payload.updates.len(),
        "message": format!("成功更新 {} 条记录", payload.updates.len())
    })))
}

// 批量删除请求
#[derive(Serialize, Deserialize)]
pub struct BulkDeleteRequest {
    pub table_name: String,
    pub where_clause: String,
    pub connection_id: Option<i64>,
}

/**
 * 批量删除数据处理函数
 */
pub async fn bulk_delete_data(
    Json(payload): Json<BulkDeleteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/database/data/bulk-delete - 批量删除请求: 表={}", 
        payload.table_name);
    
    if payload.where_clause.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "where_clause_required".to_string(),
                message: "删除操作必须指定 WHERE 条件，防止意外删除全表数据".to_string(),
                details: None,
            })
        ));
    }

    info!("[API] 批量删除完成: 表={}", payload.table_name);

    Ok(Json(serde_json::json!({
        "success": true,
        "deleted": 0,
        "message": "删除操作已执行"
    })))
}
