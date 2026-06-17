//! 步骤 8：经营分析 — ClickHouse DDL

pub(super) const CREATE_JYFX_MAIN_BIZ: &str = "
CREATE TABLE IF NOT EXISTS f10_jyfx_main_biz (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    category_type   String,
    category_code   String,
    category_name   String,
    revenue         Float64 DEFAULT 0,
    revenue_pct     Float64 DEFAULT 0,
    profit          Float64 DEFAULT 0,
    gross_margin    Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, category_type, category_code)
";

pub(super) const CREATE_JYFX_TOP5_CUSTOMER: &str = "
CREATE TABLE IF NOT EXISTS f10_jyfx_top5_customer (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    customer_name   String,
    amount          Float64 DEFAULT 0,
    revenue_pct     Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, customer_name)
";

pub(super) const CREATE_JYFX_TOP5_SUPPLIER: &str = "
CREATE TABLE IF NOT EXISTS f10_jyfx_top5_supplier (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    supplier_name   String,
    amount          Float64 DEFAULT 0,
    purchase_pct    Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, supplier_name)
";

pub(super) const CREATE_JYFX_OPER_DATA: &str = "
CREATE TABLE IF NOT EXISTS f10_jyfx_oper_data (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    metric_name     String,
    metric_value    Float64 DEFAULT 0,
    metric_code     String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, metric_code)
";
