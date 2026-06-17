//! 步骤 12：公司资讯 (gg_gszx)
//!
//! 涵盖：公司新闻、公司公告、公司研报

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_str, TqLexClient};
use crate::error::F10Error;

/// 新闻/公告/研报统一结构
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GszxNewsRow {
    pub stock_code: String,
    pub news_type: String,
    pub pub_date: String,
    pub title: String,
    pub rec_id: String,
    pub is_important: i8,
}

/// 研报（含评级信息）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GszxReportRow {
    pub stock_code: String,
    pub pub_date: String,
    pub title: String,
    pub rating: String,
    pub analyst: String,
    pub org_name: String,
    pub report_id: String,
}

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<(Vec<GszxNewsRow>, Vec<GszxReportRow>), F10Error> {
    // 3 个子请求全部并行发出
    let (gsxw, gsgg, gsyj) = tokio::join!(
        async {
            client
                .post("tdxf10_gg_gszx", &[code, "gsxw", "", "0", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gszx", &[code, "gsgg", "", "0", "1", "20"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_gszx", &[code, "gsyj", "", "0", "1", "10"])
                .await
        },
    );
    let gsxw = gsxw.unwrap_or_default();
    let gsgg = gsgg.unwrap_or_default();
    let gsyj = gsyj.unwrap_or_default();

    // 新闻/公告 rec_id,issue=日期,Title,nflag=重要标志
    let mut news_rows = vec![];
    let parse_news = |resp: &crate::client::TqLexResponse, ntype: &str| -> Vec<GszxNewsRow> {
        let mut out = vec![];
        if let Some(rs) = resp.result_sets.first() {
            for i in 0..rs.content.len() {
                let title = get_str(rs, i, "Title").unwrap_or_default();
                if title.is_empty() {
                    continue;
                }
                out.push(GszxNewsRow {
                    stock_code: code.to_string(),
                    news_type: ntype.to_string(),
                    pub_date: get_str(rs, i, "issue").unwrap_or_default(),
                    title,
                    rec_id: get_str(rs, i, "rec_id").unwrap_or_default(),
                    is_important: get_str(rs, i, "nflag")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                });
            }
        }
        out
    };
    news_rows.extend(parse_news(&gsxw, "新闻"));
    news_rows.extend(parse_news(&gsgg, "公告"));

    // 研报 T004=评级,T009=研究员,T012=日期,T011=报告id,T039=标题 (无机构名字段)
    let mut reports = vec![];
    if let Some(rs) = gsyj.result_sets.first() {
        for i in 0..rs.content.len() {
            reports.push(GszxReportRow {
                stock_code: code.to_string(),
                pub_date: get_str(rs, i, "T012").unwrap_or_default(),
                title: get_str(rs, i, "T039").unwrap_or_default(),
                rating: get_str(rs, i, "T004").unwrap_or_default(),
                analyst: get_str(rs, i, "T009").unwrap_or_default(),
                org_name: String::new(), // gsyj 接口不返回机构名
                report_id: get_str(rs, i, "T011").unwrap_or_default(),
            });
        }
    }

    debug!(
        "gszx {code}: news={} reports={}",
        news_rows.len(),
        reports.len()
    );
    Ok((news_rows, reports))
}

pub async fn insert_news(ch: &Client, rows: &[GszxNewsRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GszxNewsRow>("f10_gszx_news").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_reports(ch: &Client, rows: &[GszxReportRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GszxReportRow>("f10_gszx_report").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_news(
    ch: &Client,
    code: &str,
    news_type: &str,
) -> Result<Vec<GszxNewsRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gszx_news FINAL WHERE stock_code = ? AND news_type = ? ORDER BY pub_date DESC")
        .bind(code).bind(news_type).fetch_all().await?)
}

pub async fn query_reports(ch: &Client, code: &str) -> Result<Vec<GszxReportRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gszx_report FINAL WHERE stock_code = ? ORDER BY pub_date DESC")
        .bind(code).fetch_all().await?)
}
