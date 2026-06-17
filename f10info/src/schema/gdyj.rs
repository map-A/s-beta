//! 步骤 5：股东研究 — ClickHouse DDL

pub(super) const CREATE_GDYJ_CONTROLLING_HOLDER: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_controlling_holder (
    stock_code          LowCardinality(String),
    fetched_at          DateTime DEFAULT now(),
    report_date         String,
    controlling_holder  String,
    actual_controller   String,
    ultimate_controller String,
    hold_pct            Float64 DEFAULT 0,
    direct_hold_pct     Float64 DEFAULT 0,
    equity_chain        String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_GDYJ_HOLDER_COUNT: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_holder_count (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    holder_count    Int64 DEFAULT 0,
    avg_hold_shares Float64 DEFAULT 0,
    change_pct      Float64 DEFAULT 0,
    net_change      Int64 DEFAULT 0,
    price           Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_GDYJ_INDUSTRY_HOLDERS: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_industry_holders (
    stock_code   LowCardinality(String),
    fetched_at   DateTime DEFAULT now(),
    peer_code    String,
    peer_name    String,
    holder_count Int64 DEFAULT 0,
    change_pct   Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, peer_code)
";

pub(super) const CREATE_GDYJ_TOP10_FLOAT: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_top10_float (
    stock_code        LowCardinality(String),
    fetched_at        DateTime DEFAULT now(),
    report_date       String,
    is_report_period  Int8 DEFAULT 0,
    holder_name       String,
    holder_id         String,
    hold_shares       Int64 DEFAULT 0,
    holder_type       String,
    share_nature      String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, holder_id)
";

pub(super) const CREATE_GDYJ_TOP10_ALL: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_top10_all (
    stock_code        LowCardinality(String),
    fetched_at        DateTime DEFAULT now(),
    report_date       String,
    is_report_period  Int8 DEFAULT 0,
    holder_name       String,
    holder_id         String,
    hold_shares       Int64 DEFAULT 0,
    hold_pct          Float64 DEFAULT 0,
    holder_type       String,
    share_nature      String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, holder_id)
";

pub(super) const CREATE_GDYJ_HOLD_CHANGE: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_hold_change (
    stock_code    LowCardinality(String),
    fetched_at    DateTime DEFAULT now(),
    start_date    String,
    end_date      String,
    holder_name   String,
    avg_price     Float64 DEFAULT 0,
    change_amount Float64 DEFAULT 0,
    total_hold    Float64 DEFAULT 0,
    event_type    String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, start_date, holder_name)
";

pub(super) const CREATE_GDYJ_INST_TREND: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_inst_trend (
    stock_code     LowCardinality(String),
    fetched_at     DateTime DEFAULT now(),
    report_date    String,
    inst_count     Int64 DEFAULT 0,
    count_change   Int64 DEFAULT 0,
    hold_shares    Float64 DEFAULT 0,
    shares_change  Float64 DEFAULT 0,
    market_cap     Float64 DEFAULT 0,
    float_pct      Float64 DEFAULT 0,
    price          Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_GDYJ_INST_SUMMARY: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_inst_summary (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    inst_type_name  String,
    inst_type_code  Int64 DEFAULT 0,
    inst_count      Int64 DEFAULT 0,
    hold_shares     Float64 DEFAULT 0,
    float_pct       Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, inst_type_code)
";

pub(super) const CREATE_GDYJ_INST_DETAIL: &str = "
CREATE TABLE IF NOT EXISTS f10_gdyj_inst_detail (
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
