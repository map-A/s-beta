//! 步骤 13：主力持仓 — ClickHouse DDL

pub(super) const CREATE_ZLCC_INST_TIMELINE: &str = "
CREATE TABLE IF NOT EXISTS f10_zlcc_inst_timeline (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    inst_count      Int32 DEFAULT 0,
    inst_change     Float64 DEFAULT 0,
    hold_shares     Float64 DEFAULT 0,
    hold_market_cap Float64 DEFAULT 0,
    float_pct       Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_ZLCC_INST_BY_TYPE: &str = "
CREATE TABLE IF NOT EXISTS f10_zlcc_inst_by_type (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    inst_type_code  String,
    inst_type_name  String,
    inst_count      Int32 DEFAULT 0,
    hold_shares     Float64 DEFAULT 0,
    float_pct       Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, inst_type_code)
";

pub(super) const CREATE_ZLCC_INST_DETAIL: &str = "
CREATE TABLE IF NOT EXISTS f10_zlcc_inst_detail (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    inst_name       String,
    inst_type       String,
    hold_shares     Float64 DEFAULT 0,
    change_amount   Float64 DEFAULT 0,
    hold_market_cap Float64 DEFAULT 0,
    float_pct       Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, inst_name)
";
