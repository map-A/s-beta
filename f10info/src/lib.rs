//! qbrs-f10info：通达信 F10 股票基本面数据抓取模块
//!
//! 通过 POST JSON API 抓取 14 个 F10 页面数据，写入 ClickHouse。
//!
//! # 快速开始
//!
//! ```no_run
//! use qbrs_f10info::client::TqLexClient;
//! use qbrs_f10info::pages::gsgk;
//! use qbrs_data::clickhouse::config::CkConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let api = TqLexClient::new("http://180.168.205.46:7626");
//!     let ck = CkConfig::from_env().build_client();
//!     let rows = gsgk::fetch(&api, "600519").await?;
//!     gsgk::insert(&ck, &rows).await?;
//!     Ok(())
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod error;
pub mod pages;
pub mod runner;
pub mod schema;

pub use error::F10Error;
