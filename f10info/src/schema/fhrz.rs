//! 步骤 6：分红融资 — ClickHouse DDL

pub(super) const CREATE_FHRZ_DIVIDEND: &str = "
CREATE TABLE IF NOT EXISTS f10_fhrz_dividend (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    announce_date   String,
    dividend_plan   String,
    eps             Float64 DEFAULT 0,
    half_eps        Float64 DEFAULT 0,
    record_date     String,
    ex_date         String,
    status          String,
    payout_ratio    Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_FHRZ_RIGHTS: &str = "
CREATE TABLE IF NOT EXISTS f10_fhrz_rights (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    price           Float64 DEFAULT 0,
    amount_raised   Float64 DEFAULT 0,
    rights_ratio    String,
    shares          Float64 DEFAULT 0,
    ex_date         String,
    pay_date        String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

pub(super) const CREATE_FHRZ_ADDISSUE: &str = "
CREATE TABLE IF NOT EXISTS f10_fhrz_addissue (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    issue_type      String,
    price           Float64 DEFAULT 0,
    amount_raised   Float64 DEFAULT 0,
    issue_shares    Float64 DEFAULT 0,
    issue_object    String,
    subscribe_date  String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, issue_type, subscribe_date)
";
