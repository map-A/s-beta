//! 步骤 5：股东研究 (gg_gdyj)
//!
#![allow(missing_docs, clippy::doc_markdown)]

//! API endpoints（全量）：
//! - tdxf10_gg_gdyj  ["code","kggd",...]      → 控股股东 RS0: T001,T004,T006,T009,kggd,sjkzr,zzkzr
//! - tdxf10_gg_gdyj  ["code","gdrs",...]      → 股东人数 RS0: T002,T003,T004,T005,T007,T012
//! - tdxf10_gg_gdyj  ["code","thygdrs",...]   → 同行业股东人数 RS0: T003,T005,zqdm,zqjc
//! - tdxf10_gg_gdyj  ["code","ltgd",...]      → 十大流通股东 RS1: rq,gd,isbgq,gdid,cgs,lb,xz
//! - tdxf10_gg_gdyj  ["code","sdgdbgq",...]   → 十大全部股东 RS0: gd,rq,isbgq,gdid,cgs,lb,xz,bl
//! - tdxf10_gg_gdyj  ["code","cgbd",...]      → 持股变动 RS0: qs,jz,T005,T006,T007,T008,T009
//! - tdxf10_gg_gdyj  ["code","jgcg",...]      → 机构持股趋势 RS0: T002,T003,T004,T005,T006,T007,T008,T014
//! - tdxf10_gg_comreq ["jgcg","code"]         → 最新报告期列表 T002=yyyymmdd
//! - tdxf10_gg_gdyj  ["code","jgcgz","",date] → 机构持股分类 RS0: T003,T005,T008,T012,sT012
//! - tdxf10_gg_gdyj_jgcgmx ["code","000",date,99] → 机构持股明细 RS0: T003,T011,T004,T005,T006,T008

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

use crate::client::{get_f64, get_i64, get_i8, get_str, TqLexClient};
use crate::error::F10Error;

/// 控股股东
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjControllingHolderRow {
    pub stock_code: String,
    /// T001 报告日
    pub report_date: String,
    /// kggd 控股股东名称
    pub controlling_holder: String,
    /// sjkzr 实际控制人
    pub actual_controller: String,
    /// zzkzr 最终控制人
    pub ultimate_controller: String,
    /// T004 持股比例%
    pub hold_pct: f64,
    /// T006 直接持股%
    pub direct_hold_pct: f64,
    /// T009 股权结构链
    pub equity_chain: String,
}

/// 股东人数时序
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjHolderCountRow {
    pub stock_code: String,
    /// T002 报告日
    pub report_date: String,
    /// T003 股东户数
    pub holder_count: i64,
    /// T004 人均持股（万股）
    pub avg_hold_shares: f64,
    /// T005 环比变化%
    pub change_pct: f64,
    /// T012 本期净增减户数
    pub net_change: i64,
    /// T007 收盘价
    pub price: f64,
}

/// 同行业股东人数对比
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjIndustryHolderRow {
    /// 查询主体股票代码
    pub stock_code: String,
    /// zqdm 同行股票代码
    pub peer_code: String,
    /// zqjc 同行股票简称
    pub peer_name: String,
    /// T003 股东户数
    pub holder_count: i64,
    /// T005 变化%
    pub change_pct: f64,
}

/// 十大流通股东（ltgd RS1）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjTop10FloatRow {
    pub stock_code: String,
    /// rq 报告日
    pub report_date: String,
    /// isbgq 是否报告期末（1=是）
    pub is_report_period: i8,
    /// gd 股东名称
    pub holder_name: String,
    /// gdid 股东ID
    pub holder_id: String,
    /// cgs 持股数量
    pub hold_shares: i64,
    /// lb 股东类型
    pub holder_type: String,
    /// xz 股份性质
    pub share_nature: String,
}

/// 十大全部股东（sdgdbgq RS0）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjTop10AllRow {
    pub stock_code: String,
    /// rq 报告日
    pub report_date: String,
    /// isbgq 是否报告期末（1=是）
    pub is_report_period: i8,
    /// gd 股东名称
    pub holder_name: String,
    /// gdid 股东ID
    pub holder_id: String,
    /// cgs 持股数量
    pub hold_shares: i64,
    /// bl 持股比例%
    pub hold_pct: f64,
    /// lb 股东类型
    pub holder_type: String,
    /// xz 股份性质
    pub share_nature: String,
}

/// 持股变动（cgbd RS0）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjHoldChangeRow {
    pub stock_code: String,
    /// qs 起始日
    pub start_date: String,
    /// jz 截止日
    pub end_date: String,
    /// T005 股东名称
    pub holder_name: String,
    /// T006 区间均价
    pub avg_price: f64,
    /// T007 变动量（正=增持，负=减持）
    pub change_amount: f64,
    /// T008 最新持股总数
    pub total_hold: f64,
    /// T009 事件类型描述
    pub event_type: String,
}

/// 机构持股趋势（jgcg RS0）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjInstTrendRow {
    pub stock_code: String,
    /// T002 报告日
    pub report_date: String,
    /// T003 机构总数
    pub inst_count: i64,
    /// T004 机构数变化
    pub count_change: i64,
    /// T005 持股总量
    pub hold_shares: f64,
    /// T006 持股量变化
    pub shares_change: f64,
    /// T007 持股市值
    pub market_cap: f64,
    /// T008 占流通A%
    pub float_pct: f64,
    /// T014 收盘价
    pub price: f64,
}

/// 机构持股分类汇总（jgcgz RS0）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjInstSummaryRow {
    pub stock_code: String,
    pub report_date: String,
    /// T012 类型名称（如"基金"、"特殊法人"）
    pub inst_type_name: String,
    /// sT012 类型代码（整数）
    pub inst_type_code: i64,
    /// T003 机构数量
    pub inst_count: i64,
    /// T005 持股数量
    pub hold_shares: f64,
    /// T008 占流通A%
    pub float_pct: f64,
}

/// 机构持股明细（jgcgmx RS0）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GdyjInstDetailRow {
    pub stock_code: String,
    pub report_date: String,
    /// T003 机构名称
    pub inst_name: String,
    /// T011 机构类型
    pub inst_type: String,
    /// T004 持股数量
    pub hold_shares: f64,
    /// T005 变化量
    pub change_amount: f64,
    /// T006 持股市值
    pub hold_market_cap: f64,
    /// T008 占流通A%
    pub float_pct: f64,
}

#[allow(clippy::type_complexity)]
pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<GdyjControllingHolderRow>,
        Vec<GdyjHolderCountRow>,
        Vec<GdyjIndustryHolderRow>,
        Vec<GdyjTop10FloatRow>,
        Vec<GdyjTop10AllRow>,
        Vec<GdyjHoldChangeRow>,
        Vec<GdyjInstTrendRow>,
        Vec<GdyjInstSummaryRow>,
        Vec<GdyjInstDetailRow>,
    ),
    F10Error,
> {
    // 阶段1：先获取最新报告期（后续请求依赖此日期）
    let comreq = client
        .post_json("tdxf10_gg_comreq", vec![json!("jgcg"), json!(code)])
        .await
        .unwrap_or_default();
    let latest_date = comreq
        .result_sets
        .first()
        .and_then(|rs| rs.content.first())
        .and_then(|row| row.first())
        .and_then(serde_json::Value::as_i64)
        .map_or_else(|| "20260331".to_string(), |i| i.to_string());

    // 阶段2：其余 9 个子请求全部并行发出
    let (kggd, gdrs, thygd, ltgd, sdgd, cgbd, jgcg, jgcgz, jgcgmx) = tokio::join!(
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "kggd", "", "", "1", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "gdrs", "", "", "1", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "thygdrs", "", "", "1", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "ltgd", "", "", "1", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "sdgdbgq", "", "", "1", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "cgbd", "", "", "1", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "jgcg", "", "", "1", "1", "20"])
                .await
        },
        async {
            let d = latest_date.as_str();
            client
                .post("tdxf10_gg_gdyj", &[code, "jgcgz", "", d, "1", "1", "20"])
                .await
        },
        client.post_json(
            "tdxf10_gg_gdyj_jgcgmx",
            vec![
                json!(code),
                json!("000"),
                json!(latest_date.clone()),
                json!(99i64),
                json!("1"),
                json!("1"),
                json!("30"),
            ],
        ),
    );
    let kggd = kggd.unwrap_or_default();
    let gdrs = gdrs.unwrap_or_default();
    let thygd = thygd.unwrap_or_default();
    let ltgd = ltgd.unwrap_or_default();
    let sdgd = sdgd.unwrap_or_default();
    let cgbd = cgbd.unwrap_or_default();
    let jgcg = jgcg.unwrap_or_default();
    let jgcgz = jgcgz.unwrap_or_default();
    let jgcgmx = jgcgmx.unwrap_or_default();

    // ── 控股股东 ──────────────────────────────────────────────────────────
    let mut controlling = vec![];
    if let Some(rs) = kggd.result_sets.first() {
        for i in 0..rs.content.len() {
            controlling.push(GdyjControllingHolderRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T001").unwrap_or_default(),
                controlling_holder: get_str(rs, i, "kggd").unwrap_or_default(),
                actual_controller: get_str(rs, i, "sjkzr").unwrap_or_default(),
                ultimate_controller: get_str(rs, i, "zzkzr").unwrap_or_default(),
                hold_pct: get_f64(rs, i, "T004").unwrap_or(0.0),
                direct_hold_pct: get_f64(rs, i, "T006").unwrap_or(0.0),
                equity_chain: get_str(rs, i, "T009").unwrap_or_default(),
            });
        }
    }

    // ── 股东人数时序 ────────────────────────────────────────────────────
    let mut holder_counts = vec![];
    if let Some(rs) = gdrs.result_sets.first() {
        for i in 0..rs.content.len() {
            holder_counts.push(GdyjHolderCountRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T002").unwrap_or_default(),
                holder_count: get_i64(rs, i, "T003").unwrap_or(0),
                avg_hold_shares: get_f64(rs, i, "T004").unwrap_or(0.0),
                change_pct: get_f64(rs, i, "T005").unwrap_or(0.0),
                net_change: get_i64(rs, i, "T012").unwrap_or(0),
                price: get_f64(rs, i, "T007").unwrap_or(0.0),
            });
        }
    }

    // ── 同行业股东人数 ─────────────────────────────────────────────────
    let mut industry_holders = vec![];
    if let Some(rs) = thygd.result_sets.first() {
        for i in 0..rs.content.len() {
            industry_holders.push(GdyjIndustryHolderRow {
                stock_code: code.to_string(),
                peer_code: get_str(rs, i, "zqdm").unwrap_or_default(),
                peer_name: get_str(rs, i, "zqjc").unwrap_or_default(),
                holder_count: get_i64(rs, i, "T003").unwrap_or(0),
                change_pct: get_f64(rs, i, "T005").unwrap_or(0.0),
            });
        }
    }

    // ── 十大流通股东（ltgd RS1）────────────────────────────────────────
    let mut top10_float = vec![];
    if let Some(rs) = ltgd.result_sets.get(1) {
        for i in 0..rs.content.len() {
            let isbgq = get_i8(rs, i, "isbgq").unwrap_or(0);
            top10_float.push(GdyjTop10FloatRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "rq").unwrap_or_default(),
                is_report_period: isbgq,
                holder_name: get_str(rs, i, "gd").unwrap_or_default(),
                holder_id: get_str(rs, i, "gdid").unwrap_or_default(),
                hold_shares: get_i64(rs, i, "cgs").unwrap_or(0),
                holder_type: get_str(rs, i, "lb").unwrap_or_default(),
                share_nature: get_str(rs, i, "xz").unwrap_or_default(),
            });
        }
    }

    // ── 十大全部股东（sdgdbgq RS0）──────────────────────────────────────
    let mut top10_all = vec![];
    if let Some(rs) = sdgd.result_sets.first() {
        for i in 0..rs.content.len() {
            let isbgq = get_i8(rs, i, "isbgq").unwrap_or(0);
            top10_all.push(GdyjTop10AllRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "rq").unwrap_or_default(),
                is_report_period: isbgq,
                holder_name: get_str(rs, i, "gd").unwrap_or_default(),
                holder_id: get_str(rs, i, "gdid").unwrap_or_default(),
                hold_shares: get_i64(rs, i, "cgs").unwrap_or(0),
                hold_pct: get_f64(rs, i, "bl").unwrap_or(0.0),
                holder_type: get_str(rs, i, "lb").unwrap_or_default(),
                share_nature: get_str(rs, i, "xz").unwrap_or_default(),
            });
        }
    }

    // ── 持股变动 ────────────────────────────────────────────────────────
    let mut hold_changes = vec![];
    if let Some(rs) = cgbd.result_sets.first() {
        for i in 0..rs.content.len() {
            hold_changes.push(GdyjHoldChangeRow {
                stock_code: code.to_string(),
                start_date: get_str(rs, i, "qs").unwrap_or_default(),
                end_date: get_str(rs, i, "jz").unwrap_or_default(),
                holder_name: get_str(rs, i, "T005").unwrap_or_default(),
                avg_price: get_f64(rs, i, "T006").unwrap_or(0.0),
                change_amount: get_f64(rs, i, "T007").unwrap_or(0.0),
                total_hold: get_f64(rs, i, "T008").unwrap_or(0.0),
                event_type: get_str(rs, i, "T009").unwrap_or_default(),
            });
        }
    }

    // ── 机构持股趋势时序 ─────────────────────────────────────────────────
    let mut inst_trend = vec![];
    if let Some(rs) = jgcg.result_sets.first() {
        for i in 0..rs.content.len() {
            inst_trend.push(GdyjInstTrendRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T002").unwrap_or_default(),
                inst_count: get_i64(rs, i, "T003").unwrap_or(0),
                count_change: get_i64(rs, i, "T004").unwrap_or(0),
                hold_shares: get_f64(rs, i, "T005").unwrap_or(0.0),
                shares_change: get_f64(rs, i, "T006").unwrap_or(0.0),
                market_cap: get_f64(rs, i, "T007").unwrap_or(0.0),
                float_pct: get_f64(rs, i, "T008").unwrap_or(0.0),
                price: get_f64(rs, i, "T014").unwrap_or(0.0),
            });
        }
    }

    // ── 机构持股分类汇总 ─────────────────────────────────────────────────
    // T012=类型名称(str), sT012=类型代码(int), T003=机构数, T005=持股量, T008=占比%
    let mut inst_summaries = vec![];
    if let Some(rs) = jgcgz.result_sets.first() {
        for i in 0..rs.content.len() {
            inst_summaries.push(GdyjInstSummaryRow {
                stock_code: code.to_string(),
                report_date: latest_date.clone(),
                inst_type_name: get_str(rs, i, "T012").unwrap_or_default(),
                inst_type_code: get_i64(rs, i, "sT012").unwrap_or(0),
                inst_count: get_i64(rs, i, "T003").unwrap_or(0),
                hold_shares: get_f64(rs, i, "T005").unwrap_or(0.0),
                float_pct: get_f64(rs, i, "T008").unwrap_or(0.0),
            });
        }
    }

    // ── 机构持股明细 ──────────────────────────────────────────────────────
    // T003=机构名, T011=类型, T004=持股量, T005=变化, T006=市值, T008=占比%
    let mut inst_details = vec![];
    if let Some(rs) = jgcgmx.result_sets.first() {
        for i in 0..rs.content.len() {
            inst_details.push(GdyjInstDetailRow {
                stock_code: code.to_string(),
                report_date: latest_date.clone(),
                inst_name: get_str(rs, i, "T003").unwrap_or_default(),
                inst_type: get_str(rs, i, "T011").unwrap_or_default(),
                hold_shares: get_f64(rs, i, "T004").unwrap_or(0.0),
                change_amount: get_f64(rs, i, "T005").unwrap_or(0.0),
                hold_market_cap: get_f64(rs, i, "T006").unwrap_or(0.0),
                float_pct: get_f64(rs, i, "T008").unwrap_or(0.0),
            });
        }
    }

    debug!("gdyj {code}: ctrl={} counts={} industry={} float={} all={} changes={} trend={} sum={} det={}",
        controlling.len(), holder_counts.len(), industry_holders.len(),
        top10_float.len(), top10_all.len(), hold_changes.len(),
        inst_trend.len(), inst_summaries.len(), inst_details.len());
    Ok((
        controlling,
        holder_counts,
        industry_holders,
        top10_float,
        top10_all,
        hold_changes,
        inst_trend,
        inst_summaries,
        inst_details,
    ))
}

pub async fn insert_controlling(
    ch: &Client,
    rows: &[GdyjControllingHolderRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GdyjControllingHolderRow>("f10_gdyj_controlling_holder")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_holder_counts(
    ch: &Client,
    rows: &[GdyjHolderCountRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GdyjHolderCountRow>("f10_gdyj_holder_count")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_industry_holders(
    ch: &Client,
    rows: &[GdyjIndustryHolderRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GdyjIndustryHolderRow>("f10_gdyj_industry_holders")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_top10_float(ch: &Client, rows: &[GdyjTop10FloatRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GdyjTop10FloatRow>("f10_gdyj_top10_float")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_top10_all(ch: &Client, rows: &[GdyjTop10AllRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GdyjTop10AllRow>("f10_gdyj_top10_all").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_hold_changes(ch: &Client, rows: &[GdyjHoldChangeRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GdyjHoldChangeRow>("f10_gdyj_hold_change")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_inst_trend(ch: &Client, rows: &[GdyjInstTrendRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GdyjInstTrendRow>("f10_gdyj_inst_trend").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_inst_summaries(
    ch: &Client,
    rows: &[GdyjInstSummaryRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GdyjInstSummaryRow>("f10_gdyj_inst_summary")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_inst_details(ch: &Client, rows: &[GdyjInstDetailRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GdyjInstDetailRow>("f10_gdyj_inst_detail")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 查询股东人数历史趋势
pub async fn query_holder_counts(
    ch: &Client,
    code: &str,
) -> Result<Vec<GdyjHolderCountRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gdyj_holder_count FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

/// 查询控股股东
pub async fn query_controlling(
    ch: &Client,
    code: &str,
) -> Result<Vec<GdyjControllingHolderRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gdyj_controlling_holder FINAL WHERE stock_code = ? ORDER BY report_date DESC LIMIT 1")
        .bind(code).fetch_all().await?)
}

/// 查询最新期十大流通股东
pub async fn query_top10_float(
    ch: &Client,
    code: &str,
) -> Result<Vec<GdyjTop10FloatRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gdyj_top10_float FINAL WHERE stock_code = ? AND is_report_period = 1 ORDER BY report_date DESC, hold_shares DESC LIMIT 10")
        .bind(code).fetch_all().await?)
}

/// 查询最新期十大全部股东
pub async fn query_top10_all(ch: &Client, code: &str) -> Result<Vec<GdyjTop10AllRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gdyj_top10_all FINAL WHERE stock_code = ? AND is_report_period = 1 ORDER BY report_date DESC, hold_shares DESC LIMIT 10")
        .bind(code).fetch_all().await?)
}

/// 查询机构持股明细（最新报告期）
pub async fn query_inst_details(
    ch: &Client,
    code: &str,
    report_date: &str,
) -> Result<Vec<GdyjInstDetailRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gdyj_inst_detail FINAL WHERE stock_code = ? AND report_date = ? ORDER BY hold_shares DESC")
        .bind(code).bind(report_date).fetch_all().await?)
}

/// 查询机构持股趋势
pub async fn query_inst_trend(ch: &Client, code: &str) -> Result<Vec<GdyjInstTrendRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gdyj_inst_trend FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}
