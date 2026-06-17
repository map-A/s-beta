//! 步骤 14：行业分析 (gg_hyfx)
//!
//! 涵盖：行业新闻、行业研报、市场表现排名、公司规模排名、估值水平排名、财务状况排名、分红融资比排名

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_i32, get_str, TqLexClient, TqLexResponse};
use crate::error::F10Error;

/// 行业新闻（hyxw）— 表 f10_hyfx_industry_news
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HyfxIndustryNewsRow {
    pub stock_code: String,
    pub pub_date: String,
    pub title: String,
    pub rec_id: String,
}

/// 分红融资比排名（fhrzb）— 表 f10_hyfx_dividend_rank
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HyfxDividendRankRow {
    pub stock_code: String,
    pub peer_code: String,
    pub peer_name: String,
    pub rank: i32,
    /// 上市以来分红(元/股) ssfh
    pub dividend_per_share: f64,
    /// 上市以来融资 sssz
    pub ipo_funding: f64,
    /// 总分红金额 fhje
    pub total_dividend: f64,
    /// 首发/送配股融资额 sfmzje
    pub ipo_amount: f64,
    /// 增发次数 zfcs
    pub addissue_count: i32,
    /// 增发融资额 zfmzje
    pub addissue_amount: f64,
    /// 配股次数 pgcs
    pub rights_count: i32,
    /// 配股融资额 pgmzje
    pub rights_amount: f64,
    /// 可转债融资额 kzzmzje
    pub cb_amount: f64,
    /// 分红融资比 fhrzb
    pub dividend_funding_ratio: f64,
    /// 目标股票在同行中的排名（自身行号）
    pub self_rank: i32,
}

/// 市场表现排名（行业内同行）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HyfxMarketRankRow {
    pub stock_code: String,
    pub peer_code: String,
    pub peer_name: String,
    pub exchange: String,
    pub chg_day: f64,
    pub chg_week: f64,
    pub chg_month: f64,
    pub chg_quarter: f64,
    pub chg_half_year: f64,
    pub chg_year: f64,
    pub self_rank: i32,
}

/// 公司规模排名
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HyfxSizeRankRow {
    pub stock_code: String,
    pub peer_code: String,
    pub peer_name: String,
    pub total_market_cap: f64,
    pub float_market_cap: f64,
    pub total_shares: f64,
    pub revenue: f64,
    pub price: f64,
    pub self_rank: i32,
}

/// 估值水平排名
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HyfxValuationRankRow {
    pub stock_code: String,
    pub peer_code: String,
    pub peer_name: String,
    pub price: f64,
    pub pe_ttm: f64,
    pub pe_lyr: f64,
    pub pb: f64,
    pub ps: f64,
    pub pcf: f64,
    pub self_rank: i32,
}

/// 财务状况排名
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HyfxFinancialRankRow {
    pub stock_code: String,
    pub peer_code: String,
    pub peer_name: String,
    pub eps: f64,
    pub bvps: f64,
    pub revenue_growth: f64,
    pub roe: f64,
    pub net_profit_growth: f64,
    pub self_rank: i32,
}

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<HyfxIndustryNewsRow>, // hyxw 行业新闻
        Vec<HyfxIndustryNewsRow>, // hyyb 行业研报
        Vec<HyfxMarketRankRow>,
        Vec<HyfxSizeRankRow>,
        Vec<HyfxValuationRankRow>,
        Vec<HyfxFinancialRankRow>,
        Vec<HyfxDividendRankRow>, // fhrzb 分红融资比排名
    ),
    F10Error,
> {
    // 7 个子请求全部并行发出
    let (hyxw, hyyb, scbx, gsgm, gzsp, cwgz, fhrzb) = tokio::join!(
        async { client.post("tdxf10_gg_hyfx", &["hyxw", code, ""]).await },
        async { client.post("tdxf10_gg_hyfx", &["hyyb", code, ""]).await },
        async { client.post("tdxf10_gg_hyfx", &["scbx", code, "000"]).await },
        async { client.post("tdxf10_gg_hyfx", &["gsgm", code, "000"]).await },
        async { client.post("tdxf10_gg_hyfx", &["gzsp", code, "011"]).await },
        async { client.post("tdxf10_gg_hyfx", &["cwgz", code, "000"]).await },
        async { client.post("tdxf10_gg_hyfx", &["fhrzb", code, "100"]).await },
    );
    let hyxw = hyxw.unwrap_or_default();
    let hyyb = hyyb.unwrap_or_default();
    let scbx = scbx.unwrap_or_default();
    let gsgm = gsgm.unwrap_or_default();
    let gzsp = gzsp.unwrap_or_default();
    let cwgz = cwgz.unwrap_or_default();
    let fhrzb = fhrzb.unwrap_or_default();

    // 行业新闻/研报 — 同结构不同表
    let parse_news = |resp: &TqLexResponse| -> Vec<HyfxIndustryNewsRow> {
        let mut out = vec![];
        if let Some(rs) = resp.result_sets.first() {
            for i in 0..rs.content.len() {
                let title = get_str(rs, i, "Title").unwrap_or_default();
                if title.is_empty() {
                    continue;
                }
                out.push(HyfxIndustryNewsRow {
                    stock_code: code.to_string(),
                    pub_date: get_str(rs, i, "issue").unwrap_or_default(),
                    title,
                    rec_id: get_str(rs, i, "rec_id").unwrap_or_default(),
                });
            }
        }
        out
    };
    let industry_news = parse_news(&hyxw);
    let industry_reports = parse_news(&hyyb);

    // 辅助：从 RS1 获取本股票排名
    let get_self_rank = |resp: &TqLexResponse| -> i32 {
        resp.result_sets
            .get(1)
            .and_then(|rs| rs.content.first())
            .and_then(|row| row.first())
            .and_then(serde_json::Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(0)
    };

    // 市场表现排名 zqdm,zqjc,sc,T022=日,T023=周,T024=月,T025=季,T026=半年,T027=年
    let mut market_ranks = vec![];
    let self_rank_scbx = get_self_rank(&scbx);
    if let Some(rs) = scbx.result_sets.first() {
        for i in 0..rs.content.len() {
            market_ranks.push(HyfxMarketRankRow {
                stock_code: code.to_string(),
                peer_code: get_str(rs, i, "zqdm").unwrap_or_default(),
                peer_name: get_str(rs, i, "zqjc").unwrap_or_default(),
                exchange: get_str(rs, i, "sc").unwrap_or_default(),
                chg_day: get_f64(rs, i, "T022").unwrap_or(0.0),
                chg_week: get_f64(rs, i, "T023").unwrap_or(0.0),
                chg_month: get_f64(rs, i, "T024").unwrap_or(0.0),
                chg_quarter: get_f64(rs, i, "T025").unwrap_or(0.0),
                chg_half_year: get_f64(rs, i, "T026").unwrap_or(0.0),
                chg_year: get_f64(rs, i, "T027").unwrap_or(0.0),
                self_rank: self_rank_scbx,
            });
        }
    }

    // 公司规模排名 zqdm,zqjc,T013=总市值,T012=流通市值,T010=总股本,T009=营收,T006=股价
    let mut size_ranks = vec![];
    let self_rank_gsgm = get_self_rank(&gsgm);
    if let Some(rs) = gsgm.result_sets.first() {
        for i in 0..rs.content.len() {
            size_ranks.push(HyfxSizeRankRow {
                stock_code: code.to_string(),
                peer_code: get_str(rs, i, "zqdm").unwrap_or_default(),
                peer_name: get_str(rs, i, "zqjc").unwrap_or_default(),
                total_market_cap: get_f64(rs, i, "T013").unwrap_or(0.0),
                float_market_cap: get_f64(rs, i, "T012").unwrap_or(0.0),
                total_shares: get_f64(rs, i, "T010").unwrap_or(0.0),
                revenue: get_f64(rs, i, "T009").unwrap_or(0.0),
                price: get_f64(rs, i, "T006").unwrap_or(0.0),
                self_rank: self_rank_gsgm,
            });
        }
    }

    // 估值水平排名 zqdm,zqjc,T006=股价,T019=PE_TTM,T018=PE_LYR,T017=PB,T020=PS,T021=PCF
    let mut val_ranks = vec![];
    let self_rank_gzsp = get_self_rank(&gzsp);
    if let Some(rs) = gzsp.result_sets.first() {
        for i in 0..rs.content.len() {
            val_ranks.push(HyfxValuationRankRow {
                stock_code: code.to_string(),
                peer_code: get_str(rs, i, "zqdm").unwrap_or_default(),
                peer_name: get_str(rs, i, "zqjc").unwrap_or_default(),
                price: get_f64(rs, i, "T006").unwrap_or(0.0),
                pe_ttm: get_f64(rs, i, "T019").unwrap_or(0.0),
                pe_lyr: get_f64(rs, i, "T018").unwrap_or(0.0),
                pb: get_f64(rs, i, "T017").unwrap_or(0.0),
                ps: get_f64(rs, i, "T020").unwrap_or(0.0),
                pcf: get_f64(rs, i, "T021").unwrap_or(0.0),
                self_rank: self_rank_gzsp,
            });
        }
    }

    // 财务状况排名 pm=排名,T056=EPS,T059=净资产/股,T062=营收增长率,T067=ROE,T118=净利润增长率
    let mut fin_ranks = vec![];
    let self_rank_cwgz = get_self_rank(&cwgz);
    if let Some(rs) = cwgz.result_sets.first() {
        for i in 0..rs.content.len() {
            fin_ranks.push(HyfxFinancialRankRow {
                stock_code: code.to_string(),
                peer_code: get_str(rs, i, "zqdm").unwrap_or_default(),
                peer_name: get_str(rs, i, "zqjc").unwrap_or_default(),
                eps: get_f64(rs, i, "T056").unwrap_or(0.0),
                bvps: get_f64(rs, i, "T059").unwrap_or(0.0),
                revenue_growth: get_f64(rs, i, "T062").unwrap_or(0.0),
                roe: get_f64(rs, i, "T067").unwrap_or(0.0),
                net_profit_growth: get_f64(rs, i, "T118").unwrap_or(0.0),
                self_rank: self_rank_cwgz,
            });
        }
    }

    // 分红融资比排名 fhrzb — RS0 = 同行列表, RS1 = 本股自身行
    let mut div_ranks = vec![];
    let self_rank_fhrzb = fhrzb
        .result_sets
        .get(1)
        .and_then(|rs| rs.content.first())
        .and_then(|row| row.first())
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(0);
    if let Some(rs) = fhrzb.result_sets.first() {
        for i in 0..rs.content.len() {
            div_ranks.push(HyfxDividendRankRow {
                stock_code: code.to_string(),
                peer_code: get_str(rs, i, "zqdm").unwrap_or_default(),
                peer_name: get_str(rs, i, "zqmc").unwrap_or_default(),
                rank: get_i32(rs, i, "pm").unwrap_or(0),
                dividend_per_share: get_f64(rs, i, "ssfh").unwrap_or(0.0),
                ipo_funding: get_f64(rs, i, "sssz").unwrap_or(0.0),
                total_dividend: get_f64(rs, i, "fhje").unwrap_or(0.0),
                ipo_amount: get_f64(rs, i, "sfmzje").unwrap_or(0.0),
                addissue_count: get_i32(rs, i, "zfcs").unwrap_or(0),
                addissue_amount: get_f64(rs, i, "zfmzje").unwrap_or(0.0),
                rights_count: get_i32(rs, i, "pgcs").unwrap_or(0),
                rights_amount: get_f64(rs, i, "pgmzje").unwrap_or(0.0),
                cb_amount: get_f64(rs, i, "kzzmzje").unwrap_or(0.0),
                dividend_funding_ratio: get_f64(rs, i, "fhrzb").unwrap_or(0.0),
                self_rank: self_rank_fhrzb,
            });
        }
    }

    debug!(
        "hyfx {code}: news={} reports={} market={} size={} val={} fin={} div={}",
        industry_news.len(),
        industry_reports.len(),
        market_ranks.len(),
        size_ranks.len(),
        val_ranks.len(),
        fin_ranks.len(),
        div_ranks.len()
    );
    Ok((
        industry_news,
        industry_reports,
        market_ranks,
        size_ranks,
        val_ranks,
        fin_ranks,
        div_ranks,
    ))
}

pub async fn insert_industry_news(
    ch: &Client,
    rows: &[HyfxIndustryNewsRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<HyfxIndustryNewsRow>("f10_hyfx_industry_news")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_industry_reports(
    ch: &Client,
    rows: &[HyfxIndustryNewsRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<HyfxIndustryNewsRow>("f10_hyfx_industry_report")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_market_ranks(ch: &Client, rows: &[HyfxMarketRankRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<HyfxMarketRankRow>("f10_hyfx_market_rank")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_size_ranks(ch: &Client, rows: &[HyfxSizeRankRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<HyfxSizeRankRow>("f10_hyfx_size_rank").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_valuation_ranks(
    ch: &Client,
    rows: &[HyfxValuationRankRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<HyfxValuationRankRow>("f10_hyfx_valuation_rank")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_financial_ranks(
    ch: &Client,
    rows: &[HyfxFinancialRankRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<HyfxFinancialRankRow>("f10_hyfx_financial_rank")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_dividend_ranks(
    ch: &Client,
    rows: &[HyfxDividendRankRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<HyfxDividendRankRow>("f10_hyfx_dividend_rank")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_industry_news(
    ch: &Client,
    code: &str,
) -> Result<Vec<HyfxIndustryNewsRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_hyfx_industry_news FINAL WHERE stock_code = ? ORDER BY pub_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_industry_reports(
    ch: &Client,
    code: &str,
) -> Result<Vec<HyfxIndustryNewsRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_hyfx_industry_report FINAL WHERE stock_code = ? ORDER BY pub_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_dividend_ranks(
    ch: &Client,
    code: &str,
) -> Result<Vec<HyfxDividendRankRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_hyfx_dividend_rank FINAL WHERE stock_code = ? ORDER BY rank")
        .bind(code).fetch_all().await?)
}

pub async fn query_market_ranks(
    ch: &Client,
    code: &str,
) -> Result<Vec<HyfxMarketRankRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_hyfx_market_rank FINAL WHERE stock_code = ? ORDER BY chg_day DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_valuation_ranks(
    ch: &Client,
    code: &str,
) -> Result<Vec<HyfxValuationRankRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_hyfx_valuation_rank FINAL WHERE stock_code = ? ORDER BY pe_ttm")
        .bind(code).fetch_all().await?)
}

pub async fn query_financial_ranks(
    ch: &Client,
    code: &str,
) -> Result<Vec<HyfxFinancialRankRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_hyfx_financial_rank FINAL WHERE stock_code = ? ORDER BY roe DESC")
        .bind(code).fetch_all().await?)
}
