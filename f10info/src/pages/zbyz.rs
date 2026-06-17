//! 步骤 9：资本运作 (gg_zbyz)
//!
//! 涵盖：募集资金项目投资、违规处理、重大事项、股权转让、实控人股权变更

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_str, TqLexClient};
use crate::error::F10Error;

/// 募集资金项目投资（xmtz 接口）
///
/// API ColName: ["T004","T005","sT006","T008","T010","T011","T012","T014","T015"]
/// T004=序号, T005=项目名称, sT006=项目类型, T008=承诺投资额(万元)
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZbyzFundraiseRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期（年报期），如 "2013-12-31"
    pub report_date: String,
    /// 项目名称 (T005)
    pub project_name: String,
    /// 项目类型 (sT006)，如 "募集资金承诺投资项目"
    pub project_type: String,
    /// 承诺投资额，万元 (T008)
    pub committed_amount: f64,
}

/// 违规处理（wgcl 接口）
///
/// API ColName: ["T003","T004","T007","T008","rec_id","T006","T009"]
/// T003=日期, T004=违规事件, T006=当事人, T009=违规类型
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZbyzViolationRow {
    /// 股票代码
    pub stock_code: String,
    /// 违规日期 (T003)
    pub event_date: String,
    /// 违规事件描述 (T004)
    pub event_type: String,
    /// 当事人 (T006)
    pub party: String,
    /// 违规类型 (T009)，如 "董监高违法违规"
    pub violation_type: String,
}

/// 重大事项（zdsx 接口）
///
/// API ColName: ['ggrq', 'xmxz', 'jyje', 'xmjj', 'gljy']
/// ggrq=公告日期, xmxz=事项性质, jyje=交易金额, xmjj=事项简介, gljy=关联交易
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZbyzMajorEventRow {
    /// 股票代码
    pub stock_code: String,
    /// 公告日期 (ggrq)，如 "20260417"
    pub event_date: String,
    /// 事项性质 (xmxz)，如 "其他事项"
    pub event_type: String,
    /// 交易金额 (jyje)，万元
    pub amount: f64,
    /// 事项简介 (xmjj)
    pub event_content: String,
    /// 关联交易描述 (gljy)
    pub related_party: String,
}

/// 股权转让（gqzr 接口）
///
/// API ColName: ["T002","T004","T016","T006","T005","T007","T008","T010"]
/// T002=完成日, T016=状态, T005=转让股份数, T007=转出方, T008=转入方, T010=持股比%
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZbyzShareTransferRow {
    /// 股票代码
    pub stock_code: String,
    /// 完成日期 (T002)，如 "20191231"
    pub complete_date: String,
    /// 状态 (T016)，如 "已完成"
    pub status: String,
    /// 转让股份数 (T005)
    pub shares_count: f64,
    /// 转出方 (T007)
    pub from_party: String,
    /// 转入方 (T008)
    pub to_party: String,
    /// 转让后持股比例 % (T010)
    pub hold_pct: f64,
}

/// 实控人股权变更（sgjb 接口）
///
/// API ColName: ["rq","T005","T006","T003","T004","T014","T007","bz","jd"]
/// rq=日期, T005=资产名称, T006=资产类型, T003=转入方, T004=转出方, T014=详情, T007=金额, bz=货币, jd=状态
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZbyzShareControlRow {
    /// 股票代码
    pub stock_code: String,
    /// 变更日期 (rq)
    pub change_date: String,
    /// 资产名称 (T005)
    pub asset_name: String,
    /// 资产类型 (T006)，如 "股权"
    pub asset_type: String,
    /// 转入方 (T003)
    pub to_party: String,
    /// 转出方 (T004)
    pub from_party: String,
    /// 金额 (T007)
    pub amount: f64,
    /// 货币 (bz)
    pub currency: String,
    /// 状态 (jd)，如 "完成"
    pub status: String,
    /// 变更详情 (T014)
    pub detail: String,
}

// ── 数据抓取 ──────────────────────────────────────────────────────────────────

/// 抓取资本运作全部数据
pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<ZbyzFundraiseRow>,
        Vec<ZbyzViolationRow>,
        Vec<ZbyzMajorEventRow>,
        Vec<ZbyzShareTransferRow>,
        Vec<ZbyzShareControlRow>,
    ),
    F10Error,
> {
    // 阶段1：获取项目投资日期列表（后续请求依赖此日期）
    let xmtz_dates = client
        .post("tdxf10_gg_comreq", &["xmtz", code])
        .await
        .unwrap_or_default();
    let xmtz_date = xmtz_dates
        .result_sets
        .first()
        .and_then(|rs| rs.content.first())
        .and_then(|row| row.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // 阶段2：4 个独立子请求全部并行发出
    let (wgcl, zdsx, gqzr, sgjb) = tokio::join!(
        async { client.post("tdxf10_gg_zbyz", &["wgcl", code, ""]).await },
        async { client.post("tdxf10_gg_zbyz", &["zdsx", code, ""]).await },
        async { client.post("tdxf10_gg_zbyz", &["gqzr", code, ""]).await },
        async { client.post("tdxf10_gg_zbyz", &["sgjb", code, ""]).await },
    );
    let wgcl = wgcl.unwrap_or_default();
    let zdsx = zdsx.unwrap_or_default();
    let gqzr = gqzr.unwrap_or_default();
    let sgjb = sgjb.unwrap_or_default();

    // 募集资金项目投资（按日期）
    let mut fundraises = vec![];
    if !xmtz_date.is_empty() {
        let xmtz = client
            .post("tdxf10_gg_zbyz", &["xmtz", code, &xmtz_date])
            .await
            .unwrap_or_default();
        if let Some(rs) = xmtz.result_sets.first() {
            for i in 0..rs.content.len() {
                let pname = get_str(rs, i, "T005").unwrap_or_default();
                if pname.is_empty() {
                    continue;
                }
                fundraises.push(ZbyzFundraiseRow {
                    stock_code: code.to_string(),
                    report_date: xmtz_date.clone(),
                    project_name: pname,
                    project_type: get_str(rs, i, "sT006").unwrap_or_default(),
                    committed_amount: get_f64(rs, i, "T008").unwrap_or(0.0),
                });
            }
        }
    }

    // 违规处理
    let mut violations = vec![];
    if let Some(rs) = wgcl.result_sets.first() {
        for i in 0..rs.content.len() {
            violations.push(ZbyzViolationRow {
                stock_code: code.to_string(),
                event_date: get_str(rs, i, "T003").unwrap_or_default(),
                event_type: get_str(rs, i, "T004").unwrap_or_default(),
                party: get_str(rs, i, "T006").unwrap_or_default(),
                violation_type: get_str(rs, i, "T009").unwrap_or_default(),
            });
        }
    }

    // 重大事项（ggrq=日期, xmxz=性质, jyje=金额, xmjj=简介, gljy=关联交易）
    let mut major_events = vec![];
    if let Some(rs) = zdsx.result_sets.first() {
        for i in 0..rs.content.len() {
            major_events.push(ZbyzMajorEventRow {
                stock_code: code.to_string(),
                event_date: get_str(rs, i, "ggrq").unwrap_or_default(),
                event_type: get_str(rs, i, "xmxz").unwrap_or_default(),
                amount: get_f64(rs, i, "jyje").unwrap_or(0.0),
                event_content: get_str(rs, i, "xmjj").unwrap_or_default(),
                related_party: get_str(rs, i, "gljy").unwrap_or_default(),
            });
        }
    }

    // 股权转让
    let mut transfers = vec![];
    if let Some(rs) = gqzr.result_sets.first() {
        for i in 0..rs.content.len() {
            transfers.push(ZbyzShareTransferRow {
                stock_code: code.to_string(),
                complete_date: get_str(rs, i, "T002").unwrap_or_default(),
                status: get_str(rs, i, "T016").unwrap_or_default(),
                shares_count: get_f64(rs, i, "T005").unwrap_or(0.0),
                from_party: get_str(rs, i, "T007").unwrap_or_default(),
                to_party: get_str(rs, i, "T008").unwrap_or_default(),
                hold_pct: get_f64(rs, i, "T010").unwrap_or(0.0),
            });
        }
    }

    // 实控人股权变更
    let mut share_controls = vec![];
    if let Some(rs) = sgjb.result_sets.first() {
        for i in 0..rs.content.len() {
            share_controls.push(ZbyzShareControlRow {
                stock_code: code.to_string(),
                change_date: get_str(rs, i, "rq").unwrap_or_default(),
                asset_name: get_str(rs, i, "T005").unwrap_or_default(),
                asset_type: get_str(rs, i, "T006").unwrap_or_default(),
                to_party: get_str(rs, i, "T003").unwrap_or_default(),
                from_party: get_str(rs, i, "T004").unwrap_or_default(),
                amount: get_f64(rs, i, "T007").unwrap_or(0.0),
                currency: get_str(rs, i, "bz").unwrap_or_default(),
                status: get_str(rs, i, "jd").unwrap_or_default(),
                detail: get_str(rs, i, "T014").unwrap_or_default(),
            });
        }
    }

    debug!(
        "zbyz {code}: fundraises={} violations={} major={} transfers={} share_controls={}",
        fundraises.len(),
        violations.len(),
        major_events.len(),
        transfers.len(),
        share_controls.len()
    );
    Ok((
        fundraises,
        violations,
        major_events,
        transfers,
        share_controls,
    ))
}

// ── ClickHouse 写入 ───────────────────────────────────────────────────────────

/// 写入募集资金项目投资
pub async fn insert_fundraises(ch: &Client, rows: &[ZbyzFundraiseRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZbyzFundraiseRow>("f10_zbyz_fundraise").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 写入违规处理
pub async fn insert_violations(ch: &Client, rows: &[ZbyzViolationRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZbyzViolationRow>("f10_zbyz_violation").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 写入重大事项
pub async fn insert_major_events(ch: &Client, rows: &[ZbyzMajorEventRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<ZbyzMajorEventRow>("f10_zbyz_major_event")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 写入股权转让
pub async fn insert_transfers(ch: &Client, rows: &[ZbyzShareTransferRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<ZbyzShareTransferRow>("f10_zbyz_share_transfer")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 写入实控人股权变更
pub async fn insert_share_controls(
    ch: &Client,
    rows: &[ZbyzShareControlRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<ZbyzShareControlRow>("f10_zbyz_share_control")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

// ── 查询示例 ──────────────────────────────────────────────────────────────────

/// 查询募集资金项目投资
pub async fn query_fundraises(ch: &Client, code: &str) -> Result<Vec<ZbyzFundraiseRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zbyz_fundraise FINAL WHERE stock_code = ? ORDER BY report_date DESC, project_name")
        .bind(code).fetch_all().await?)
}

/// 查询违规处理
pub async fn query_violations(ch: &Client, code: &str) -> Result<Vec<ZbyzViolationRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zbyz_violation FINAL WHERE stock_code = ? ORDER BY event_date DESC")
        .bind(code).fetch_all().await?)
}

/// 查询重大事项
pub async fn query_major_events(
    ch: &Client,
    code: &str,
) -> Result<Vec<ZbyzMajorEventRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zbyz_major_event FINAL WHERE stock_code = ? ORDER BY event_date DESC")
        .bind(code).fetch_all().await?)
}

/// 查询股权转让
pub async fn query_transfers(
    ch: &Client,
    code: &str,
) -> Result<Vec<ZbyzShareTransferRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zbyz_share_transfer FINAL WHERE stock_code = ? ORDER BY complete_date DESC")
        .bind(code).fetch_all().await?)
}

/// 查询实控人股权变更
pub async fn query_share_controls(
    ch: &Client,
    code: &str,
) -> Result<Vec<ZbyzShareControlRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zbyz_share_control FINAL WHERE stock_code = ? ORDER BY change_date DESC")
        .bind(code).fetch_all().await?)
}
