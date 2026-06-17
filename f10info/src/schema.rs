//! ClickHouse 建表 DDL — 所有 14 个 F10 页面对应的表
//!
//! 每个页面的 DDL 常量拆分到 `schema/` 子目录的同名文件中，
//! 本文件仅保留 [`init_schema`] 入口函数。

#![allow(clippy::wildcard_imports)]

mod cwfx;
mod fhrz;
mod gbjg;
mod gdyj;
mod gsgk;
mod gszx;
mod hyfx;
mod jyds;
mod jyfx;
mod rdtc;
mod ybpj;
mod zbyz;
mod zlcc;
mod zxts;

use cwfx::*;
use fhrz::*;
use gbjg::*;
use gdyj::*;
use gsgk::*;
use gszx::*;
use hyfx::*;
use jyds::*;
use jyfx::*;
use rdtc::*;
use ybpj::*;
use zbyz::*;
use zlcc::*;
use zxts::*;

use clickhouse::{Client, Row};
use serde::Deserialize;
use tracing::info;

use crate::error::F10Error;

/// 检查 f10 数据库的全部表是否已初始化（快速路径，避免每次启动等待 66 条 DDL）。
///
/// # Errors
///
/// 查询失败时返回 [`F10Error`]。
pub async fn schema_exists(client: &Client) -> Result<bool, F10Error> {
    #[derive(Row, Deserialize)]
    struct CountRow {
        cnt: u64,
    }
    let row = client
        .query(
            "SELECT count() AS cnt FROM system.tables \
             WHERE database = currentDatabase() AND name LIKE 'f10_%'",
        )
        .fetch_one::<CountRow>()
        .await?;
    // f10 库共 66 张表；只要 ≥60 张存在就认为已初始化
    Ok(row.cnt >= 60)
}

/// 初始化所有 ClickHouse 表（幂等，IF NOT EXISTS）
///
/// # Errors
///
/// 建表失败返回 [`F10Error`]。
pub async fn init_schema(client: &Client) -> Result<(), F10Error> {
    let ddls: &[(&str, &str)] = &[
        // ── 步骤 1：最新提示
        ("f10_zxts_overview", CREATE_ZXTS_OVERVIEW),
        ("f10_zxts_kpi", CREATE_ZXTS_KPI),
        ("f10_zxts_concept", CREATE_ZXTS_CONCEPT),
        ("f10_zxts_events", CREATE_ZXTS_EVENTS),
        ("f10_zxts_news", CREATE_ZXTS_NEWS),
        ("f10_zxts_jgdy", CREATE_ZXTS_JGDY),
        ("f10_zxts_hdwd", CREATE_ZXTS_HDWD),
        ("f10_zxts_peer", CREATE_ZXTS_PEER),
        // ── 步骤 2：资金动向
        ("f10_jyds_block_trade", CREATE_JYDS_BLOCK_TRADE),
        ("f10_jyds_margin", CREATE_JYDS_MARGIN),
        ("f10_jyds_moneyflow", CREATE_JYDS_MONEYFLOW),
        ("f10_jyds_dragon_tiger", CREATE_JYDS_DRAGON_TIGER),
        ("f10_jyds_northbound", CREATE_JYDS_NORTHBOUND),
        // ── 步骤 3：基本情况
        ("f10_gsgk_basic", CREATE_GSGK_BASIC),
        ("f10_gsgk_employee", CREATE_GSGK_EMPLOYEE),
        ("f10_gsgk_emp_struct", CREATE_GSGK_EMP_STRUCT),
        ("f10_gsgk_rd", CREATE_GSGK_RD),
        ("f10_gsgk_subsidiary", CREATE_GSGK_SUBSIDIARY),
        // ── 步骤 4：股本结构
        ("f10_gbjg_share_struct", CREATE_GBJG_SHARE_STRUCT),
        ("f10_gbjg_change", CREATE_GBJG_CHANGE),
        ("f10_gbjg_unlock", CREATE_GBJG_UNLOCK),
        ("f10_gbjg_buyback", CREATE_GBJG_BUYBACK),
        // ── 步骤 5：股东研究
        (
            "f10_gdyj_controlling_holder",
            CREATE_GDYJ_CONTROLLING_HOLDER,
        ),
        ("f10_gdyj_holder_count", CREATE_GDYJ_HOLDER_COUNT),
        ("f10_gdyj_industry_holders", CREATE_GDYJ_INDUSTRY_HOLDERS),
        ("f10_gdyj_top10_float", CREATE_GDYJ_TOP10_FLOAT),
        ("f10_gdyj_top10_all", CREATE_GDYJ_TOP10_ALL),
        ("f10_gdyj_hold_change", CREATE_GDYJ_HOLD_CHANGE),
        ("f10_gdyj_inst_trend", CREATE_GDYJ_INST_TREND),
        ("f10_gdyj_inst_summary", CREATE_GDYJ_INST_SUMMARY),
        ("f10_gdyj_inst_detail", CREATE_GDYJ_INST_DETAIL),
        // ── 步骤 6：分红融资
        ("f10_fhrz_dividend", CREATE_FHRZ_DIVIDEND),
        ("f10_fhrz_rights", CREATE_FHRZ_RIGHTS),
        ("f10_fhrz_addissue", CREATE_FHRZ_ADDISSUE),
        // ── 步骤 7：财务分析
        ("f10_cwfx_indicator", CREATE_CWFX_INDICATOR),
        ("f10_cwfx_report", CREATE_CWFX_REPORT),
        ("f10_cwfx_profit", CREATE_CWFX_PROFIT),
        ("f10_cwfx_research", CREATE_CWFX_RESEARCH),
        // ── 步骤 8：经营分析
        ("f10_jyfx_main_biz", CREATE_JYFX_MAIN_BIZ),
        ("f10_jyfx_top5_customer", CREATE_JYFX_TOP5_CUSTOMER),
        ("f10_jyfx_top5_supplier", CREATE_JYFX_TOP5_SUPPLIER),
        ("f10_jyfx_oper_data", CREATE_JYFX_OPER_DATA),
        // ── 步骤 9：资本运作
        ("f10_zbyz_fundraise", CREATE_ZBYZ_FUNDRAISE),
        ("f10_zbyz_violation", CREATE_ZBYZ_VIOLATION),
        ("f10_zbyz_major_event", CREATE_ZBYZ_MAJOR_EVENT),
        ("f10_zbyz_share_transfer", CREATE_ZBYZ_SHARE_TRANSFER),
        ("f10_zbyz_share_control", CREATE_ZBYZ_SHARE_CONTROL),
        // ── 步骤 10：研报评级
        ("f10_ybpj_rating_stat", CREATE_YBPJ_RATING_STAT),
        ("f10_ybpj_forecast", CREATE_YBPJ_FORECAST),
        ("f10_ybpj_report", CREATE_YBPJ_REPORT),
        // ── 步骤 11：热点题材
        ("f10_rdtc_theme", CREATE_RDTC_THEME),
        ("f10_rdtc_event", CREATE_RDTC_EVENT),
        ("f10_rdtc_logic", CREATE_RDTC_LOGIC),
        ("f10_rdtc_concept", CREATE_RDTC_CONCEPT),
        // ── 步骤 12：公司资讯
        ("f10_gszx_news", CREATE_GSZX_NEWS),
        ("f10_gszx_report", CREATE_GSZX_REPORT),
        // ── 步骤 13：主力持仓
        ("f10_zlcc_inst_timeline", CREATE_ZLCC_INST_TIMELINE),
        ("f10_zlcc_inst_by_type", CREATE_ZLCC_INST_BY_TYPE),
        ("f10_zlcc_inst_detail", CREATE_ZLCC_INST_DETAIL),
        // ── 步骤 14：行业分析
        ("f10_hyfx_industry_news", CREATE_HYFX_INDUSTRY_NEWS),
        ("f10_hyfx_industry_report", CREATE_HYFX_INDUSTRY_REPORT),
        ("f10_hyfx_market_rank", CREATE_HYFX_MARKET_RANK),
        ("f10_hyfx_size_rank", CREATE_HYFX_SIZE_RANK),
        ("f10_hyfx_valuation_rank", CREATE_HYFX_VALUATION_RANK),
        ("f10_hyfx_financial_rank", CREATE_HYFX_FINANCIAL_RANK),
        ("f10_hyfx_dividend_rank", CREATE_HYFX_DIVIDEND_RANK),
    ];

    for (name, ddl) in ddls {
        info!("创建表 {name}");
        client.query(ddl).execute().await?;
    }
    info!("所有 {} 张表创建完成", ddls.len());
    Ok(())
}
