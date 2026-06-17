//! 批量抓取运行器：针对单支股票抓取全部 14 个页面并写入 ClickHouse
use std::sync::Arc;

use clickhouse::Client;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{info, warn};

use crate::client::TqLexClient;
use crate::error::F10Error;
use crate::pages::{
    cwfx, fhrz, gbjg, gdyj, gsgk, gszx, hyfx, jyds, jyfx, rdtc, ybpj, zbyz, zlcc, zxts,
};


/// 串行逐只抓取多只股票（每只股票内部 14 个页面并行），带进度条
pub async fn run_batch(
    api: Arc<TqLexClient>,
    ck: Arc<Client>,
    codes: Vec<String>,
) -> Result<(), anyhow::Error> {
    let total = codes.len();
    info!("批量抓取共 {total} 只股票");

    // 进度条
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    for code in codes {
        pb.set_message(format!("抓取 {code}…"));
        if let Err(e) = fetch_and_insert_one(&api, &ck, &code).await {
            warn!("{code} 抓取失败: {e}");
            pb.println(format!("⚠ {code} 失败: {e}"));
        }
        pb.inc(1);
    }

    pb.finish_with_message("全部完成");
    info!("全部 {total} 只股票处理完成");
    Ok(())
}

/// 抓取单只股票的所有 F10 数据并写入 ClickHouse
pub async fn fetch_and_insert_one(
    api: &TqLexClient,
    ck: &Client,
    code: &str,
) -> Result<(), F10Error> {
    info!("开始抓取 {code}");

    // 14 个页面并行抓取
    let (
        r_zxts,
        r_jyds,
        r_gsgk,
        r_gbjg,
        r_gdyj,
        r_fhrz,
        r_cwfx,
        r_jyfx,
        r_zbyz,
        r_ybpj,
        r_rdtc,
        r_gszx,
        r_zlcc,
        r_hyfx,
    ) = tokio::join!(
        zxts::fetch(api, code),
        jyds::fetch(api, code),
        gsgk::fetch(api, code),
        gbjg::fetch(api, code),
        gdyj::fetch(api, code),
        fhrz::fetch(api, code),
        cwfx::fetch(api, code),
        jyfx::fetch(api, code),
        zbyz::fetch(api, code),
        ybpj::fetch(api, code),
        rdtc::fetch(api, code),
        gszx::fetch(api, code),
        zlcc::fetch(api, code),
        hyfx::fetch(api, code),
    );

    // 1. 最新提示
    match r_zxts {
        Ok(d) => {
            zxts::insert_overviews(ck, &d.overviews)
                .await
                .unwrap_or_else(|e| warn!("zxts overviews insert: {e}"));
            zxts::insert_kpi(ck, &d.kpi)
                .await
                .unwrap_or_else(|e| warn!("zxts kpi insert: {e}"));
            zxts::insert_concepts(ck, &d.concepts)
                .await
                .unwrap_or_else(|e| warn!("zxts concepts insert: {e}"));
            zxts::insert_events(ck, &d.events)
                .await
                .unwrap_or_else(|e| warn!("zxts events insert: {e}"));
            zxts::insert_news(ck, &d.news)
                .await
                .unwrap_or_else(|e| warn!("zxts news insert: {e}"));
            zxts::insert_jgdy(ck, &d.jgdy)
                .await
                .unwrap_or_else(|e| warn!("zxts jgdy insert: {e}"));
            zxts::insert_hdwd(ck, &d.hdwd)
                .await
                .unwrap_or_else(|e| warn!("zxts hdwd insert: {e}"));
            zxts::insert_peers(ck, &d.peers)
                .await
                .unwrap_or_else(|e| warn!("zxts peers insert: {e}"));
        }
        Err(e) => warn!("zxts fetch {code}: {e}"),
    }

    // 2. 资金动向
    match r_jyds {
        Ok((blocks, margins, flows, tigers, northbound)) => {
            jyds::insert_block_trades(ck, &blocks)
                .await
                .unwrap_or_else(|e| warn!("jyds blocks: {e}"));
            jyds::insert_margins(ck, &margins)
                .await
                .unwrap_or_else(|e| warn!("jyds margins: {e}"));
            jyds::insert_moneyflows(ck, &flows)
                .await
                .unwrap_or_else(|e| warn!("jyds flows: {e}"));
            jyds::insert_dragon_tigers(ck, &tigers)
                .await
                .unwrap_or_else(|e| warn!("jyds tigers: {e}"));
            jyds::insert_northbound(ck, &northbound)
                .await
                .unwrap_or_else(|e| warn!("jyds northbound: {e}"));
        }
        Err(e) => warn!("jyds fetch {code}: {e}"),
    }

    // 3. 基本情况
    match r_gsgk {
        Ok((basics, employees, emp_structs, rds, subs)) => {
            gsgk::insert_basics(ck, &basics)
                .await
                .unwrap_or_else(|e| warn!("gsgk basics: {e}"));
            gsgk::insert_employees(ck, &employees)
                .await
                .unwrap_or_else(|e| warn!("gsgk employees: {e}"));
            gsgk::insert_emp_structs(ck, &emp_structs)
                .await
                .unwrap_or_else(|e| warn!("gsgk emp_structs: {e}"));
            gsgk::insert_rds(ck, &rds)
                .await
                .unwrap_or_else(|e| warn!("gsgk rds: {e}"));
            gsgk::insert_subsidiaries(ck, &subs)
                .await
                .unwrap_or_else(|e| warn!("gsgk subsidiaries: {e}"));
        }
        Err(e) => warn!("gsgk fetch {code}: {e}"),
    }

    // 4. 股本结构
    match r_gbjg {
        Ok((structs, changes, unlocks, buybacks)) => {
            gbjg::insert_share_structs(ck, &structs)
                .await
                .unwrap_or_else(|e| warn!("gbjg structs: {e}"));
            gbjg::insert_changes(ck, &changes)
                .await
                .unwrap_or_else(|e| warn!("gbjg changes: {e}"));
            gbjg::insert_unlocks(ck, &unlocks)
                .await
                .unwrap_or_else(|e| warn!("gbjg unlocks: {e}"));
            gbjg::insert_buybacks(ck, &buybacks)
                .await
                .unwrap_or_else(|e| warn!("gbjg buybacks: {e}"));
        }
        Err(e) => warn!("gbjg fetch {code}: {e}"),
    }

    // 5. 股东研究
    match r_gdyj {
        Ok((
            ctrl,
            counts,
            industry,
            top10_float,
            top10_all,
            changes,
            trend,
            inst_summary,
            inst_detail,
        )) => {
            gdyj::insert_controlling(ck, &ctrl)
                .await
                .unwrap_or_else(|e| warn!("gdyj ctrl: {e}"));
            gdyj::insert_holder_counts(ck, &counts)
                .await
                .unwrap_or_else(|e| warn!("gdyj counts: {e}"));
            gdyj::insert_industry_holders(ck, &industry)
                .await
                .unwrap_or_else(|e| warn!("gdyj industry: {e}"));
            gdyj::insert_top10_float(ck, &top10_float)
                .await
                .unwrap_or_else(|e| warn!("gdyj top10_float: {e}"));
            gdyj::insert_top10_all(ck, &top10_all)
                .await
                .unwrap_or_else(|e| warn!("gdyj top10_all: {e}"));
            gdyj::insert_hold_changes(ck, &changes)
                .await
                .unwrap_or_else(|e| warn!("gdyj changes: {e}"));
            gdyj::insert_inst_trend(ck, &trend)
                .await
                .unwrap_or_else(|e| warn!("gdyj trend: {e}"));
            gdyj::insert_inst_summaries(ck, &inst_summary)
                .await
                .unwrap_or_else(|e| warn!("gdyj inst summary: {e}"));
            gdyj::insert_inst_details(ck, &inst_detail)
                .await
                .unwrap_or_else(|e| warn!("gdyj inst detail: {e}"));
        }
        Err(e) => warn!("gdyj fetch {code}: {e}"),
    }

    // 6. 分红融资
    match r_fhrz {
        Ok((divs, rights, addissues)) => {
            fhrz::insert_dividends(ck, &divs)
                .await
                .unwrap_or_else(|e| warn!("fhrz divs: {e}"));
            fhrz::insert_rights(ck, &rights)
                .await
                .unwrap_or_else(|e| warn!("fhrz rights: {e}"));
            fhrz::insert_addissues(ck, &addissues)
                .await
                .unwrap_or_else(|e| warn!("fhrz addissues: {e}"));
        }
        Err(e) => warn!("fhrz fetch {code}: {e}"),
    }

    // 7. 财务分析
    match r_cwfx {
        Ok((indicators, reports, profits, research)) => {
            cwfx::insert_indicators(ck, &indicators)
                .await
                .unwrap_or_else(|e| warn!("cwfx indicators: {e}"));
            cwfx::insert_reports(ck, &reports)
                .await
                .unwrap_or_else(|e| warn!("cwfx reports: {e}"));
            cwfx::insert_profits(ck, &profits)
                .await
                .unwrap_or_else(|e| warn!("cwfx profits: {e}"));
            cwfx::insert_research(ck, &research)
                .await
                .unwrap_or_else(|e| warn!("cwfx research: {e}"));
        }
        Err(e) => warn!("cwfx fetch {code}: {e}"),
    }

    // 8. 经营分析
    match r_jyfx {
        Ok((main_biz, customers, suppliers, oper_data)) => {
            jyfx::insert_main_bizs(ck, &main_biz)
                .await
                .unwrap_or_else(|e| warn!("jyfx main biz: {e}"));
            jyfx::insert_top5_customers(ck, &customers)
                .await
                .unwrap_or_else(|e| warn!("jyfx customers: {e}"));
            jyfx::insert_top5_suppliers(ck, &suppliers)
                .await
                .unwrap_or_else(|e| warn!("jyfx suppliers: {e}"));
            jyfx::insert_oper_data(ck, &oper_data)
                .await
                .unwrap_or_else(|e| warn!("jyfx oper data: {e}"));
        }
        Err(e) => warn!("jyfx fetch {code}: {e}"),
    }

    // 9. 资本运作
    match r_zbyz {
        Ok((fundraises, violations, majors, transfers, share_controls)) => {
            zbyz::insert_fundraises(ck, &fundraises)
                .await
                .unwrap_or_else(|e| warn!("zbyz fundraises: {e}"));
            zbyz::insert_violations(ck, &violations)
                .await
                .unwrap_or_else(|e| warn!("zbyz violations: {e}"));
            zbyz::insert_major_events(ck, &majors)
                .await
                .unwrap_or_else(|e| warn!("zbyz majors: {e}"));
            zbyz::insert_transfers(ck, &transfers)
                .await
                .unwrap_or_else(|e| warn!("zbyz transfers: {e}"));
            zbyz::insert_share_controls(ck, &share_controls)
                .await
                .unwrap_or_else(|e| warn!("zbyz share_controls: {e}"));
        }
        Err(e) => warn!("zbyz fetch {code}: {e}"),
    }

    // 10. 研报评级
    match r_ybpj {
        Ok((stats, forecasts, reports)) => {
            ybpj::insert_rating_stats(ck, &stats)
                .await
                .unwrap_or_else(|e| warn!("ybpj stats: {e}"));
            ybpj::insert_forecasts(ck, &forecasts)
                .await
                .unwrap_or_else(|e| warn!("ybpj forecasts: {e}"));
            ybpj::insert_reports(ck, &reports)
                .await
                .unwrap_or_else(|e| warn!("ybpj reports: {e}"));
        }
        Err(e) => warn!("ybpj fetch {code}: {e}"),
    }

    // 11. 热点题材
    match r_rdtc {
        Ok((themes, events, logics, concepts)) => {
            rdtc::insert_themes(ck, &themes)
                .await
                .unwrap_or_else(|e| warn!("rdtc themes: {e}"));
            rdtc::insert_events(ck, &events)
                .await
                .unwrap_or_else(|e| warn!("rdtc events: {e}"));
            rdtc::insert_logics(ck, &logics)
                .await
                .unwrap_or_else(|e| warn!("rdtc logics: {e}"));
            rdtc::insert_concepts(ck, &concepts)
                .await
                .unwrap_or_else(|e| warn!("rdtc concepts: {e}"));
        }
        Err(e) => warn!("rdtc fetch {code}: {e}"),
    }

    // 12. 公司资讯
    match r_gszx {
        Ok((news, reports)) => {
            gszx::insert_news(ck, &news)
                .await
                .unwrap_or_else(|e| warn!("gszx news: {e}"));
            gszx::insert_reports(ck, &reports)
                .await
                .unwrap_or_else(|e| warn!("gszx reports: {e}"));
        }
        Err(e) => warn!("gszx fetch {code}: {e}"),
    }

    // 13. 主力持仓
    match r_zlcc {
        Ok((timelines, by_types, details)) => {
            zlcc::insert_timelines(ck, &timelines)
                .await
                .unwrap_or_else(|e| warn!("zlcc timelines: {e}"));
            zlcc::insert_by_types(ck, &by_types)
                .await
                .unwrap_or_else(|e| warn!("zlcc by types: {e}"));
            zlcc::insert_details(ck, &details)
                .await
                .unwrap_or_else(|e| warn!("zlcc details: {e}"));
        }
        Err(e) => warn!("zlcc fetch {code}: {e}"),
    }

    // 14. 行业分析
    match r_hyfx {
        Ok((news, reports, market, size, val, fin, div)) => {
            hyfx::insert_industry_news(ck, &news)
                .await
                .unwrap_or_else(|e| warn!("hyfx news: {e}"));
            hyfx::insert_industry_reports(ck, &reports)
                .await
                .unwrap_or_else(|e| warn!("hyfx reports: {e}"));
            hyfx::insert_market_ranks(ck, &market)
                .await
                .unwrap_or_else(|e| warn!("hyfx market: {e}"));
            hyfx::insert_size_ranks(ck, &size)
                .await
                .unwrap_or_else(|e| warn!("hyfx size: {e}"));
            hyfx::insert_valuation_ranks(ck, &val)
                .await
                .unwrap_or_else(|e| warn!("hyfx val: {e}"));
            hyfx::insert_financial_ranks(ck, &fin)
                .await
                .unwrap_or_else(|e| warn!("hyfx fin: {e}"));
            hyfx::insert_dividend_ranks(ck, &div)
                .await
                .unwrap_or_else(|e| warn!("hyfx div: {e}"));
        }
        Err(e) => warn!("hyfx fetch {code}: {e}"),
    }

    info!("{code} 全部抓取完成");
    Ok(())
}
