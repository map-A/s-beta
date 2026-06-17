//! 14 个 F10 页面各自对应一个子模块
//!
//! 每个模块提供：
//! - 数据模型结构体（实现 `clickhouse::Row`）
//! - `fetch(client, code)` — 调用 API 并解析响应
//! - `insert(ck, rows)` — 写入 ClickHouse
//! - `query_*(ck, code)` — 查询示例函数

pub mod cwfx;
pub mod fhrz;
pub mod gbjg;
pub mod gdyj;
pub mod gsgk;
pub mod gszx;
pub mod hyfx;
pub mod jyds;
pub mod jyfx;
pub mod rdtc;
pub mod ybpj;
pub mod zbyz;
pub mod zlcc;
pub mod zxts;
