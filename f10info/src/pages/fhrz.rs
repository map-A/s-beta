//! 步骤 6：分红融资 (`gg_fhrz`)
//!
//! 涵盖：分红历史、配股、增发
//!
//! API 端点：POST tdxf10_gg_fhrz
//!   - params ["code","fh"]  → 分红历史 ColName: ["rq","T003","T004","T006","T026","T021","T023","T036","aT036","glzfl","jdcode"]
//!   - params ["code","pf"]  → 配股      RS0 ColName: ["rq","T005","T006","T011","T012","T015","T017"]
//!   - params ["code","zf"]  → 增发      RS0 ColName: ["T003","T005","T006","T011","T012","T017","T025","T026",...]

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_str, TqLexClient};
use crate::error::F10Error;

/// 分红记录
/// API: params=["code","fh"]
/// rq=报告期, T003=公告日, T004=方案描述, T006=EPS, T026=半年EPS,
/// T021=股权登记日, T023=除息日, T036=方案状态, aT036=状态代码, glzfl=分红/净利润占比%, jdcode=季度代码
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct FhrzDividendRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期
    pub report_date: String,
    /// 公告日
    pub announce_date: String,
    /// 分红方案描述
    pub dividend_plan: String,
    /// 每股收益 EPS (T006)
    pub eps: f64,
    /// 上半年每股收益 (T026)
    pub half_eps: f64,
    /// 股权登记日 (T021)
    pub record_date: String,
    /// 除息日 (T023)
    pub ex_date: String,
    /// 方案状态 (T036)
    pub status: String,
    /// 分红占净利润比例% (glzfl)
    pub payout_ratio: f64,
}

/// 配股记录
/// API: params=["code","pf"]
/// RS0 ColName: ["rq","T005","T006","T011","T012","T015","T017"]
/// rq=报告期, T005=配股价格, T006=实际募集资金, T011=配股比例, T012=配股股数, T015=除权日, T017=缴款日
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct FhrzRightsRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期
    pub report_date: String,
    /// 配股价格 (T005)
    pub price: f64,
    /// 实际募集资金 (T006)
    pub amount_raised: f64,
    /// 配股比例 (T011)
    pub rights_ratio: String,
    /// 配股股数 (T012)
    pub shares: f64,
    /// 除权日 (T015)
    pub ex_date: String,
    /// 缴款日 (T017)
    pub pay_date: String,
}

/// 增发记录
/// API: params=["code","zf"]
/// RS0 ColName: ["T003","T005","T006","T011","T012","T017","T025","T026","T111","T110",...]
/// T003=增发类型, T005=价格, T006=募集资金, T011=增发股数, T012=增发对象, T017=申购日
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct FhrzAddIssueRow {
    /// 股票代码
    pub stock_code: String,
    /// 增发类型 (T003)
    pub issue_type: String,
    /// 价格 (T005)
    pub price: f64,
    /// 募集资金 (T006)
    pub amount_raised: f64,
    /// 增发股数 (T011)
    pub issue_shares: f64,
    /// 增发对象 (T012)
    pub issue_object: String,
    /// 申购日 (T017)
    pub subscribe_date: String,
}

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<FhrzDividendRow>,
        Vec<FhrzRightsRow>,
        Vec<FhrzAddIssueRow>,
    ),
    F10Error,
> {
    // 3 个子请求全部并行发出
    let (fh, pf, zf) = tokio::join!(
        async { client.post("tdxf10_gg_fhrz", &[code, "fh"]).await },
        async { client.post("tdxf10_gg_fhrz", &[code, "pf"]).await },
        async { client.post("tdxf10_gg_fhrz", &[code, "zf"]).await },
    );
    let fh = fh.unwrap_or_default();
    let pf = pf.unwrap_or_default();
    let zf = zf.unwrap_or_default();

    // 分红历史
    // ColName: ["rq","T003","T004","T006","T026","T021","T023","T036","aT036","glzfl","jdcode"]
    let mut dividends = vec![];
    if let Some(rs) = fh.result_sets.first() {
        for i in 0..rs.content.len() {
            dividends.push(FhrzDividendRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "rq").unwrap_or_default(),
                announce_date: get_str(rs, i, "T003").unwrap_or_default(),
                dividend_plan: get_str(rs, i, "T004").unwrap_or_default(),
                eps: get_f64(rs, i, "T006").unwrap_or(0.0),
                half_eps: get_f64(rs, i, "T026").unwrap_or(0.0),
                record_date: get_str(rs, i, "T021").unwrap_or_default(),
                ex_date: get_str(rs, i, "T023").unwrap_or_default(),
                status: get_str(rs, i, "T036").unwrap_or_default(),
                payout_ratio: get_f64(rs, i, "glzfl").unwrap_or(0.0),
            });
        }
    }

    // 配股
    // RS0 ColName: ["rq","T005","T006","T011","T012","T015","T017"]
    let mut rights = vec![];
    if let Some(rs) = pf.result_sets.first() {
        for i in 0..rs.content.len() {
            rights.push(FhrzRightsRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "rq").unwrap_or_default(),
                price: get_f64(rs, i, "T005").unwrap_or(0.0),
                amount_raised: get_f64(rs, i, "T006").unwrap_or(0.0),
                rights_ratio: get_str(rs, i, "T011").unwrap_or_default(),
                shares: get_f64(rs, i, "T012").unwrap_or(0.0),
                ex_date: get_str(rs, i, "T015").unwrap_or_default(),
                pay_date: get_str(rs, i, "T017").unwrap_or_default(),
            });
        }
    }

    // 增发
    // RS0 ColName: ["T003","T005","T006","T011","T012","T017","T025","T026",...]
    let mut addissues = vec![];
    if let Some(rs) = zf.result_sets.first() {
        for i in 0..rs.content.len() {
            addissues.push(FhrzAddIssueRow {
                stock_code: code.to_string(),
                issue_type: get_str(rs, i, "T003").unwrap_or_default(),
                price: get_f64(rs, i, "T005").unwrap_or(0.0),
                amount_raised: get_f64(rs, i, "T006").unwrap_or(0.0),
                issue_shares: get_f64(rs, i, "T011").unwrap_or(0.0),
                issue_object: get_str(rs, i, "T012").unwrap_or_default(),
                subscribe_date: get_str(rs, i, "T017").unwrap_or_default(),
            });
        }
    }

    debug!(
        "fhrz {code}: dividends={} rights={} addissues={}",
        dividends.len(),
        rights.len(),
        addissues.len()
    );
    Ok((dividends, rights, addissues))
}

pub async fn insert_dividends(ch: &Client, rows: &[FhrzDividendRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<FhrzDividendRow>("f10_fhrz_dividend").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_rights(ch: &Client, rows: &[FhrzRightsRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<FhrzRightsRow>("f10_fhrz_rights").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_addissues(ch: &Client, rows: &[FhrzAddIssueRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<FhrzAddIssueRow>("f10_fhrz_addissue").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_dividends(ch: &Client, code: &str) -> Result<Vec<FhrzDividendRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_fhrz_dividend FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_rights(ch: &Client, code: &str) -> Result<Vec<FhrzRightsRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_fhrz_rights FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_addissues(ch: &Client, code: &str) -> Result<Vec<FhrzAddIssueRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_fhrz_addissue FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}
