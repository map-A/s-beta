//! 步骤 1：最新提示 (gg_zxts)
//!
//! 涵盖：公司概要（含公司地位/市场人气/行业人气/概念题材/可比公司/质押信息）、
//! 主要指标（多期历史序列）、公司大事、资讯（新闻/公告/研报/路演）、机构调研、互动问答

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::client::{get_f64, get_i64, get_str, TqLexClient};
use crate::error::F10Error;

/// 公司概要快照（含公司地位、市场/行业人气排名、质押等完整字段）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsOverviewRow {
    pub stock_code: String,
    /// 报告期
    pub report_date: String,
    /// 每股收益 (mgsy)
    pub eps: f64,
    /// 每股净资产 (mgjzc)
    pub bvps: f64,
    /// 营业收入 (yysr)
    pub revenue: f64,
    /// 归母净利润 (gmjlr)
    pub net_profit: f64,
    /// 毛利率 % (mll)
    pub gross_margin: f64,
    /// 资产负债率 % (zcfzl)
    pub debt_ratio: f64,
    /// 每股未分配利润 (mgwfplr)
    pub undist_profit_ps: f64,
    /// 每股资本公积 (mgzbgj)
    pub capres_ps: f64,
    /// 每股经营现金流 (mgjjxjl)
    pub op_cf_ps: f64,
    /// 营收同比 % (yysrtbzzl)
    pub revenue_yoy: f64,
    /// 归母净利润同比 % (gmjlrtbzzl)
    pub profit_yoy: f64,
    /// 扣非净利润同比 % (kfjlrtbzzl)
    pub deducted_profit_yoy: f64,
    /// 市盈率 TTM (sylttm)
    pub pe_ttm: f64,
    /// 市盈率 LYR (syllyr)
    pub pe_lyr: f64,
    /// 市净率 (sjl)
    pub pb: f64,
    /// 总市值 (zsz)
    pub total_market_cap: f64,
    /// 总股本 (zgb)
    pub total_shares: f64,
    /// 流通 A 股 (ltag)
    pub float_a_shares: f64,
    /// 公司地位 (gsdw)
    pub company_status: String,
    /// 加权净资产收益率 % (jqjzcsyl)
    pub weighted_roe: f64,
    /// 通达信研究行业1 (hy1)
    pub industry_1: String,
    /// 通达信研究行业2 (hy2)
    pub industry_2: String,
    /// 主营业务 (T017)
    pub main_business: String,
    /// 最近质押登记日 (zyrq)
    pub pledge_date: String,
    /// 质押比例 % (zygf)
    pub pledge_pct: f64,
    /// 总质押股数 (zzygf)
    pub total_pledge_shares: f64,
    /// 证监会行业 (zzhy)
    pub csrc_industry: String,
    /// 行业平均市盈率 (pjgdsyll)
    pub industry_pe: f64,
    /// 行业 PE 更新日期 (gxrq)
    pub industry_pe_date: String,
    /// 市场人气排名 (scpmdela RS0.pm)
    pub market_rank: i64,
    /// 行业人气排名 (hypmdelat RS0.pm)
    pub industry_rank: i64,
}

/// 主要指标历史序列（每份报告期一条记录）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsKpiRow {
    pub stock_code: String,
    /// 报告期 T002
    pub report_date: String,
    /// 每股收益 T056
    pub eps: f64,
    /// 扣非每股收益 T059
    pub deducted_eps: f64,
    /// 加权 ROE % T064
    pub roe: f64,
    /// 每股经营现金流 T062
    pub op_cf_ps: f64,
    /// 每股未分配利润 T060
    pub undist_profit_ps: f64,
    /// 每股资本公积 T061
    pub capres_ps: f64,
    /// 每股净资产（扣非）T123
    pub bvps: f64,
    /// 净利润 T022
    pub net_profit: f64,
    /// 营业收入 T003
    pub revenue: f64,
    /// 扣非净利润 T016
    pub deducted_net_profit: f64,
    /// 归母净利润 T019
    pub parent_net_profit: f64,
}

/// 概念题材（来自 gsgy RS2，一股多条）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsConceptRow {
    pub stock_code: String,
    /// 概念 ID T001
    pub concept_id: String,
    /// 概念名称 T002
    pub concept_name: String,
}

/// 公司大事记录
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsEventRow {
    pub stock_code: String,
    pub event_date: String,
    pub event_type: String,
    pub event_content: String,
}

/// 资讯（新闻/公告/研报/路演，按 news_type 区分）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsNewsRow {
    pub stock_code: String,
    pub news_type: String,
    pub pub_date: String,
    pub title: String,
    pub rec_id: String,
}

/// 机构调研
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsJgdyRow {
    pub stock_code: String,
    pub visit_date: String,
    pub org_name: String,
    pub org_type: String,
    pub visit_type: String,
    pub contact_person: String,
}

/// 互动问答（正式名称 hdwd）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsHdwdRow {
    pub stock_code: String,
    /// 提问日期 rq2
    pub question_date: String,
    /// 回答日期 rq
    pub answer_date: String,
    /// 问题内容 T005
    pub question: String,
    /// 回答内容 T007
    pub answer: String,
    /// 问题类型 T010（通常为 "wd"）
    pub q_type: String,
}

/// 可比公司（来自 tdxf10_gg_kbgs，一股多条）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ZxtsPeerRow {
    /// 被查询股票
    pub stock_code: String,
    /// 可比公司代码 t003
    pub peer_code: String,
    /// 可比公司名称 T004
    pub peer_name: String,
    /// 市场类型：国内 / 境外 / 其它
    pub market_type: String,
}

/// fetch() 全部返回数据
pub struct ZxtsData {
    pub overviews: Vec<ZxtsOverviewRow>,
    pub kpi: Vec<ZxtsKpiRow>,
    pub concepts: Vec<ZxtsConceptRow>,
    pub events: Vec<ZxtsEventRow>,
    pub news: Vec<ZxtsNewsRow>,
    pub jgdy: Vec<ZxtsJgdyRow>,
    pub hdwd: Vec<ZxtsHdwdRow>,
    pub peers: Vec<ZxtsPeerRow>,
}

/// 抓取最新提示全部数据
pub async fn fetch(client: &TqLexClient, code: &str) -> Result<ZxtsData, F10Error> {
    // 12 个子请求全部并行发出
    let (
        gsgy_res,
        gsds,
        company_news,
        company_announcements,
        company_reports,
        roadshow_events,
        jgdy,
        hdwd,
        zxts_kpi,
        scpm,
        hypm,
        kbgs,
    ) = tokio::join!(
        async { client.post("tdxf10_gg_zxts", &[code, "gsgy", ""]).await },
        async { client.post("tdxf10_gg_zxts", &[code, "gsds", ""]).await },
        async { client.post("tdxf10_gg_zxts", &[code, "gsxw", ""]).await },
        async { client.post("tdxf10_gg_zxts", &[code, "gsgg", ""]).await },
        async { client.post("tdxf10_gg_zxts", &[code, "gsyj", ""]).await },
        async { client.post("tdxf10_gg_zxts", &[code, "lyhd", ""]).await },
        async { client.post("tdxf10_gg_comreq", &["jgdy", code]).await },
        async {
            client
                .post("tdxf10_gg_zxtstzzhd", &[code, "1", "1", "5"])
                .await
        },
        async { client.post("tdxf10_gg_zxts", &[code, "zxts", ""]).await },
        async {
            client
                .post("tdxf10_gg_zxts_rqpm", &[code, "scpmdela"])
                .await
        },
        async {
            client
                .post("tdxf10_gg_zxts_rqpm", &[code, "hypmdelat"])
                .await
        },
        async { client.post("tdxf10_gg_kbgs", &["1", code]).await },
    );
    let gsgy = gsgy_res?;
    let gsds = gsds.unwrap_or_else(|e| {
        warn!("gsds: {e}");
        crate::client::TqLexResponse::default()
    });
    let company_news = company_news.unwrap_or_default();
    let company_announcements = company_announcements.unwrap_or_default();
    let company_reports = company_reports.unwrap_or_default();
    let roadshow_events = roadshow_events.unwrap_or_default();
    let jgdy = jgdy.unwrap_or_default();
    let hdwd = hdwd.unwrap_or_default();
    let zxts_kpi = zxts_kpi.unwrap_or_default();
    let scpm = scpm.unwrap_or_default();
    let hypm = hypm.unwrap_or_default();
    let kbgs = kbgs.unwrap_or_default();

    // ── 解析公司概要 ────────────────────────────────────────────────────────
    // RS0: kfjlr, jqjzcsyl, jjrq, gsdw
    let rs0 = gsgy.result_sets.first();
    // RS1: T017(主营业务), hy1, hy2
    let rs1 = gsgy.result_sets.get(1);
    // RS2: rec_id, T002(概念名), T001(概念ID)
    let rs2 = gsgy.result_sets.get(2);
    // RS4: 财务 KPI：mgsy/mgjzc/yysr/gmjlr/mll/zcfzl/kfjlrtbzzl/yysrtbzzl/gmjlrtbzzl/mgzbgj/mgwfplr/mgjjxjl/bgq
    let rs4 = gsgy.result_sets.get(4);
    // RS5: 质押：zyrq/zygf/zzygf
    let rs5 = gsgy.result_sets.get(5);
    // RS6: 估值：sylttm/syllyr/sjl/zsz/zgb/ltag
    let rs6 = gsgy.result_sets.get(6);
    // RS8: 证监会行业：name/zzhy/pjgdsyll/gxrq
    let rs8 = gsgy.result_sets.get(8);

    let market_rank = scpm
        .result_sets
        .first()
        .and_then(|r| get_i64(r, 0, "pm"))
        .unwrap_or(0);
    let industry_rank = hypm
        .result_sets
        .first()
        .and_then(|r| get_i64(r, 0, "pm"))
        .unwrap_or(0);

    let mut overviews = vec![];
    if let Some(fin) = rs4 {
        if !fin.content.is_empty() {
            overviews.push(ZxtsOverviewRow {
                stock_code: code.to_string(),
                report_date: get_str(fin, 0, "bgq").unwrap_or_default(),
                eps: get_f64(fin, 0, "mgsy").unwrap_or(0.0),
                bvps: get_f64(fin, 0, "mgjzc").unwrap_or(0.0),
                revenue: get_f64(fin, 0, "yysr").unwrap_or(0.0),
                net_profit: get_f64(fin, 0, "gmjlr").unwrap_or(0.0),
                gross_margin: get_f64(fin, 0, "mll").unwrap_or(0.0),
                debt_ratio: get_f64(fin, 0, "zcfzl").unwrap_or(0.0),
                undist_profit_ps: get_f64(fin, 0, "mgwfplr").unwrap_or(0.0),
                capres_ps: get_f64(fin, 0, "mgzbgj").unwrap_or(0.0),
                op_cf_ps: get_f64(fin, 0, "mgjjxjl").unwrap_or(0.0),
                revenue_yoy: get_f64(fin, 0, "yysrtbzzl").unwrap_or(0.0),
                profit_yoy: get_f64(fin, 0, "gmjlrtbzzl").unwrap_or(0.0),
                deducted_profit_yoy: get_f64(fin, 0, "kfjlrtbzzl").unwrap_or(0.0),
                pe_ttm: rs6.and_then(|r| get_f64(r, 0, "sylttm")).unwrap_or(0.0),
                pe_lyr: rs6.and_then(|r| get_f64(r, 0, "syllyr")).unwrap_or(0.0),
                pb: rs6.and_then(|r| get_f64(r, 0, "sjl")).unwrap_or(0.0),
                total_market_cap: rs6.and_then(|r| get_f64(r, 0, "zsz")).unwrap_or(0.0),
                total_shares: rs6.and_then(|r| get_f64(r, 0, "zgb")).unwrap_or(0.0),
                float_a_shares: rs6.and_then(|r| get_f64(r, 0, "ltag")).unwrap_or(0.0),
                company_status: rs0.and_then(|r| get_str(r, 0, "gsdw")).unwrap_or_default(),
                weighted_roe: rs0.and_then(|r| get_f64(r, 0, "jqjzcsyl")).unwrap_or(0.0),
                industry_1: rs1.and_then(|r| get_str(r, 0, "hy1")).unwrap_or_default(),
                industry_2: rs1.and_then(|r| get_str(r, 0, "hy2")).unwrap_or_default(),
                main_business: rs1.and_then(|r| get_str(r, 0, "T017")).unwrap_or_default(),
                pledge_date: rs5.and_then(|r| get_str(r, 0, "zyrq")).unwrap_or_default(),
                pledge_pct: rs5.and_then(|r| get_f64(r, 0, "zygf")).unwrap_or(0.0),
                total_pledge_shares: rs5.and_then(|r| get_f64(r, 0, "zzygf")).unwrap_or(0.0),
                csrc_industry: rs8.and_then(|r| get_str(r, 0, "zzhy")).unwrap_or_default(),
                industry_pe: rs8.and_then(|r| get_f64(r, 0, "pjgdsyll")).unwrap_or(0.0),
                industry_pe_date: rs8.and_then(|r| get_str(r, 0, "gxrq")).unwrap_or_default(),
                market_rank,
                industry_rank,
            });
        }
    }

    // ── 解析概念题材（RS2） ──────────────────────────────────────────────────
    let mut concepts = vec![];
    if let Some(rs) = rs2 {
        for i in 0..rs.content.len() {
            let concept_id = get_str(rs, i, "T001").unwrap_or_default();
            let concept_name = get_str(rs, i, "T002").unwrap_or_default();
            if !concept_name.is_empty() {
                concepts.push(ZxtsConceptRow {
                    stock_code: code.to_string(),
                    concept_id,
                    concept_name,
                });
            }
        }
    }

    // ── 解析主要指标多期（["zxts",""] RS1）──────────────────────────────────
    let mut kpi = vec![];
    if let Some(rs) = zxts_kpi.result_sets.get(1) {
        for i in 0..rs.content.len() {
            let report_date = get_str(rs, i, "T002").unwrap_or_default();
            if !report_date.is_empty() {
                kpi.push(ZxtsKpiRow {
                    stock_code: code.to_string(),
                    report_date,
                    eps: get_f64(rs, i, "T056").unwrap_or(0.0),
                    deducted_eps: get_f64(rs, i, "T059").unwrap_or(0.0),
                    roe: get_f64(rs, i, "T064").unwrap_or(0.0),
                    op_cf_ps: get_f64(rs, i, "T062").unwrap_or(0.0),
                    undist_profit_ps: get_f64(rs, i, "T060").unwrap_or(0.0),
                    capres_ps: get_f64(rs, i, "T061").unwrap_or(0.0),
                    bvps: get_f64(rs, i, "T123").unwrap_or(0.0),
                    net_profit: get_f64(rs, i, "T022").unwrap_or(0.0),
                    revenue: get_f64(rs, i, "T003").unwrap_or(0.0),
                    deducted_net_profit: get_f64(rs, i, "T016").unwrap_or(0.0),
                    parent_net_profit: get_f64(rs, i, "T019").unwrap_or(0.0),
                });
            }
        }
    }

    // ── 解析公司大事（RS0: T001=类型, T002=内容, T003=日期）───────────────
    let mut events = vec![];
    if let Some(rs) = gsds.result_sets.first() {
        for i in 0..rs.content.len() {
            let event_date = get_str(rs, i, "T003").unwrap_or_default();
            let event_type = get_str(rs, i, "T001").unwrap_or_default();
            let event_content = get_str(rs, i, "T002").unwrap_or_default();
            if !event_date.is_empty() {
                events.push(ZxtsEventRow {
                    stock_code: code.to_string(),
                    event_date,
                    event_type,
                    event_content,
                });
            }
        }
    }

    // ── 解析资讯（gsxw/gsgg/gsyj 用 "rq"，lyhd 用 "Issue_date"）───────────
    let mut news_rows: Vec<ZxtsNewsRow> = vec![];
    let parse_news = |resp: &crate::client::TqLexResponse, ntype: &str| -> Vec<ZxtsNewsRow> {
        let mut out = vec![];
        if let Some(rs) = resp.result_sets.first() {
            for i in 0..rs.content.len() {
                let pub_date = get_str(rs, i, "rq")
                    .or_else(|| get_str(rs, i, "Issue_date"))
                    .unwrap_or_default();
                let title = get_str(rs, i, "Title").unwrap_or_default();
                let rec_id = get_str(rs, i, "rec_id").unwrap_or_default();
                if !title.is_empty() {
                    out.push(ZxtsNewsRow {
                        stock_code: code.to_string(),
                        news_type: ntype.to_string(),
                        pub_date,
                        title,
                        rec_id,
                    });
                }
            }
        }
        out
    };
    news_rows.extend(parse_news(&company_news, "新闻"));
    news_rows.extend(parse_news(&company_announcements, "公告"));
    news_rows.extend(parse_news(&company_reports, "研报"));
    news_rows.extend(parse_news(&roadshow_events, "路演"));

    // ── 解析机构调研 ─────────────────────────────────────────────────────────
    let mut jgdy_rows = vec![];
    if let Some(rs) = jgdy.result_sets.first() {
        for i in 0..rs.content.len() {
            jgdy_rows.push(ZxtsJgdyRow {
                stock_code: code.to_string(),
                visit_date: get_str(rs, i, "T003").unwrap_or_default(),
                org_name: get_str(rs, i, "T004").unwrap_or_default(),
                org_type: get_str(rs, i, "T005").unwrap_or_default(),
                visit_type: get_str(rs, i, "T006").unwrap_or_default(),
                contact_person: get_str(rs, i, "T007").unwrap_or_default(),
            });
        }
    }

    // ── 解析互动问答（T005=问题, T007=回答, rq2=提问日期, rq=回答日期）──────
    let mut hdwd_rows = vec![];
    if let Some(rs) = hdwd.result_sets.first() {
        for i in 0..rs.content.len() {
            let question = get_str(rs, i, "T005").unwrap_or_default();
            if !question.is_empty() {
                hdwd_rows.push(ZxtsHdwdRow {
                    stock_code: code.to_string(),
                    question_date: get_str(rs, i, "rq2").unwrap_or_default(),
                    answer_date: get_str(rs, i, "rq").unwrap_or_default(),
                    question,
                    answer: get_str(rs, i, "T007").unwrap_or_default(),
                    q_type: get_str(rs, i, "T010").unwrap_or_default(),
                });
            }
        }
    }

    // ── 解析可比公司（["1","code"]: RS0=国内, RS1=国内, RS2=国内, RS3=境外, RS4=其它）
    let mut peers = vec![];
    let market_types = ["国内", "国内", "国内", "境外", "其它"];
    for (rs_idx, &mtype) in market_types.iter().enumerate() {
        if let Some(rs) = kbgs.result_sets.get(rs_idx) {
            for i in 0..rs.content.len() {
                let peer_code = get_str(rs, i, "t003").unwrap_or_default();
                let peer_name = get_str(rs, i, "T004").unwrap_or_default();
                if !peer_code.is_empty() || !peer_name.is_empty() {
                    peers.push(ZxtsPeerRow {
                        stock_code: code.to_string(),
                        peer_code,
                        peer_name,
                        market_type: mtype.to_string(),
                    });
                }
            }
        }
    }

    debug!(
        "zxts {code}: overviews={} kpi={} concepts={} events={} news={} jgdy={} hdwd={} peers={}",
        overviews.len(),
        kpi.len(),
        concepts.len(),
        events.len(),
        news_rows.len(),
        jgdy_rows.len(),
        hdwd_rows.len(),
        peers.len()
    );

    Ok(ZxtsData {
        overviews,
        kpi,
        concepts,
        events,
        news: news_rows,
        jgdy: jgdy_rows,
        hdwd: hdwd_rows,
        peers,
    })
}

// ── 写入函数 ─────────────────────────────────────────────────────────────────

pub async fn insert_overviews(ch: &Client, rows: &[ZxtsOverviewRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsOverviewRow>("f10_zxts_overview").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_kpi(ch: &Client, rows: &[ZxtsKpiRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsKpiRow>("f10_zxts_kpi").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_concepts(ch: &Client, rows: &[ZxtsConceptRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsConceptRow>("f10_zxts_concept").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_events(ch: &Client, rows: &[ZxtsEventRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsEventRow>("f10_zxts_events").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_news(ch: &Client, rows: &[ZxtsNewsRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsNewsRow>("f10_zxts_news").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_jgdy(ch: &Client, rows: &[ZxtsJgdyRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsJgdyRow>("f10_zxts_jgdy").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_hdwd(ch: &Client, rows: &[ZxtsHdwdRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsHdwdRow>("f10_zxts_hdwd").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_peers(ch: &Client, rows: &[ZxtsPeerRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<ZxtsPeerRow>("f10_zxts_peer").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

// ── 查询函数 ─────────────────────────────────────────────────────────────────

/// 查询公司概要（含市场/行业排名、质押、证监会行业等完整字段）
pub async fn query_overview(ch: &Client, code: &str) -> Result<Vec<ZxtsOverviewRow>, F10Error> {
    Ok(ch
        .query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_overview FINAL WHERE stock_code = ?")
        .bind(code)
        .fetch_all::<ZxtsOverviewRow>()
        .await?)
}

/// 查询主要指标历史序列（按报告期降序）
pub async fn query_kpi(ch: &Client, code: &str) -> Result<Vec<ZxtsKpiRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_kpi FINAL WHERE stock_code = ? ORDER BY report_date DESC")
        .bind(code).fetch_all::<ZxtsKpiRow>().await?)
}

/// 查询概念题材
pub async fn query_concepts(ch: &Client, code: &str) -> Result<Vec<ZxtsConceptRow>, F10Error> {
    Ok(ch
        .query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_concept FINAL WHERE stock_code = ?")
        .bind(code)
        .fetch_all::<ZxtsConceptRow>()
        .await?)
}

/// 查询公司大事（按日期降序）
pub async fn query_events(ch: &Client, code: &str) -> Result<Vec<ZxtsEventRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_events FINAL WHERE stock_code = ? ORDER BY event_date DESC")
        .bind(code).fetch_all::<ZxtsEventRow>().await?)
}

/// 查询资讯（按类型和日期降序）
pub async fn query_news(
    ch: &Client,
    code: &str,
    news_type: &str,
) -> Result<Vec<ZxtsNewsRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_news FINAL WHERE stock_code = ? AND news_type = ? ORDER BY pub_date DESC")
        .bind(code).bind(news_type).fetch_all::<ZxtsNewsRow>().await?)
}

/// 查询机构调研
pub async fn query_jgdy(ch: &Client, code: &str) -> Result<Vec<ZxtsJgdyRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_jgdy FINAL WHERE stock_code = ? ORDER BY visit_date DESC")
        .bind(code).fetch_all::<ZxtsJgdyRow>().await?)
}

/// 查询互动问答（按回答日期降序）
pub async fn query_hdwd(ch: &Client, code: &str) -> Result<Vec<ZxtsHdwdRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_hdwd FINAL WHERE stock_code = ? ORDER BY answer_date DESC")
        .bind(code).fetch_all::<ZxtsHdwdRow>().await?)
}

/// 查询可比公司
pub async fn query_peers(ch: &Client, code: &str) -> Result<Vec<ZxtsPeerRow>, F10Error> {
    Ok(ch
        .query("SELECT * EXCEPT(fetched_at) FROM f10_zxts_peer FINAL WHERE stock_code = ?")
        .bind(code)
        .fetch_all::<ZxtsPeerRow>()
        .await?)
}
