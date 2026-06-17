//! 步骤 3：基本情况 (gg_gsgk)
//!
//! 涵盖：公司基本情况、发行交易、员工效益、员工构成、研发投入、参股控股公司

#![allow(missing_docs, clippy::doc_markdown)]

use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::client::{get_f64, get_i64, get_str, TqLexClient};
use crate::error::F10Error;

/// 公司基本信息
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GsgkBasicRow {
    pub stock_code: String,
    pub company_name: String,
    pub full_name: String,
    pub english_name: String,
    pub reg_capital: String,
    pub setup_date: String,
    pub list_date: String,
    pub legal_person: String,
    pub secretary: String,
    pub address: String,
    pub website: String,
    pub phone: String,
    pub fax: String,
    pub email: String,
    pub main_business: String,
    pub business_scope: String,
    pub reg_no: String,
    pub org_code: String,
}

/// 员工构成（专业岗位/教育程度分类）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GsgkEmployeeStructRow {
    pub stock_code: String,
    pub report_date: String,
    pub category_type: String, // 专业岗位构成/教育程度
    pub item_name: String,
    pub head_count: i64,
    pub pct: f64,
}

/// 员工效益（人均）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GsgkEmployeeRow {
    pub stock_code: String,
    pub year_date: String,
    /// 人均产值(元/人)
    pub value_per_person: f64,
    /// 营收规模(元)
    pub total_revenue: f64,
    /// 净利润(元)
    pub total_profit: f64,
}

/// 研发投入
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GsgkRdRow {
    pub stock_code: String,
    pub year_date: String,
    pub rd_staff: i64,
    pub rd_amount: f64,
    pub rd_pct_revenue: f64,
}

/// 参股控股子公司
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GsgkSubsidiaryRow {
    pub stock_code: String,
    pub sub_name: String,
    pub reg_capital: String,
    pub hold_pct: f64,
    pub hold_type: String,
    pub main_business: String,
}

/// 抓取基本情况全部数据
pub async fn fetch(
    client: &TqLexClient,
    code: &str,
) -> Result<
    (
        Vec<GsgkBasicRow>,
        Vec<GsgkEmployeeRow>,
        Vec<GsgkEmployeeStructRow>,
        Vec<GsgkRdRow>,
        Vec<GsgkSubsidiaryRow>,
    ),
    F10Error,
> {
    // 6 个子请求全部并行发出
    let (basic_res, employee, emp_struct, rd, subs, listing) = tokio::join!(
        async { client.post("tdxf10_gg_gsgk", &["0", code, ""]).await },
        async { client.post("tdxf10_gg_gsgk", &["5", code, ""]).await },
        async { client.post("tdxf10_gg_gsgk", &["4", code, ""]).await },
        async { client.post("tdxf10_gg_gsgk", &["6", code, ""]).await },
        async { client.post("tdxf10_gg_gsgk", &["3", code, ""]).await },
        async { client.post("tdxf10_gg_gsgk", &["8", code, ""]).await },
    );
    let basic = basic_res?;
    let employee = employee.unwrap_or_default();
    let emp_struct = emp_struct.unwrap_or_default();
    let rd = rd.unwrap_or_default();
    let subs = subs.unwrap_or_default();
    let listing = listing.unwrap_or_default();

    // 基本情况：RS0 = 单行扁平记录
    // kwjc=快文简称(可能为null), T003=全称, T006=英文名, url=网址
    // T008=董秘, T009=注册地址, T010=注册资本(数字), T017=主营业务
    // T018=经营范围, T024=电话, T026=邮箱, dsz=董事长
    // yjhy=行业, shxydm=统一信用代码
    let mut list_date = String::new();
    if let Some(rs) = listing.result_sets.first() {
        if !rs.content.is_empty() {
            list_date = get_str(rs, 0, "T031").unwrap_or_default();
        }
    }

    let mut basics = vec![];
    if let Some(rs) = basic.result_sets.first() {
        if !rs.content.is_empty() {
            // T010 is a float number (注册资本), convert to string
            let reg_capital = {
                let v = get_f64(rs, 0, "T010").unwrap_or(0.0);
                if v.abs() > f64::EPSILON {
                    format!("{v:.4}")
                } else {
                    String::new()
                }
            };
            basics.push(GsgkBasicRow {
                stock_code: code.to_string(),
                company_name: get_str(rs, 0, "kwjc")
                    .or_else(|| get_str(rs, 0, "T003"))
                    .unwrap_or_else(|| code.to_string()),
                full_name: get_str(rs, 0, "T003").unwrap_or_default(),
                english_name: get_str(rs, 0, "T006").unwrap_or_default(),
                reg_capital,
                setup_date: String::new(),
                list_date,
                legal_person: get_str(rs, 0, "dsz").unwrap_or_default(),
                secretary: get_str(rs, 0, "T008").unwrap_or_default(),
                address: get_str(rs, 0, "T009").unwrap_or_default(),
                website: get_str(rs, 0, "url").unwrap_or_default(),
                phone: get_str(rs, 0, "T024").unwrap_or_default(),
                fax: String::new(),
                email: get_str(rs, 0, "T026").unwrap_or_default(),
                main_business: get_str(rs, 0, "T017").unwrap_or_default(),
                business_scope: get_str(rs, 0, "T018").unwrap_or_default(),
                reg_no: get_str(rs, 0, "shxydm").unwrap_or_default(),
                org_code: String::new(),
            });
        }
    }

    // 员工效益 N001=年份, N002=人均产值, N003=营收规模, N004=净利润
    let mut employees = vec![];
    if let Some(rs) = employee.result_sets.first() {
        for i in 0..rs.content.len() {
            employees.push(GsgkEmployeeRow {
                stock_code: code.to_string(),
                year_date: get_str(rs, i, "N001").unwrap_or_default(),
                value_per_person: get_f64(rs, i, "N002").unwrap_or(0.0),
                total_revenue: get_f64(rs, i, "N003").unwrap_or(0.0),
                total_profit: get_f64(rs, i, "N004").unwrap_or(0.0),
            });
        }
    }

    // 员工构成 T002=日期, sT003=构成类型名称, T004=分类名称, T005=人数, T006=占比%
    let mut emp_structs = vec![];
    if let Some(rs) = emp_struct.result_sets.first() {
        for i in 0..rs.content.len() {
            emp_structs.push(GsgkEmployeeStructRow {
                stock_code: code.to_string(),
                report_date: get_str(rs, i, "T002")
                    .map(|v| v.trim_end_matches(" 00:00:00").to_string())
                    .unwrap_or_default(),
                category_type: get_str(rs, i, "sT003").unwrap_or_default(),
                item_name: get_str(rs, i, "T004").unwrap_or_default(),
                head_count: get_i64(rs, i, "T005").unwrap_or(0),
                pct: get_f64(rs, i, "T006").unwrap_or(0.0),
            });
        }
    }

    // 研发投入 N001=日期,N002=研发人数,N005=金额,N006=占营收%
    let mut rds = vec![];
    if let Some(rs) = rd.result_sets.first() {
        for i in 0..rs.content.len() {
            rds.push(GsgkRdRow {
                stock_code: code.to_string(),
                year_date: get_str(rs, i, "N001").unwrap_or_default(),
                rd_staff: get_i64(rs, i, "N002").unwrap_or(0),
                rd_amount: get_f64(rs, i, "N005").unwrap_or(0.0),
                rd_pct_revenue: get_f64(rs, i, "N006").unwrap_or(0.0),
            });
        }
    }

    // 参股控股子公司：glgs=公司名, ckgx=关系, cgbl=持股比例, tzje=投资金额, jly=净利润, T012=主营业务
    let mut subsidiaries = vec![];
    if let Some(rs) = subs.result_sets.first() {
        for i in 0..rs.content.len() {
            subsidiaries.push(GsgkSubsidiaryRow {
                stock_code: code.to_string(),
                sub_name: get_str(rs, i, "glgs").unwrap_or_default(),
                reg_capital: {
                    let v = get_f64(rs, i, "tzje").unwrap_or(0.0);
                    if v.abs() > f64::EPSILON {
                        format!("{v:.2}")
                    } else {
                        String::new()
                    }
                },
                hold_pct: get_f64(rs, i, "cgbl").unwrap_or(0.0),
                hold_type: get_str(rs, i, "ckgx").unwrap_or_default(),
                main_business: get_str(rs, i, "T012").unwrap_or_default(),
            });
        }
    }

    debug!(
        "gsgk {code}: basics={} employees={} emp_structs={} rds={} subs={}",
        basics.len(),
        employees.len(),
        emp_structs.len(),
        rds.len(),
        subsidiaries.len()
    );
    Ok((basics, employees, emp_structs, rds, subsidiaries))
}

pub async fn insert_basics(ch: &Client, rows: &[GsgkBasicRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GsgkBasicRow>("f10_gsgk_basic").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_employees(ch: &Client, rows: &[GsgkEmployeeRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GsgkEmployeeRow>("f10_gsgk_employee").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_emp_structs(
    ch: &Client,
    rows: &[GsgkEmployeeStructRow],
) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GsgkEmployeeStructRow>("f10_gsgk_emp_struct")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_rds(ch: &Client, rows: &[GsgkRdRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch.insert::<GsgkRdRow>("f10_gsgk_rd").await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn insert_subsidiaries(ch: &Client, rows: &[GsgkSubsidiaryRow]) -> Result<(), F10Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut ins = ch
        .insert::<GsgkSubsidiaryRow>("f10_gsgk_subsidiary")
        .await?;
    for r in rows {
        ins.write(r).await?;
    }
    ins.end().await?;
    Ok(())
}

pub async fn query_basic(ch: &Client, code: &str) -> Result<Vec<GsgkBasicRow>, F10Error> {
    Ok(ch
        .query("SELECT * EXCEPT(fetched_at) FROM f10_gsgk_basic FINAL WHERE stock_code = ?")
        .bind(code)
        .fetch_all()
        .await?)
}

pub async fn query_employees(ch: &Client, code: &str) -> Result<Vec<GsgkEmployeeRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gsgk_employee FINAL WHERE stock_code = ? ORDER BY year_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_emp_structs(
    ch: &Client,
    code: &str,
) -> Result<Vec<GsgkEmployeeStructRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gsgk_emp_struct FINAL WHERE stock_code = ? ORDER BY report_date DESC, category_type, item_name")
        .bind(code).fetch_all().await?)
}

pub async fn query_rds(ch: &Client, code: &str) -> Result<Vec<GsgkRdRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gsgk_rd FINAL WHERE stock_code = ? ORDER BY year_date DESC")
        .bind(code).fetch_all().await?)
}

pub async fn query_subsidiaries(
    ch: &Client,
    code: &str,
) -> Result<Vec<GsgkSubsidiaryRow>, F10Error> {
    Ok(ch.query("SELECT * EXCEPT(fetched_at) FROM f10_gsgk_subsidiary FINAL WHERE stock_code = ? ORDER BY hold_pct DESC")
        .bind(code).fetch_all().await?)
}
