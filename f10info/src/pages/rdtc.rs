//! 步骤 11：热点题材 (gg_rdtc)
//!
//! 涵盖：板块族、主题库、事件驱动、投资逻辑、热点概念

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_str, TqLexClient};
use crate::error::F10Error;

/// 题材（板块族/主题库合并）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct RdtcThemeRow {
    pub stock_code: String,
    pub theme_type: String,
    pub theme_date: String,
    pub theme_name: String,
    pub theme_content: String,
    pub heat: f64,
}

/// 事件驱动
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct RdtcEventRow {
    pub stock_code: String,
    pub event_date: String,
    pub event_name: String,
    pub event_type: String,
}

/// 投资逻辑
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct RdtcLogicRow {
    pub stock_code: String,
    pub category: String,
    pub content: String,
}

/// 热点概念
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct RdtcConceptRow {
    pub stock_code: String,
    pub concept_name: String,
    pub concept_code: String,
    pub heat_score: f64,
}

pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<RdtcThemeRow>,
        Vec<RdtcEventRow>,
        Vec<RdtcLogicRow>,
        Vec<RdtcConceptRow>,
    ),
    F10Error,
> {
    // 5 个子请求全部并行发出
    let (bkz, ztk, sjcd, xxmmg, concept) = tokio::join!(
        async { client.post("tdxf10_gg_rdtc", &[code, "zttzbkz"]).await },
        async { client.post("tdxf10_gg_rdtc", &[code, "zttzztk"]).await },
        async { client.post("tdxf10_gg_rdtc", &[code, "sjcd"]).await },
        async { client.post("tdxf10_gg_rdtc", &[code, "xxmmg"]).await },
        async { client.post("tdxf10_gg_comreq", &["rdtcgn", code]).await },
    );
    let bkz = bkz.unwrap_or_default();
    let ztk = ztk.unwrap_or_default();
    let sjcd = sjcd.unwrap_or_default();
    let xxmmg = xxmmg.unwrap_or_default();
    let concept = concept.unwrap_or_default();

    // 板块族 bflag,ztrq,ztmc,gld,rxsj,ztnr
    let mut themes = vec![];
    let parse_themes = |resp: &crate::client::TqLexResponse, ttype: &str| -> Vec<RdtcThemeRow> {
        let mut out = vec![];
        if let Some(rs) = resp.result_sets.first() {
            for i in 0..rs.content.len() {
                out.push(RdtcThemeRow {
                    stock_code: code.to_string(),
                    theme_type: ttype.to_string(),
                    theme_date: get_str(rs, i, "ztrq").unwrap_or_default(),
                    theme_name: get_str(rs, i, "ztmc").unwrap_or_default(),
                    theme_content: get_str(rs, i, "ztnr").unwrap_or_default(),
                    heat: get_f64(rs, i, "gld").unwrap_or(0.0),
                });
            }
        }
        out
    };
    themes.extend(parse_themes(&bkz, "板块族"));
    themes.extend(parse_themes(&ztk, "主题库"));

    // 事件驱动 cjrq=日期,sjmc=事件名,sjxz=事件性质
    let mut events = vec![];
    if let Some(rs) = sjcd.result_sets.first() {
        for i in 0..rs.content.len() {
            events.push(RdtcEventRow {
                stock_code: code.to_string(),
                event_date: get_str(rs, i, "cjrq").unwrap_or_default(),
                event_name: get_str(rs, i, "sjmc").unwrap_or_default(),
                event_type: get_str(rs, i, "sjxz").unwrap_or_default(),
            });
        }
    }

    // 投资逻辑 lmmc=类目名,zynr=内容
    let mut logics = vec![];
    if let Some(rs) = xxmmg.result_sets.first() {
        for i in 0..rs.content.len() {
            logics.push(RdtcLogicRow {
                stock_code: code.to_string(),
                category: get_str(rs, i, "lmmc").unwrap_or_default(),
                content: get_str(rs, i, "zynr").unwrap_or_default(),
            });
        }
    }

    // 热点概念 t001=概念ID, t002=概念名称 (注意是小写)
    let mut concepts = vec![];
    if let Some(rs) = concept.result_sets.first() {
        for i in 0..rs.content.len() {
            concepts.push(RdtcConceptRow {
                stock_code: code.to_string(),
                concept_code: get_str(rs, i, "t001").unwrap_or_default(),
                concept_name: get_str(rs, i, "t002").unwrap_or_default(),
                heat_score: 0.0, // rdtcgn 不返回热度
            });
        }
    }

    debug!(
        "rdtc {code}: themes={} events={} logics={} concepts={}",
        themes.len(),
        events.len(),
        logics.len(),
        concepts.len()
    );
    Ok((themes, events, logics, concepts))
}

pub async fn insert_themes(ch: &Client, rows: &[RdtcThemeRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<RdtcThemeRow>("f10_rdtc_theme").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_events(ch: &Client, rows: &[RdtcEventRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<RdtcEventRow>("f10_rdtc_event").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_logics(ch: &Client, rows: &[RdtcLogicRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<RdtcLogicRow>("f10_rdtc_logic").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_concepts(ch: &Client, rows: &[RdtcConceptRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<RdtcConceptRow>("f10_rdtc_concept").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_themes(ch: &Client, code: &str) -> Result<Vec<RdtcThemeRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_rdtc_theme FINAL WHERE stock_code = ? ORDER BY theme_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_events(ch: &Client, code: &str) -> Result<Vec<RdtcEventRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_rdtc_event FINAL WHERE stock_code = ? ORDER BY event_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_logics(ch: &Client, code: &str) -> Result<Vec<RdtcLogicRow>, F10Error> {
    Ok(ch
        .query("SELECT * EXCEPT(fetched_at) FROM f10_rdtc_logic FINAL WHERE stock_code = ?")
        .bind(code)
        .fetch_all()
        .await?)
}

pub async fn query_concepts(ch: &Client, code: &str) -> Result<Vec<RdtcConceptRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_rdtc_concept FINAL WHERE stock_code = ? ORDER BY heat_score DESC")
        .bind(code).fetch_all().await?)
}
