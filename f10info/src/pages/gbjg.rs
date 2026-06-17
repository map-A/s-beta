//! 步骤 4：股本结构 (gg_gbjg)
//!
//! 涵盖：股本结构时序、股本变动、限售解禁、股票回购

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_i64, get_str, TqLexClient};
use crate::error::F10Error;

/// 股本结构时序
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GbjgShareStructRow {
    pub stock_code: String,
    pub change_date: String,
    pub total_shares: f64,
    pub a_shares: f64,
    pub b_shares: f64,
    /// 无限售流通A股
    pub float_a: f64,
    /// 国有法人股等限售
    pub restricted_a: f64,
    /// 流通A股总计
    pub tradeable_a: f64,
}

/// 股本变动
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GbjgChangeRow {
    pub stock_code: String,
    pub change_date: String,
    pub total_after: f64,
    /// 变动幅度(%)
    pub change_pct: f64,
    /// 变动类型代码
    pub event_code: String,
}

/// 限售解禁批次
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GbjgUnlockRow {
    pub stock_code: String,
    /// 解禁日期 (yyyymmdd 整数 → string)
    pub unlock_date: String,
    /// 限售批次开始日期
    pub lock_date: String,
    /// 限售类型描述
    pub lock_type: String,
    pub unlock_shares: f64,
    pub unlock_reason: String,
}

/// 股票回购
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GbjgBuybackRow {
    pub stock_code: String,
    pub announce_date: String,
    pub plan_end_date: String,
    /// 计划回购股数
    pub plan_shares: f64,
    /// 计划回购金额
    pub plan_amount: f64,
    /// 已回购股数
    pub done_shares: f64,
    /// 已回购金额占比%
    pub done_pct: f64,
    pub price_min: f64,
    pub price_avg: f64,
}

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<GbjgShareStructRow>,
        Vec<GbjgChangeRow>,
        Vec<GbjgUnlockRow>,
        Vec<GbjgBuybackRow>,
    ),
    F10Error,
> {
    // 4 个子请求全部并行发出
    let (gbjg_res, gbbd, xslt, gphg) = tokio::join!(
        async { client.post("tdxf10_gg_gbjg", &[code, "gbjg"]).await },
        async { client.post("tdxf10_gg_gbjg", &[code, "gbbd"]).await },
        async { client.post("tdxf10_gg_gbjg", &[code, "xslt"]).await },
        async { client.post("tdxf10_gg_gbjg", &[code, "gphg"]).await },
    );
    let gbjg = gbjg_res?;
    let gbbd = gbbd.unwrap_or_default();
    let xslt = xslt.unwrap_or_default();
    let gphg = gphg.unwrap_or_default();

    // 股本结构 T002=日期,T003=总股本,T004=A股,T005=B股,T010=限售,T011=流通A,T019=A总
    let mut structs = vec![];
    if let Some(rs) = gbjg.result_sets.first() {
        for i in 0..rs.content.len() {
            structs.push(GbjgShareStructRow {
                stock_code: code.to_string(),
                change_date: get_str(rs, i, "T002").unwrap_or_default(),
                total_shares: get_f64(rs, i, "T003").unwrap_or(0.0),
                a_shares: get_f64(rs, i, "T004").unwrap_or(0.0),
                b_shares: get_f64(rs, i, "T005").unwrap_or(0.0),
                float_a: get_f64(rs, i, "T009").unwrap_or(0.0),
                restricted_a: get_f64(rs, i, "T010").unwrap_or(0.0),
                tradeable_a: get_f64(rs, i, "T011").unwrap_or(0.0),
            });
        }
    }

    // 股本变动 T002=日期,T003=总股本,T006=变动幅度%,T008=变动类型代码
    let mut changes = vec![];
    if let Some(rs) = gbbd.result_sets.first() {
        for i in 0..rs.content.len() {
            changes.push(GbjgChangeRow {
                stock_code: code.to_string(),
                change_date: get_str(rs, i, "T002").unwrap_or_default(),
                total_after: get_f64(rs, i, "T003").unwrap_or(0.0),
                change_pct: get_f64(rs, i, "T006").unwrap_or(0.0),
                event_code: get_str(rs, i, "T008").unwrap_or_default(),
            });
        }
    }

    // 限售解禁: xslt RS0: T003=批次开始日(int), T011=解禁日(int), T006=限售类型, T004=数量, xsyy=原因
    let mut unlocks = vec![];
    if let Some(rs) = xslt.result_sets.first() {
        for i in 0..rs.content.len() {
            let unlock_date = get_i64(rs, i, "T011")
                .map(|d| d.to_string())
                .or_else(|| get_str(rs, i, "T011"))
                .unwrap_or_default();
            let lock_date = get_i64(rs, i, "T003")
                .map(|d| d.to_string())
                .or_else(|| get_str(rs, i, "T003"))
                .unwrap_or_default();
            unlocks.push(GbjgUnlockRow {
                stock_code: code.to_string(),
                unlock_date,
                lock_date,
                lock_type: get_str(rs, i, "T006").unwrap_or_default(),
                unlock_shares: get_f64(rs, i, "T004").unwrap_or(0.0),
                unlock_reason: get_str(rs, i, "xsyy").unwrap_or_default(),
            });
        }
    }

    // 股票回购: T016=公告日,T003=计划开始,T004=计划结束,T007=计划股数,T006=计划金额,
    //           T010=已回购股数,slbl=已完成%(按股数),T013=最低价,hgjj=均价
    let mut buybacks = vec![];
    if let Some(rs) = gphg.result_sets.first() {
        for i in 0..rs.content.len() {
            buybacks.push(GbjgBuybackRow {
                stock_code: code.to_string(),
                announce_date: get_str(rs, i, "T016").unwrap_or_default(),
                plan_end_date: get_str(rs, i, "T004").unwrap_or_default(),
                plan_shares: get_f64(rs, i, "T007").unwrap_or(0.0),
                plan_amount: get_f64(rs, i, "T006").unwrap_or(0.0),
                done_shares: get_f64(rs, i, "T010").unwrap_or(0.0),
                done_pct: get_f64(rs, i, "slbl").unwrap_or(0.0),
                price_min: get_f64(rs, i, "T013").unwrap_or(0.0),
                price_avg: get_f64(rs, i, "hgjj").unwrap_or(0.0),
            });
        }
    }

    debug!(
        "gbjg {code}: structs={} changes={} unlocks={} buybacks={}",
        structs.len(),
        changes.len(),
        unlocks.len(),
        buybacks.len()
    );
    Ok((structs, changes, unlocks, buybacks))
}

pub async fn insert_share_structs(
    ch: &Client,
    rows: &[GbjgShareStructRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GbjgShareStructRow>("f10_gbjg_share_struct")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_changes(ch: &Client, rows: &[GbjgChangeRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GbjgChangeRow>("f10_gbjg_change").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_unlocks(ch: &Client, rows: &[GbjgUnlockRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GbjgUnlockRow>("f10_gbjg_unlock").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_buybacks(ch: &Client, rows: &[GbjgBuybackRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GbjgBuybackRow>("f10_gbjg_buyback").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_share_structs(
    ch: &Client,
    code: &str,
) -> Result<Vec<GbjgShareStructRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gbjg_share_struct FINAL WHERE stock_code = ? ORDER BY change_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_unlocks(ch: &Client, code: &str) -> Result<Vec<GbjgUnlockRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gbjg_unlock FINAL WHERE stock_code = ? ORDER BY unlock_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_buybacks(ch: &Client, code: &str) -> Result<Vec<GbjgBuybackRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gbjg_buyback FINAL WHERE stock_code = ? ORDER BY announce_date DESC")
        .bind(code).fetch_all().await?)
}
