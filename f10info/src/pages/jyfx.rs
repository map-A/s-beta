//! 步骤 8：经营分析 (gg_jyfx)
//!
#![allow(missing_docs, clippy::doc_markdown)]

//! 涵盖：
//! - zygc      主营构成明细（按产品/地区/销售模式）
//! - qwm       前五名客户（年报期）
//! - qwmgys    前五名供应商（年报期）
//! - jysj      经营数据（KV 格式，多维度经营指标）

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_str, TqLexClient};
use crate::error::F10Error;

// ── 主营构成明细 ───────────────────────────────────────────────────────────────

/// 主营构成明细（按产品/地区/销售模式），来自 zygc 接口。
///
/// API ColName: ["N000","N001","N002","N003","N004","N005","N006","N007","N008","N009"]
/// N000=分类方式, N001=类型码, N002=子类名, N003=收入, N004=收入占比,
/// N005=利润, N006=利润占比, N007=毛利率（部分公司有）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JyfxMainBizRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期，如 "20260331"
    pub report_date: String,
    /// 分类方式，如 "按产品(项目)"
    pub category_type: String,
    /// 类型编码 (N001)
    pub category_code: String,
    /// 子类名称 (N002)，如 "茅台酒"
    pub category_name: String,
    /// 收入，元 (N003)
    pub revenue: f64,
    /// 收入占比 % (N004)
    pub revenue_pct: f64,
    /// 利润，元 (N005)，未披露为 0
    pub profit: f64,
    /// 毛利率 % (N007)，未披露为 0
    pub gross_margin: f64,
}

// ── 前五名客户 ────────────────────────────────────────────────────────────────

/// 前五名客户合计信息，来自 qwm 接口。
///
/// API ColName: ["T003","T004","T005"]
/// T003=名称/合计, T004=销售金额, T005=占年度销售总额比例%
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JyfxTop5CustomerRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期（年报期）
    pub report_date: String,
    /// 名称（通常为"合计"）(T003)
    pub customer_name: String,
    /// 销售金额，元 (T004)
    pub amount: f64,
    /// 占年度销售总额比例 % (T005)
    pub revenue_pct: f64,
}

// ── 前五名供应商 ──────────────────────────────────────────────────────────────

/// 前五名供应商合计信息，来自 qwmgys 接口。
///
/// API ColName: ["T003","T004","T005"]
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JyfxTop5SupplierRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期（年报期）
    pub report_date: String,
    /// 名称（通常为"合计"）(T003)
    pub supplier_name: String,
    /// 采购金额，元 (T004)
    pub amount: f64,
    /// 占年度采购总额比例 % (T005)
    pub purchase_pct: f64,
}

// ── 经营数据 ──────────────────────────────────────────────────────────────────

/// 经营数据（多期 KV 格式），来自 jysj 接口。
///
/// 包含 茅台酒营业收入、系列酒收入、国内/国外收入、经销商数量等。
/// API ColName: ["N001","N002","N003","N004"]
/// N001=报告期, N002=指标名称, N003=指标值, N004=指标编码
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JyfxOperDataRow {
    /// 股票代码
    pub stock_code: String,
    /// 报告期，如 "2026-03-31"
    pub report_date: String,
    /// 指标名称 (N002)，如 "茅台酒营业收入(万元)"
    pub metric_name: String,
    /// 指标值 (N003)
    pub metric_value: f64,
    /// 指标编码 (N004)，如 "1007"
    pub metric_code: String,
}

// ── 数据抓取 ──────────────────────────────────────────────────────────────────

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<JyfxMainBizRow>,
        Vec<JyfxTop5CustomerRow>,
        Vec<JyfxTop5SupplierRow>,
        Vec<JyfxOperDataRow>,
    ),
    F10Error,
> {
    // 阶段1：两个 comreq 日期查询可并行（互不依赖）
    let (zygcfx_dates, qwm_dates) = tokio::join!(
        async { client.post("tdxf10_gg_comreq", &["zygcfx", code]).await },
        async { client.post("tdxf10_gg_comreq", &["qwm", code]).await },
    );
    let zygcfx_dates = zygcfx_dates.unwrap_or_default();
    let qwm_dates = qwm_dates.unwrap_or_default();

    let report_date = zygcfx_dates
        .result_sets
        .first()
        .and_then(|rs| rs.content.first())
        .and_then(|row| row.first())
        .and_then(serde_json::Value::as_i64)
        .map_or_else(|| "20260331".to_string(), |i| i.to_string());
    let annual_date = qwm_dates
        .result_sets
        .first()
        .and_then(|rs| rs.content.first())
        .and_then(|row| row.first())
        .and_then(serde_json::Value::as_i64)
        .map_or_else(|| "20251231".to_string(), |i| i.to_string());

    // 阶段2：4 个子请求全部并行发出
    let (zygc, qwm, qwmgys, jysj) = tokio::join!(
        async {
            let d = report_date.as_str();
            client.post("tdxf10_gg_jyfx", &[code, "zygc", d]).await
        },
        async {
            let d = annual_date.as_str();
            client.post("tdxf10_gg_jyfx", &[code, "qwm", d]).await
        },
        async {
            let d = annual_date.as_str();
            client.post("tdxf10_gg_jyfx", &[code, "qwmgys", d]).await
        },
        async { client.post("tdxf10_gg_jyfx_jysj", &[code]).await },
    );
    let zygc = zygc.unwrap_or_default();
    let qwm = qwm.unwrap_or_default();
    let qwmgys = qwmgys.unwrap_or_default();
    let jysj = jysj.unwrap_or_default();

    // 主营构成
    let mut main_bizs = vec![];
    if let Some(rs) = zygc.result_sets.first() {
        for i in 0..rs.content.len() {
            main_bizs.push(JyfxMainBizRow {
                stock_code: code.to_string(),
                report_date: report_date.clone(),
                category_type: get_str(rs, i, "N000").unwrap_or_default(),
                category_code: get_str(rs, i, "N001").unwrap_or_default(),
                category_name: get_str(rs, i, "N002").unwrap_or_default(),
                revenue: get_f64(rs, i, "N003").unwrap_or(0.0),
                revenue_pct: get_f64(rs, i, "N004").unwrap_or(0.0),
                profit: get_f64(rs, i, "N005").unwrap_or(0.0),
                gross_margin: get_f64(rs, i, "N007").unwrap_or(0.0),
            });
        }
    }

    // 前五名客户（T003=名称, T004=金额, T005=占比）
    let mut customers = vec![];
    if let Some(rs) = qwm.result_sets.first() {
        for i in 0..rs.content.len() {
            customers.push(JyfxTop5CustomerRow {
                stock_code: code.to_string(),
                report_date: annual_date.clone(),
                customer_name: get_str(rs, i, "T003").unwrap_or_default(),
                amount: get_f64(rs, i, "T004").unwrap_or(0.0),
                revenue_pct: get_f64(rs, i, "T005").unwrap_or(0.0),
            });
        }
    }

    // 前五名供应商（T003=名称, T004=金额, T005=占比）
    let mut suppliers = vec![];
    if let Some(rs) = qwmgys.result_sets.first() {
        for i in 0..rs.content.len() {
            suppliers.push(JyfxTop5SupplierRow {
                stock_code: code.to_string(),
                report_date: annual_date.clone(),
                supplier_name: get_str(rs, i, "T003").unwrap_or_default(),
                amount: get_f64(rs, i, "T004").unwrap_or(0.0),
                purchase_pct: get_f64(rs, i, "T005").unwrap_or(0.0),
            });
        }
    }

    // 经营数据（KV，多报告期）：RS2 和 RS3 都是 N001/N002/N003/N004 格式
    let mut oper_data = vec![];
    for rs_idx in [2usize, 3, 4] {
        if let Some(rs) = jysj.result_sets.get(rs_idx) {
            for i in 0..rs.content.len() {
                let rdate = get_str(rs, i, "N001").unwrap_or_default();
                let mname = get_str(rs, i, "N002").unwrap_or_default();
                if rdate.is_empty() || mname.is_empty() {
                    continue;
                }
                oper_data.push(JyfxOperDataRow {
                    stock_code: code.to_string(),
                    report_date: rdate,
                    metric_name: mname,
                    metric_value: get_f64(rs, i, "N003").unwrap_or(0.0),
                    metric_code: get_str(rs, i, "N004").unwrap_or_default(),
                });
            }
        }
    }
    // 去重（同 date+name 可能在 RS2/RS3 都出现）
    oper_data.dedup_by(|a, b| a.report_date == b.report_date && a.metric_name == b.metric_name);

    debug!(
        "jyfx {code}: main_biz={} customers={} suppliers={} oper_data={}",
        main_bizs.len(),
        customers.len(),
        suppliers.len(),
        oper_data.len()
    );
    Ok((main_bizs, customers, suppliers, oper_data))
}

// ── ClickHouse 写入 ───────────────────────────────────────────────────────────

/// 写入主营构成数据
pub async fn insert_main_bizs(ch: &Client, rows: &[JyfxMainBizRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<JyfxMainBizRow>("f10_jyfx_main_biz").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 写入前五名客户数据
pub async fn insert_top5_customers(
    ch: &Client,
    rows: &[JyfxTop5CustomerRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<JyfxTop5CustomerRow>("f10_jyfx_top5_customer")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 写入前五名供应商数据
pub async fn insert_top5_suppliers(
    ch: &Client,
    rows: &[JyfxTop5SupplierRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<JyfxTop5SupplierRow>("f10_jyfx_top5_supplier")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

/// 写入经营数据
pub async fn insert_oper_data(ch: &Client, rows: &[JyfxOperDataRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<JyfxOperDataRow>("f10_jyfx_oper_data").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

// ── 查询示例 ──────────────────────────────────────────────────────────────────

/// 查询主营构成（指定报告期）
pub async fn query_main_biz(
    ch: &Client,
    code: &str,
    report_date: &str,
) -> Result<Vec<JyfxMainBizRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyfx_main_biz FINAL WHERE stock_code = ? AND report_date = ? ORDER BY revenue DESC")
        .bind(code).bind(report_date).fetch_all().await?)
}

/// 查询前五名客户
pub async fn query_top5_customers(
    ch: &Client,
    code: &str,
) -> Result<Vec<JyfxTop5CustomerRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyfx_top5_customer FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

/// 查询前五名供应商
pub async fn query_top5_suppliers(
    ch: &Client,
    code: &str,
) -> Result<Vec<JyfxTop5SupplierRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyfx_top5_supplier FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

/// 查询经营数据（指定报告期，按指标名称过滤）
pub async fn query_oper_data(
    ch: &Client,
    code: &str,
    report_date: &str,
) -> Result<Vec<JyfxOperDataRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyfx_oper_data FINAL WHERE stock_code = ? AND report_date = ? ORDER BY metric_code")
        .bind(code).bind(report_date).fetch_all().await?)
}
