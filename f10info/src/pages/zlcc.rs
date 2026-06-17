//! 步骤 13：主力持仓 (gg_zlcc)
//!
//! 涵盖：机构持股时序汇总、按类型汇总、机构持股明细

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

use crate::client::{get_f64, get_i32, get_str, TqLexClient};
use crate::error::F10Error;

/// 机构持股时序汇总
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZlccInstTimelineRow {
    pub stock_code: String,
    pub report_date: String,
    pub inst_count: i32,
    pub inst_change: f64,
    pub hold_shares: f64,
    pub hold_market_cap: f64,
    pub float_pct: f64,
}

/// 机构持股按类型汇总
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZlccInstByTypeRow {
    pub stock_code: String,
    pub report_date: String,
    pub inst_type_code: String,
    pub inst_type_name: String,
    pub inst_count: i32,
    pub hold_shares: f64,
    pub float_pct: f64,
}

/// 机构持股明细
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZlccInstDetailRow {
    pub stock_code: String,
    pub report_date: String,
    pub inst_name: String,
    pub inst_type: String,
    pub hold_shares: f64,
    pub change_amount: f64,
    pub hold_market_cap: f64,
    pub float_pct: f64,
}

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<ZlccInstTimelineRow>,
        Vec<ZlccInstByTypeRow>,
        Vec<ZlccInstDetailRow>,
    ),
    F10Error,
> {
    let report_date = "20260331";

    // 3 个子请求全部并行发出
    let (timeline, by_type, detail) = tokio::join!(
        async {
            client
                .post("tdxf10_gg_gdyj", &[code, "jgcg", "", "", "1", "1", "20"])
                .await
        },
        async {
            client
                .post(
                    "tdxf10_gg_gdyj",
                    &[code, "jgcgz", "", report_date, "1", "1", "20"],
                )
                .await
        },
        client.post_json(
            "tdxf10_gg_gdyj_jgcgmx",
            vec![
                json!(code),
                json!("000"),
                json!(report_date),
                json!(99i64),
                json!("1"),
                json!("1"),
                json!("30"),
            ],
        ),
    );
    let timeline = timeline.unwrap_or_default();
    let by_type = by_type.unwrap_or_default();
    let detail = detail.unwrap_or_default();

    // 时序汇总 T002=日期,T003=机构数,T004=变化量,T005=持股数,T007=持股市值,T008=占流通A%
    let mut timelines = vec![];
    if let Some(rs) = timeline.result_sets.first() {
        for i in 0..rs.content.len() {
            timelines.push(ZlccInstTimelineRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T002").unwrap_or_default(),
                inst_count: get_i32(rs, i, "T003").unwrap_or(0),
                inst_change: get_f64(rs, i, "T004").unwrap_or(0.0),
                hold_shares: get_f64(rs, i, "T005").unwrap_or(0.0),
                hold_market_cap: get_f64(rs, i, "T007").unwrap_or(0.0),
                float_pct: get_f64(rs, i, "T008").unwrap_or(0.0),
            });
        }
    }

    // 按类型汇总：T003=机构数,T005=持股数,T008=占流通%,T012=类型名,sT012=类型代码
    let mut by_types = vec![];
    if let Some(rs) = by_type.result_sets.first() {
        for i in 0..rs.content.len() {
            by_types.push(ZlccInstByTypeRow {
                stock_code: code.to_string(),
                report_date: report_date.to_string(),
                inst_type_code: get_str(rs, i, "sT012").unwrap_or_default(),
                inst_type_name: get_str(rs, i, "T012").unwrap_or_default(),
                inst_count: get_i32(rs, i, "T003").unwrap_or(0),
                hold_shares: get_f64(rs, i, "T005").unwrap_or(0.0),
                float_pct: get_f64(rs, i, "T008").unwrap_or(0.0),
            });
        }
    }

    // 明细
    let mut details = vec![];
    if let Some(rs) = detail.result_sets.first() {
        for i in 0..rs.content.len() {
            details.push(ZlccInstDetailRow {
                stock_code: code.to_string(),
                report_date: report_date.to_string(),
                inst_name: get_str(rs, i, "T003").unwrap_or_default(),
                inst_type: get_str(rs, i, "T011").unwrap_or_default(),
                hold_shares: get_f64(rs, i, "T004").unwrap_or(0.0),
                change_amount: get_f64(rs, i, "T005").unwrap_or(0.0),
                hold_market_cap: get_f64(rs, i, "T006").unwrap_or(0.0),
                float_pct: get_f64(rs, i, "T008").unwrap_or(0.0),
            });
        }
    }

    debug!(
        "zlcc {code}: timelines={} by_types={} details={}",
        timelines.len(),
        by_types.len(),
        details.len()
    );
    Ok((timelines, by_types, details))
}

pub async fn insert_timelines(ch: &Client, rows: &[ZlccInstTimelineRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<ZlccInstTimelineRow>("f10_zlcc_inst_timeline")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_by_types(ch: &Client, rows: &[ZlccInstByTypeRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<ZlccInstByTypeRow>("f10_zlcc_inst_by_type")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_details(ch: &Client, rows: &[ZlccInstDetailRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<ZlccInstDetailRow>("f10_zlcc_inst_detail")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_timeline(ch: &Client, code: &str) -> Result<Vec<ZlccInstTimelineRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zlcc_inst_timeline FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_by_type(
    ch: &Client,
    code: &str,
    report_date: &str,
) -> Result<Vec<ZlccInstByTypeRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zlcc_inst_by_type FINAL WHERE stock_code = ? AND report_date = ? ORDER BY hold_shares DESC")
        .bind(code).bind(report_date).fetch_all().await?)
}

pub async fn query_details(
    ch: &Client,
    code: &str,
    report_date: &str,
) -> Result<Vec<ZlccInstDetailRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zlcc_inst_detail FINAL WHERE stock_code = ? AND report_date = ? ORDER BY hold_shares DESC")
        .bind(code).bind(report_date).fetch_all().await?)
}
