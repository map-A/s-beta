//! 步骤 2：资金动向 — ClickHouse DDL

pub(super) const CREATE_JYDS_BLOCK_TRADE: &str = "
CREATE TABLE IF NOT EXISTS f10_jyds_block_trade (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    trade_date      String,
    price           Float64 DEFAULT 0,
    amount          Float64 DEFAULT 0,
    volume          Float64 DEFAULT 0,
    premium_pct     Float64 DEFAULT 0,
    buyer           String,
    seller          String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, trade_date, buyer, seller)
";

pub(super) const CREATE_JYDS_MARGIN: &str = "
CREATE TABLE IF NOT EXISTS f10_jyds_margin (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    trade_date      String,
    rzye            Float64 DEFAULT 0,
    rzmre           Float64 DEFAULT 0,
    rzch            Float64 DEFAULT 0,
    rqyl            Float64 DEFAULT 0,
    rqmcl           Float64 DEFAULT 0,
    rqch            Float64 DEFAULT 0,
    rzrqye          Float64 DEFAULT 0,
    close_price     Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, trade_date)
";

pub(super) const CREATE_JYDS_MONEYFLOW: &str = "
CREATE TABLE IF NOT EXISTS f10_jyds_moneyflow (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    trade_date      String,
    zl_net          Float64 DEFAULT 0,
    zl_pct          Float64 DEFAULT 0,
    super_net       Float64 DEFAULT 0,
    super_pct       Float64 DEFAULT 0,
    big_net         Float64 DEFAULT 0,
    big_pct         Float64 DEFAULT 0,
    retail_net      Float64 DEFAULT 0,
    retail_pct      Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, trade_date)
";

pub(super) const CREATE_JYDS_DRAGON_TIGER: &str = "
CREATE TABLE IF NOT EXISTS f10_jyds_dragon_tiger (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    trade_date      String,
    reason          String,
    event_type      String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, trade_date)
";

/// 北上资金成交明细
pub(super) const CREATE_JYDS_NORTHBOUND: &str = "
CREATE TABLE IF NOT EXISTS f10_jyds_northbound (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    trade_date      String,
    direction       String,
    price           Float64 DEFAULT 0,
    volume          Float64 DEFAULT 0,
    amount          Float64 DEFAULT 0,
    hold_pct        Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, trade_date)
";
