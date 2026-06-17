//! 步骤 7：财务分析 (gg_cwfx)
//!
//! 涵盖：
//! - zyzb  主要指标（每报告期 EPS、ROE、毛利率等）
//! - cwbg  财务报告链接（年报 / 季报 PDF）
//! - ylnl  盈利能力（ROE、毛利率、营业利润率、净利率）
//! - cwfx_cbdp  财报点评（机构研究报告标题）

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_i64, get_str, TqLexClient};
use crate::error::F10Error;

// ── 主要指标 ──────────────────────────────────────────────────────────────────

/// 主要财务指标（每报告期），来自 zyzb 接口。
///
/// API ColName: ["T002","mgsy","kfjlr","mgxjll","lrze","jyr","jzzsyl","xsmll",
///              "jlrtbzzl","yysrtb","yyzsrhb","jlrhb","pjjzcsyl","kfjlrhb"]
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CwfxIndicatorRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期，如 "2026-03-31"
    pub report_date: String,
    /// 每股收益 (mgsy)
    pub eps: f64,
    /// 扣非净利润，元 (kfjlr)
    pub non_recurring_profit: f64,
    /// 每股现金流量 (mgxjll)
    pub per_share_cashflow: f64,
    /// 利润总额，元 (lrze)
    pub total_profit: f64,
    /// 净利润，元 (jyr)
    pub net_profit: f64,
    /// 净资产收益率 % (jzzsyl)
    pub roe: f64,
    /// 销售毛利率 % (xsmll)
    pub gross_margin: f64,
    /// 净利润同比 % (jlrtbzzl)
    pub net_profit_yoy: f64,
    /// 营业收入同比 % (yysrtb)
    pub revenue_yoy: f64,
    /// 营业收入环比 % (yyzsrhb)
    pub revenue_qoq: f64,
    /// 净利润环比 % (jlrhb)
    pub net_profit_qoq: f64,
    /// 加权平均净资产收益率 % (pjjzcsyl)
    pub weighted_roe: f64,
    /// 扣非净利润环比 % (kfjlrhb)
    pub non_recurring_profit_qoq: f64,
}

// ── 财务报告链接 ───────────────────────────────────────────────────────────────

/// 财务报告列表（PDF 链接），来自 cwbg 接口。
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CwfxReportRow {
    /// 股票代码
    pub stock_code: String,
    /// 年份，如 "2025"
    pub report_year: String,
    /// 报告类型，如 "年报"、"三季报"
    pub report_period: String,
    /// 报告基准日，如 "2025-12-31"
    pub report_date: String,
    /// PDF 下载链接
    pub report_url: String,
}

// ── 盈利能力 ──────────────────────────────────────────────────────────────────

/// 盈利能力指标（每报告期），来自 ylnl 接口 RS0（公司数据）。
///
/// API `ColName`: ["T002","jzzsyl","xsmll","yylrl","xsjll"]
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CwfxProfitRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期
    pub report_date: String,
    /// 净资产收益率 % (jzzsyl)
    pub roe: f64,
    /// 销售毛利率 % (xsmll)
    pub gross_margin: f64,
    /// 营业利润率 % (yylrl)
    pub op_profit_margin: f64,
    /// 销售净利率 % (xsjll)
    pub net_profit_margin: f64,
}

// ── 财报点评 ──────────────────────────────────────────────────────────────────

/// 机构财报点评研究报告（标题+机构），来自 cwfx_cbdp 接口。
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CwfxResearchRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告标题 (bt)
    pub title: String,
    /// 发布机构 (jg)
    pub institution: String,
    /// 发布日期 (rq)
    pub report_date: String,
    /// 原始记录 ID (`rec_id`)
    pub rec_id: i64,
}

// ── 数据抓取 ──────────────────────────────────────────────────────────────────

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<CwfxIndicatorRow>,
        Vec<CwfxReportRow>,
        Vec<CwfxProfitRow>,
        Vec<CwfxResearchRow>,
    ),
    F10Error,
> {
    // 4 个子请求全部并行发出
    let (zyzb_res, cwbg, ylnl, cbdp) = tokio::join!(
        async { client.post("tdxf10_gg_cwfx", &[code, "zyzb", ""]).await },
        async { client.post("tdxf10_gg_cwfx", &[code, "cwbg", ""]).await },
        async { client.post("tdxf10_gg_cwfx", &[code, "ylnl", ""]).await },
        async { client.post("tdxf10_gg_cwfx_cbdp", &[code, "1"]).await },
    );
    let zyzb = zyzb_res?;
    let cwbg = cwbg.unwrap_or_default();
    let ylnl = ylnl.unwrap_or_default();
    let cbdp = cbdp.unwrap_or_default();

    // 主要指标
    let mut indicators = vec![];
    if let Some(rs) = zyzb.result_sets.first() {
        for i in 0..rs.content.len() {
            indicators.push(CwfxIndicatorRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T002").unwrap_or_default(),
                eps: get_f64(rs, i, "mgsy").unwrap_or(0.0),
                non_recurring_profit: get_f64(rs, i, "kfjlr").unwrap_or(0.0),
                per_share_cashflow: get_f64(rs, i, "mgxjll").unwrap_or(0.0),
                total_profit: get_f64(rs, i, "lrze").unwrap_or(0.0),
                net_profit: get_f64(rs, i, "jyr").unwrap_or(0.0),
                roe: get_f64(rs, i, "jzzsyl").unwrap_or(0.0),
                gross_margin: get_f64(rs, i, "xsmll").unwrap_or(0.0),
                net_profit_yoy: get_f64(rs, i, "jlrtbzzl").unwrap_or(0.0),
                revenue_yoy: get_f64(rs, i, "yysrtb").unwrap_or(0.0),
                revenue_qoq: get_f64(rs, i, "yyzsrhb").unwrap_or(0.0),
                net_profit_qoq: get_f64(rs, i, "jlrhb").unwrap_or(0.0),
                weighted_roe: get_f64(rs, i, "pjjzcsyl").unwrap_or(0.0),
                non_recurring_profit_qoq: get_f64(rs, i, "kfjlrhb").unwrap_or(0.0),
            });
        }
    }

    // 财务报告链接（year 是整数，转为字符串）
    let mut reports = vec![];
    if let Some(rs) = cwbg.result_sets.first() {
        for i in 0..rs.content.len() {
            reports.push(CwfxReportRow {
                stock_code: code.to_string(),
                report_year: get_str(rs, i, "year").unwrap_or_default(),
                report_period: get_str(rs, i, "bgq").unwrap_or_default(),
                report_date: get_str(rs, i, "rq").unwrap_or_default(),
                report_url: get_str(rs, i, "url").unwrap_or_default(),
            });
        }
    }

    // 盈利能力：RS0 = 公司本期数据
    let mut profits = vec![];
    if let Some(rs) = ylnl.result_sets.first() {
        for i in 0..rs.content.len() {
            profits.push(CwfxProfitRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T002").unwrap_or_default(),
                roe: get_f64(rs, i, "jzzsyl").unwrap_or(0.0),
                gross_margin: get_f64(rs, i, "xsmll").unwrap_or(0.0),
                op_profit_margin: get_f64(rs, i, "yylrl").unwrap_or(0.0),
                net_profit_margin: get_f64(rs, i, "xsjll").unwrap_or(0.0),
            });
        }
    }

    // 财报点评研究报告
    let mut research = vec![];
    if let Some(rs) = cbdp.result_sets.first() {
        for i in 0..rs.content.len() {
            research.push(CwfxResearchRow {
                stock_code: code.to_string(),
                title: get_str(rs, i, "bt").unwrap_or_default(),
                institution: get_str(rs, i, "jg").unwrap_or_default(),
                report_date: get_str(rs, i, "rq").unwrap_or_default(),
                rec_id: get_i64(rs, i, "rec_id").unwrap_or(0),
            });
        }
    }

    debug!(
        "cwfx {code}: indicators={} reports={} profits={} research={}",
        indicators.len(),
        reports.len(),
        profits.len(),
        research.len()
    );
    Ok((indicators, reports, profits, research))
}

// ── ClickHouse 写入 ───────────────────────────────────────────────────────────

pub async fn insert_indicators(ch: &Client, rows: &[CwfxIndicatorRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<CwfxIndicatorRow>("f10_cwfx_indicator").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_reports(ch: &Client, rows: &[CwfxReportRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<CwfxReportRow>("f10_cwfx_report").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_profits(ch: &Client, rows: &[CwfxProfitRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<CwfxProfitRow>("f10_cwfx_profit").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_research(ch: &Client, rows: &[CwfxResearchRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<CwfxResearchRow>("f10_cwfx_research").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

// ── 查询示例 ──────────────────────────────────────────────────────────────────

pub async fn query_indicators(ch: &Client, code: &str) -> Result<Vec<CwfxIndicatorRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_cwfx_indicator FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_reports(ch: &Client, code: &str) -> Result<Vec<CwfxReportRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_cwfx_report FINAL WHERE stock_code = ? ORDER BY report_year DESC, report_period DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_profits(ch: &Client, code: &str) -> Result<Vec<CwfxProfitRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_cwfx_profit FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_research(ch: &Client, code: &str) -> Result<Vec<CwfxResearchRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_cwfx_research FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}
