//! 步骤 10：研报评级 — ClickHouse DDL

pub(super) const CREATE_YBPJ_RATING_STAT: &str = "
CREATE TABLE IF NOT EXISTS f10_ybpj_rating_stat (
    stock_code        LowCardinality(String),
    fetched_at        DateTime DEFAULT now(),
    stat_date         String,
    days_count        Int32 DEFAULT 0,
    total_inst        Int32 DEFAULT 0,
    buy_count         Int32 DEFAULT 0,
    overweight_count  Int32 DEFAULT 0,
    neutral_count     Int32 DEFAULT 0,
    underweight_count Int32 DEFAULT 0,
    sell_count        Int32 DEFAULT 0,
    avg_score         Float64 DEFAULT 0,
    avg_target_price  Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, stat_date)
";

pub(super) const CREATE_YBPJ_FORECAST: &str = "
CREATE TABLE IF NOT EXISTS f10_ybpj_forecast (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    org_name        String,
    rating          String,
    change_type     String,
    target_price    String,
    eps_year1       Float64 DEFAULT 0,
    eps_year2       Float64 DEFAULT 0,
    eps_year3       Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, org_name)
";

pub(super) const CREATE_YBPJ_REPORT: &str = "
CREATE TABLE IF NOT EXISTS f10_ybpj_report (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    title           String,
    rating          String,
    org_name        String,
    summary         String,
    report_id       String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, report_id)
";
