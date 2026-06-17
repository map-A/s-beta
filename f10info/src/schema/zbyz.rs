//! 步骤 9：资本运作 — ClickHouse DDL

pub(super) const CREATE_ZBYZ_FUNDRAISE: &str = "
CREATE TABLE IF NOT EXISTS f10_zbyz_fundraise (
    stock_code        LowCardinality(String),
    fetched_at        DateTime DEFAULT now(),
    report_date       String,
    project_name      String,
    project_type      String,
    committed_amount  Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, project_name)
";

pub(super) const CREATE_ZBYZ_VIOLATION: &str = "
CREATE TABLE IF NOT EXISTS f10_zbyz_violation (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    event_date      String,
    event_type      String,
    party           String,
    violation_type  String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, event_date, event_type)
";

pub(super) const CREATE_ZBYZ_MAJOR_EVENT: &str = "
CREATE TABLE IF NOT EXISTS f10_zbyz_major_event (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    event_date      String,
    event_type      String,
    amount          Float64 DEFAULT 0,
    event_content   String,
    related_party   String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, event_date, event_type)
";

pub(super) const CREATE_ZBYZ_SHARE_TRANSFER: &str = "
CREATE TABLE IF NOT EXISTS f10_zbyz_share_transfer (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    complete_date   String,
    status          String,
    shares_count    Float64 DEFAULT 0,
    from_party      String,
    to_party        String,
    hold_pct        Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, complete_date, from_party)
";

pub(super) const CREATE_ZBYZ_SHARE_CONTROL: &str = "
CREATE TABLE IF NOT EXISTS f10_zbyz_share_control (
    stock_code    LowCardinality(String),
    fetched_at    DateTime DEFAULT now(),
    change_date   String,
    asset_name    String,
    asset_type    String,
    to_party      String,
    from_party    String,
    amount        Float64 DEFAULT 0,
    currency      String,
    status        String,
    detail        String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, change_date, asset_name)
";
