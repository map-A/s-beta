//! 通达信 TQLEX API 客户端封装
//!
//! 所有 F10 数据均通过 POST JSON API 获取：
//! `POST /TQLEX?Entry=CWServ.tdxf10_gg_<page>`
//! 请求体：`{"Params":["<p1>","<p2>",""]}`
//! 响应：`{"ErrorCode":0,"ResultSets":[{"ColName":[...],"Content":[[...]]}]}`
//!
//! ## 并发控制 & 重试
//! - Cookie Store：自动维护 ASPSessionID/LST 会话 cookie，模拟浏览器行为
//! - 全局信号量限制同时发出的 HTTP 请求数（默认 200），防止服务器被压垮
//! - 遇到连接/超时错误自动指数退避重试，最多 3 次（间隔 1s/2s/4s）

use std::sync::Arc;

use crate::error::F10Error;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use tracing::warn;

/// TQLEX API 响应中的单个结果集
#[derive(Debug, Deserialize)]
pub struct ResultSet {
    /// 列名数组，如 ["T001","T002",...]
    #[serde(rename = "ColName")]
    pub col_name: Vec<String>,
    /// 数据行，每行为 JSON Value 数组
    #[serde(rename = "Content")]
    pub content: Vec<Vec<Value>>,
}

/// TQLEX API 响应
#[derive(Debug, Default, Deserialize)]
pub struct TqLexResponse {
    /// 错误码，0 表示成功
    #[serde(rename = "ErrorCode")]
    pub error_code: i64,
    /// 错误消息（可选）
    #[serde(rename = "ErrorMsg")]
    pub error_msg: Option<String>,
    /// 结果集数组
    #[serde(rename = "ResultSets")]
    pub result_sets: Vec<ResultSet>,
}

/// 最大重试次数
const MAX_RETRIES: u32 = 3;
/// 默认全局并发 HTTP 请求限制（高并发以充分利用本地服务器）
const DEFAULT_MAX_CONCURRENT: usize = 200;

/// 通达信 TQLEX HTTP 客户端
///
/// 内置：
/// - **Cookie Store**：自动维护 ASPSessionID/LST 会话 cookie（模拟浏览器行为）
/// - **全局信号量**：限制同时发出的 HTTP 请求数，防止服务器被压垮
/// - **指数退避重试**：遇到连接错误时自动重试，最多 3 次
#[derive(Debug, Clone)]
pub struct TqLexClient {
    /// 服务器基础 URL，如 `http://180.168.205.46:7626`
    pub base_url: String,
    inner: Client,
    /// 全局并发 HTTP 请求信号量
    semaphore: Arc<Semaphore>,
}

impl TqLexClient {
    /// 创建新客户端
    ///
    /// `max_concurrent`：最多同时发出的 HTTP 请求数，建议 10-20，防止服务器过载。
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        Self::with_concurrency(base_url, DEFAULT_MAX_CONCURRENT)
    }

    /// 创建指定并发度的客户端
    #[must_use]
    pub fn with_concurrency(base_url: &str, max_concurrent: usize) -> Self {
        // 启用 cookie_store 模拟浏览器：服务器每次响应会设置 ASPSessionID 和 LST cookies，
        // 浏览器会在后续请求中携带它们，我们也需要这样做。
        let inner = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            // 服务器响应头为 Connection:Close，每次响应后即关闭 TCP 连接。
            // 禁用连接池，避免 reqwest 复用已关闭的连接导致 "error sending request"。
            .pool_max_idle_per_host(0)
            // 启用 cookie jar：自动存储和发送 ASPSessionID/LST session cookies
            .cookie_store(true)
            .build()
            .expect("构建 reqwest 客户端失败");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            inner,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }



    /// 带信号量 + 指数退避重试的核心 HTTP 发送
    async fn send_with_retry(&self, url: &str, body_str: String) -> Result<String, F10Error> {
        let mut last_err: Option<F10Error> = None;
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                // 指数退避：第1次1s、第2次2s、第3次4s
                let delay = Duration::from_millis(1000 * (1u64 << (attempt - 1)));
                warn!(
                    "HTTP 重试第 {attempt} 次（延迟 {}ms）: {url}",
                    delay.as_millis()
                );
                sleep(delay).await;
            }

            // 获取并发许可（在整个请求期间持有，完成后自动释放）
            let _permit = self.semaphore.acquire().await.expect("semaphore 已关闭");

            let result = self
                .inner
                .post(url)
                .header(
                    "content-type",
                    "application/x-www-form-urlencoded; charset=UTF-8",
                )
                .header("x-requested-with", "XMLHttpRequest")
                .header("accept", "text/plain, */*; q=0.01")
                .body(body_str.clone())
                .send()
                .await;

            match result {
                Ok(resp) => match resp.text().await {
                    Ok(text) => return Ok(text),
                    Err(e) => {
                        warn!("读取响应体失败 (第{attempt}次): {e}");
                        last_err = Some(F10Error::Http(e));
                    }
                },
                Err(e) if e.is_connect() || e.is_timeout() || e.is_request() => {
                    // 连接/超时/请求级错误 — 可重试
                    warn!("HTTP 连接错误 (第{attempt}次): {e}");
                    last_err = Some(F10Error::Http(e));
                }
                Err(e) => {
                    // 其他错误（如 URL 格式错误）— 不重试
                    return Err(F10Error::Http(e));
                }
            }
        }
        Err(last_err.expect("至少有一次尝试"))
    }

    /// 发送 POST 请求，支持混合类型参数（字符串+整数）
    ///
    /// # Errors
    ///
    /// 返回 [`F10Error`] 当 HTTP 失败（含重试耗尽）、JSON 解析失败或 API 返回非零错误码。
    pub async fn post_json(
        &self,
        entry: &str,
        params: Vec<Value>,
    ) -> Result<TqLexResponse, F10Error> {
        let url = format!("{}/TQLEX?Entry=CWServ.{}", self.base_url, entry);
        let body = serde_json::json!({ "Params": params });
        let body_str = serde_json::to_string(&body)?;
        let resp = self.send_with_retry(&url, body_str).await?;
        let parsed: TqLexResponse = serde_json::from_str(&resp)?;
        if parsed.error_code != 0 {
            return Err(F10Error::ApiError {
                code: parsed.error_code,
                msg: parsed.error_msg.unwrap_or_default(),
            });
        }
        Ok(parsed)
    }

    /// 发送 POST 请求到指定 TQLEX 端点
    ///
    /// # 参数
    /// - `entry`：端点名，如 `tdxf10_gg_gsgk`
    /// - `params`：JSON Params 数组，如 `["0","600519",""]`
    ///
    /// # Errors
    ///
    /// 返回 [`F10Error`] 当 HTTP 失败（含重试耗尽）、JSON 解析失败或 API 返回非零错误码。
    pub async fn post(&self, entry: &str, params: &[&str]) -> Result<TqLexResponse, F10Error> {
        let url = format!("{}/TQLEX?Entry=CWServ.{}", self.base_url, entry);
        // 构造 JSON 请求体：{"Params":["p1","p2","p3"]}
        let params_json: Vec<Value> = params
            .iter()
            .map(|p| Value::String(p.to_string()))
            .collect();
        let body = serde_json::json!({ "Params": params_json });
        let body_str = serde_json::to_string(&body)?;
        let resp = self.send_with_retry(&url, body_str).await?;
        let parsed: TqLexResponse = serde_json::from_str(&resp)?;
        if parsed.error_code != 0 {
            return Err(F10Error::ApiError {
                code: parsed.error_code,
                msg: parsed.error_msg.unwrap_or_default(),
            });
        }
        Ok(parsed)
    }
}

/// 辅助：从 ResultSet 的指定行列中取字符串值（清理空白）
pub fn get_str(rs: &ResultSet, row: usize, col: &str) -> Option<String> {
    let col_idx = rs.col_name.iter().position(|c| c == col)?;
    let val = rs.content.get(row)?.get(col_idx)?;
    match val {
        Value::String(s) => {
            let s = s.trim().to_string();
            if s.is_empty() || s == "--" {
                None
            } else {
                Some(s)
            }
        }
        Value::Null => None,
        other => Some(other.to_string()),
    }
}

/// 辅助：从 ResultSet 的指定行列中取 f64 值
pub fn get_f64(rs: &ResultSet, row: usize, col: &str) -> Option<f64> {
    let col_idx = rs.col_name.iter().position(|c| c == col)?;
    let val = rs.content.get(row)?.get(col_idx)?;
    match val {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// 辅助：从 ResultSet 的指定行列中取 i32 值
pub fn get_i32(rs: &ResultSet, row: usize, col: &str) -> Option<i32> {
    get_i64(rs, row, col).and_then(|v| i32::try_from(v).ok())
}

/// 辅助：从 ResultSet 的指定行列中取 i8 值
pub fn get_i8(rs: &ResultSet, row: usize, col: &str) -> Option<i8> {
    get_i64(rs, row, col).and_then(|v| i8::try_from(v).ok())
}

/// 辅助：从 ResultSet 的指定行列中取 i64 值
pub fn get_i64(rs: &ResultSet, row: usize, col: &str) -> Option<i64> {
    let col_idx = rs.col_name.iter().position(|c| c == col)?;
    let val = rs.content.get(row)?.get(col_idx)?;
    match val {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}
