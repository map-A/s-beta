//! qbrs-f10info CLI 入口
//!
//! 用法：
//! ```shell
//! # 单只股票
//! qbrs-f10info --stock 600519
//!
//! # 批量（从文件）
//! qbrs-f10info --file test/codes_20260506.txt --concurrency 10
//!
//! # 自定义服务器
//! qbrs-f10info --stock 600519 --base-url http://192.168.1.10:7626
//! ```


use anyhow::{bail, Context, Result};
use tracing::info;
use tracing_subscriber::EnvFilter;

use qbrs_f10info::client::TqLexClient;
use qbrs_f10info::runner::{fetch_and_insert_one};
use qbrs_f10info::schema::{init_schema, schema_exists};

/// CLI 配置（通过环境变量或命令行参数传入）
struct Config {
    /// 单只股票代码
    stock: Option<String>,
    /// ClickHouse 连接 URL
    ck_url: String,
    /// ClickHouse 数据库名
    ck_database: String,
    /// ClickHouse 用户名
    ck_user: String,
    /// ClickHouse 密码
    ck_password: String,
    /// 是否只初始化 schema（不抓取数据）
    init_only: bool,
}

impl Config {
    /// 从环境变量 + 命令行参数解析配置
    fn from_env_and_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut stock = None;
        let mut init_only = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--stock" => {
                    i += 1;
                    if i < args.len() {
                        stock = Some(args[i].clone());
                    }
                }
                "--init" => {
                    init_only = true;
                }
                _ => {}
            }
            i += 1;
        }

        Config {
            stock,
            ck_url: std::env::var("CK_URL").unwrap_or_else(|_| "http://localhost:8123".to_string()),
            ck_database: std::env::var("CK_DATABASE").unwrap_or_else(|_| "f10".to_string()),
            ck_user: std::env::var("CK_USER").unwrap_or_else(|_| "default".to_string()),
            ck_password: std::env::var("CK_PASSWORD").unwrap_or_default(),
            init_only,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cfg = Config::from_env_and_args();

    // 构建 ClickHouse 客户端
    let ck = clickhouse::Client::default()
        .with_url(&cfg.ck_url)
        .with_database(&cfg.ck_database)
        .with_user(&cfg.ck_user)
        .with_password(&cfg.ck_password);

    // 初始化 schema（快速路径：若表已存在则跳过 66 条 DDL，节省约 7 分钟）
    if schema_exists(&ck).await.context("检查 schema 状态失败")? {
        info!("schema 已存在，跳过初始化");
    } else {
        info!("首次运行，开始初始化 ClickHouse schema…");
        init_schema(&ck)
            .await
            .context("初始化 ClickHouse schema 失败")?;
    }

    if cfg.init_only {
        info!("--init 模式：schema 初始化完成，退出。");
        return Ok(());
    }

    let api = TqLexClient::new("http://180.168.205.46:7626");

    if let Some(code) = cfg.stock {
        // 单只股票模式
        info!("开始抓取股票 {code}...");
        fetch_and_insert_one(&api, &ck, &code)
            .await
            .context(format!("抓取 {code} 失败"))?;
        info!("股票 {code} 所有页面抓取完成");
    } else {
        bail!(
            "使用方式:\n\
            单只股票：qbrs-f10info --stock 600519\n\
            仅初始化：qbrs-f10info --init"
        );
    }

    Ok(())
}
