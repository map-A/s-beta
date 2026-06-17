//! F10 模块错误类型

use thiserror::Error;

/// F10 模块统一错误类型
#[derive(Debug, Error)]
pub enum F10Error {
    /// HTTP 请求错误
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON 解析错误
    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),

    /// `ClickHouse` 错误
    #[error("ClickHouse 错误: {0}")]
    ClickHouse(#[from] clickhouse::error::Error),

    /// 数据解析错误（字段缺失、类型不匹配等）
    #[error("数据解析失败: {0}")]
    Parse(String),

    /// API 返回非零错误码
    #[error("API 返回错误码 {code}: {msg}")]
    ApiError {
        /// API 错误码
        code: i64,
        /// 错误描述
        msg: String,
    },
}
