//! 步骤 7：财务分析 — ClickHouse DDL

pub(super) const CREATE_CWFX_INDICATOR: &str = "
CREATE TABLE IF NOT EXISTS f10_cwfx_indicator (
    stock_code              LowCardinality(String),
    fetched_at              DateTime DEFAULT now(),
    report_date             String,
    eps                     Float64 DEFAULT 0,
    non_recurring_profit    Float64 DEFAULT 0,
    per_share_cashflow      Float64 DEFAULT 0,
    total_profit            Float64 DEFAULT 0,
    net_profit              Float64 DEFAULT 0,
    roe                     Float64 DEFAULT 0,
    gross_margin            Float64 DEFAULT 0,
    net_profit_yoy          Float64 DEFAULT 0,
    revenue_yoy             Float64 DEFAULT 0,
    revenue_qoq             Float64 DEFAULT 0,
    net_profit_qoq          Float64 DEFAULT 0,
    weighted_roe            Float64 DEFAULT 0,
    non_recurring_profit_qoq Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_CWFX_REPORT: &str = "
CREATE TABLE IF NOT EXISTS f10_cwfx_report (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_year     String,
    report_period   String,
    report_date     String,
    report_url      String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_year, report_period)
";

pub(super) const CREATE_CWFX_PROFIT: &str = "
CREATE TABLE IF NOT EXISTS f10_cwfx_profit (
    stock_code          LowCardinality(String),
    fetched_at          DateTime DEFAULT now(),
    report_date         String,
    roe                 Float64 DEFAULT 0,
    gross_margin        Float64 DEFAULT 0,
    op_profit_margin    Float64 DEFAULT 0,
    net_profit_margin   Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_CWFX_RESEARCH: &str = "
CREATE TABLE IF NOT EXISTS f10_cwfx_research (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    title           String,
    institution     String,
    report_date     String,
    rec_id          Int64
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, rec_id)
";
