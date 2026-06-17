//! 步骤 10：研报评级 (gg_ybpj)
//!
//! 涵盖：投资评级统计、盈利预测明细、研报列表

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_i32, get_str, TqLexClient};
use crate::error::F10Error;

/// 投资评级统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct YbpjRatingStatRow {
    pub stock_code: String,
    pub stat_date: String,
    pub days_count: i32,
    pub total_inst: i32,
    pub buy_count: i32,
    pub overweight_count: i32,
    pub neutral_count: i32,
    pub underweight_count: i32,
    pub sell_count: i32,
    pub avg_score: f64,
    pub avg_target_price: f64,
}

/// 盈利预测明细（ylycmx 接口）
///
/// API RS1 ColName: ["T012","T005","flag","T004","T006","T014","T015","T016","T003"]
/// T012=日期, T005=变动类型(维持/上调), T004=评级, T006=目标价(String), T014-T016=EPS年1-3, T003=机构
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct YbpjForecastRow {
    /// 股票代码
    pub stock_code: String,
    /// 研报日期 (T012)，如 "20260516"
    pub report_date: String,
    /// 机构名 (T003)
    pub org_name: String,
    /// 评级 (T004)，如 "买入"
    pub rating: String,
    /// 变动类型 (T005)，如 "维持"/"上调"
    pub change_type: String,
    /// 目标价 (T006)，字符串
    pub target_price: String,
    /// 第一年 EPS 预测 (T014)
    pub eps_year1: f64,
    /// 第二年 EPS 预测 (T015)
    pub eps_year2: f64,
    /// 第三年 EPS 预测 (T016)
    pub eps_year3: f64,
}

/// 研报列表
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct YbpjReportRow {
    pub stock_code: String,
    pub report_date: String,
    pub title: String,
    pub rating: String,
    pub org_name: String,
    pub summary: String,
    pub report_id: String,
}

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<YbpjRatingStatRow>,
        Vec<YbpjForecastRow>,
        Vec<YbpjReportRow>,
    ),
    F10Error,
> {
    // 3 个子请求全部并行发出
    let (tzpjtj, ylycmx, ycpjyjbg) = tokio::join!(
        async { client.post("tdxf10_gg_ybpj", &[code, "tzpjtj"]).await },
        async { client.post("tdxf10_gg_ybpj", &[code, "ylycmx"]).await },
        async { client.post("tdxf10_gg_ybpj", &[code, "ycpjyjbg"]).await },
    );
    let tzpjtj = tzpjtj.unwrap_or_default();
    let ylycmx = ylycmx.unwrap_or_default();
    let ycpjyjbg = ycpjyjbg.unwrap_or_default();

    // 评级统计 T016=日期,sj=天数,zj=总机构,mr=买入,zc=增持,zx=中性,jc=减持,mc=卖出,pj=平均分,T006=平均目标价
    let mut stats = vec![];
    if let Some(rs) = tzpjtj.result_sets.first() {
        for i in 0..rs.content.len() {
            stats.push(YbpjRatingStatRow {
                stock_code: code.to_string(),
                stat_date: get_str(rs, i, "T016").unwrap_or_default(),
                days_count: get_i32(rs, i, "sj").unwrap_or(0),
                total_inst: get_i32(rs, i, "zj").unwrap_or(0),
                buy_count: get_i32(rs, i, "mr").unwrap_or(0),
                overweight_count: get_i32(rs, i, "zc").unwrap_or(0),
                neutral_count: get_i32(rs, i, "zx").unwrap_or(0),
                underweight_count: get_i32(rs, i, "jc").unwrap_or(0),
                sell_count: get_i32(rs, i, "mc").unwrap_or(0),
                avg_score: get_f64(rs, i, "pj").unwrap_or(0.0),
                avg_target_price: get_f64(rs, i, "T006").unwrap_or(0.0),
            });
        }
    }

    // 盈利预测明细 RS1=预测明细(T012=日期,T005=变动,T004=评级,T006=目标价,T014-T016=EPS年1-3,T003=机构)
    let mut forecasts = vec![];
    if let Some(rs) = ylycmx.result_sets.get(1) {
        for i in 0..rs.content.len() {
            forecasts.push(YbpjForecastRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T012").unwrap_or_default(),
                org_name: get_str(rs, i, "T003").unwrap_or_default(),
                rating: get_str(rs, i, "T004").unwrap_or_default(),
                change_type: get_str(rs, i, "T005").unwrap_or_default(),
                target_price: get_str(rs, i, "T006").unwrap_or_default(),
                eps_year1: get_f64(rs, i, "T014").unwrap_or(0.0),
                eps_year2: get_f64(rs, i, "T015").unwrap_or(0.0),
                eps_year3: get_f64(rs, i, "T016").unwrap_or(0.0),
            });
        }
    }

    // 研报列表 T011=报告id,sj=日期,pj=评级,jg=机构,ytxt=摘要,T039=标题
    let mut reports = vec![];
    if let Some(rs) = ycpjyjbg.result_sets.first() {
        for i in 0..rs.content.len() {
            reports.push(YbpjReportRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "sj").unwrap_or_default(),
                title: get_str(rs, i, "T039").unwrap_or_default(),
                rating: get_str(rs, i, "pj").unwrap_or_default(),
                org_name: get_str(rs, i, "jg").unwrap_or_default(),
                summary: get_str(rs, i, "ytxt").unwrap_or_default(),
                report_id: get_str(rs, i, "T011").unwrap_or_default(),
            });
        }
    }

    debug!(
        "ybpj {code}: stats={} forecasts={} reports={}",
        stats.len(),
        forecasts.len(),
        reports.len()
    );
    Ok((stats, forecasts, reports))
}

pub async fn insert_rating_stats(ch: &Client, rows: &[YbpjRatingStatRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<YbpjRatingStatRow>("f10_ybpj_rating_stat")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_forecasts(ch: &Client, rows: &[YbpjForecastRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<YbpjForecastRow>("f10_ybpj_forecast").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_reports(ch: &Client, rows: &[YbpjReportRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<YbpjReportRow>("f10_ybpj_report").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_rating_stats(
    ch: &Client,
    code: &str,
) -> Result<Vec<YbpjRatingStatRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_ybpj_rating_stat FINAL WHERE stock_code = ? ORDER BY stat_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_forecasts(ch: &Client, code: &str) -> Result<Vec<YbpjForecastRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_ybpj_forecast FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_reports(ch: &Client, code: &str) -> Result<Vec<YbpjReportRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_ybpj_report FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all().await?)
}
