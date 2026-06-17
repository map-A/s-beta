//! 步骤 2：资金动向 (gg_jyds)
//!
//! 涵盖：大宗交易、融资融券、资金流向、龙虎榜、北上资金

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_str, TqLexClient};
use crate::error::F10Error;

/// 大宗交易记录
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JydsBlockTradeRow {
    pub stock_code: String,
    pub trade_date: String,
    pub price: f64,
    pub amount: f64, // 万元
    pub volume: f64, // 万股
    pub premium_pct: f64,
    pub buyer: String,
    pub seller: String,
}

/// 融资融券记录
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JydsMarginRow {
    pub stock_code: String,
    pub trade_date: String,
    /// 融资余额
    pub rzye: f64,
    /// 融资买入额
    pub rzmre: f64,
    /// 融资偿还额
    pub rzch: f64,
    /// 融券余量
    pub rqyl: f64,
    /// 融券卖出量
    pub rqmcl: f64,
    /// 融券偿还量
    pub rqch: f64,
    /// 融资融券余额
    pub rzrqye: f64,
    pub close_price: f64,
}

/// 资金流向记录（主力=超大单+大单）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JydsMoneyflowRow {
    pub stock_code: String,
    pub trade_date: String,
    /// 主力净额（超大+大单）
    pub zl_net: f64,
    pub zl_pct: f64,
    /// 超大单净额
    pub super_net: f64,
    pub super_pct: f64,
    /// 大单净额
    pub big_net: f64,
    pub big_pct: f64,
    /// 散户净额
    pub retail_net: f64,
    pub retail_pct: f64,
}

/// 龙虎榜触发记录
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JydsDragonTigerRow {
    pub stock_code: String,
    pub trade_date: String,
    pub reason: String,
    pub event_type: String,
}

/// 北上资金成交明细
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct JydsNorthboundRow {
    pub stock_code: String,
    pub trade_date: String,
    pub direction: String,
    pub price: f64,
    pub volume: f64, // 万股
    pub amount: f64, // 万元
    pub hold_pct: f64,
}

/// 抓取资金动向全部数据
pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<JydsBlockTradeRow>,
        Vec<JydsMarginRow>,
        Vec<JydsMoneyflowRow>,
        Vec<JydsDragonTigerRow>,
        Vec<JydsNorthboundRow>,
    ),
    F10Error,
> {
    // 5 个子请求全部并行发出
    let (dzjy, rzrq, zjlx, lgt, yxsb) = tokio::join!(
        async { client.post("tdxf10_gg_jyds", &[code, "dzjy", ""]).await },
        async { client.post("tdxf10_gg_jyds", &[code, "rzrq", ""]).await },
        async { client.post("tdxf10_gg_jyds", &[code, "zjlx", ""]).await },
        async { client.post("tdxf10_gg_jyds", &[code, "lgt", "2"]).await },
        async { client.post("tdxf10_gg_jyds", &[code, "yxsbxx", ""]).await },
    );
    let dzjy = dzjy.unwrap_or_default();
    let rzrq = rzrq.unwrap_or_default();
    let zjlx = zjlx.unwrap_or_default();
    let lgt = lgt.unwrap_or_default();
    let yxsb = yxsb.unwrap_or_default();

    // 大宗交易 RS0: T003=日期(int), T004=价格, T005=金额(万), T006=数量(万股), T007=买方, T008=卖方, yjl=折价率
    let mut blocks = vec![];
    if let Some(rs) = dzjy.result_sets.first() {
        for i in 0..rs.content.len() {
            blocks.push(JydsBlockTradeRow {
                stock_code: code.to_string(),
                trade_date: get_str(rs, i, "T003").unwrap_or_default(),
                price: get_f64(rs, i, "T004").unwrap_or(0.0),
                amount: get_f64(rs, i, "T005").unwrap_or(0.0),
                volume: get_f64(rs, i, "T006").unwrap_or(0.0),
                premium_pct: get_f64(rs, i, "yjl").unwrap_or(0.0),
                buyer: get_str(rs, i, "T007").unwrap_or_default(),
                seller: get_str(rs, i, "T008").unwrap_or_default(),
            });
        }
    }

    // 融资融券 RS0: T001=日期, T003=融资余额, T004=融资买入, T005=融资偿还,
    //              T006=融券余量, T007=融券卖出量, T008=融券偿还量, bt010=融资融券余额, ClosePrice
    let mut margins = vec![];
    if let Some(rs) = rzrq.result_sets.first() {
        for i in 0..rs.content.len() {
            margins.push(JydsMarginRow {
                stock_code: code.to_string(),
                trade_date: get_str(rs, i, "T001").unwrap_or_default(),
                rzye: get_f64(rs, i, "T003").unwrap_or(0.0),
                rzmre: get_f64(rs, i, "T004").unwrap_or(0.0),
                rzch: get_f64(rs, i, "T005").unwrap_or(0.0),
                rqyl: get_f64(rs, i, "T006").unwrap_or(0.0),
                rqmcl: get_f64(rs, i, "T007").unwrap_or(0.0),
                rqch: get_f64(rs, i, "T008").unwrap_or(0.0),
                rzrqye: get_f64(rs, i, "bt010").unwrap_or(0.0),
                close_price: get_f64(rs, i, "ClosePrice").unwrap_or(0.0),
            });
        }
    }

    // 资金流向: rq=日期, N001/N002=主力净/率, N003/N004=超大单净/率, N005/N006=大单净/率, N007/N008=散户净/率
    let mut flows = vec![];
    if let Some(rs) = zjlx.result_sets.first() {
        for i in 0..rs.content.len() {
            flows.push(JydsMoneyflowRow {
                stock_code: code.to_string(),
                trade_date: get_str(rs, i, "rq").unwrap_or_default(),
                zl_net: get_f64(rs, i, "N001").unwrap_or(0.0),
                zl_pct: get_f64(rs, i, "N002").unwrap_or(0.0),
                super_net: get_f64(rs, i, "N003").unwrap_or(0.0),
                super_pct: get_f64(rs, i, "N004").unwrap_or(0.0),
                big_net: get_f64(rs, i, "N005").unwrap_or(0.0),
                big_pct: get_f64(rs, i, "N006").unwrap_or(0.0),
                retail_net: get_f64(rs, i, "N007").unwrap_or(0.0),
                retail_pct: get_f64(rs, i, "N008").unwrap_or(0.0),
            });
        }
    }

    // 龙虎榜 RS0: rq=日期, T006=原因, T007=类型（空时无龙虎榜）
    let mut dragons = vec![];
    if let Some(rs) = lgt.result_sets.first() {
        for i in 0..rs.content.len() {
            dragons.push(JydsDragonTigerRow {
                stock_code: code.to_string(),
                trade_date: get_str(rs, i, "rq").unwrap_or_default(),
                reason: get_str(rs, i, "T006").unwrap_or_default(),
                event_type: get_str(rs, i, "T007").unwrap_or_default(),
            });
        }
    }

    // 北上资金成交明细: N001=日期, N002=方向, N003=价格, N004=数量(万股), N005=金额(万元), N006=持股比例
    let mut northbound = vec![];
    if let Some(rs) = yxsb.result_sets.first() {
        for i in 0..rs.content.len() {
            northbound.push(JydsNorthboundRow {
                stock_code: code.to_string(),
                trade_date: get_str(rs, i, "N001").unwrap_or_default(),
                direction: get_str(rs, i, "N002").unwrap_or_default(),
                price: get_f64(rs, i, "N003").unwrap_or(0.0),
                volume: get_f64(rs, i, "N004").unwrap_or(0.0),
                amount: get_f64(rs, i, "N005").unwrap_or(0.0),
                hold_pct: get_f64(rs, i, "N006").unwrap_or(0.0),
            });
        }
    }

    debug!(
        "jyds {code}: blocks={} margins={} flows={} dragons={} northbound={}",
        blocks.len(),
        margins.len(),
        flows.len(),
        dragons.len(),
        northbound.len()
    );
    Ok((blocks, margins, flows, dragons, northbound))
}

pub async fn insert_block_trades(ch: &Client, rows: &[JydsBlockTradeRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<JydsBlockTradeRow>("f10_jyds_block_trade")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_margins(ch: &Client, rows: &[JydsMarginRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<JydsMarginRow>("f10_jyds_margin").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_moneyflows(ch: &Client, rows: &[JydsMoneyflowRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<JydsMoneyflowRow>("f10_jyds_moneyflow").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_dragon_tigers(
    ch: &Client,
    rows: &[JydsDragonTigerRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<JydsDragonTigerRow>("f10_jyds_dragon_tiger")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_northbound(ch: &Client, rows: &[JydsNorthboundRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<JydsNorthboundRow>("f10_jyds_northbound")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_block_trades(
    ch: &Client,
    code: &str,
) -> Result<Vec<JydsBlockTradeRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyds_block_trade FINAL WHERE stock_code = ? ORDER BY trade_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_margins(ch: &Client, code: &str) -> Result<Vec<JydsMarginRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyds_margin FINAL WHERE stock_code = ? ORDER BY trade_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_moneyflows(ch: &Client, code: &str) -> Result<Vec<JydsMoneyflowRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyds_moneyflow FINAL WHERE stock_code = ? ORDER BY trade_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_northbound(ch: &Client, code: &str) -> Result<Vec<JydsNorthboundRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_jyds_northbound FINAL WHERE stock_code = ? ORDER BY trade_date DESC")
        .bind(code).fetch_all().await?)
}
