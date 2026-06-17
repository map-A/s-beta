//! 步骤 4：股本结构 — ClickHouse DDL

pub(super) const CREATE_GBJG_SHARE_STRUCT: &str = "
CREATE TABLE IF NOT EXISTS f10_gbjg_share_struct (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    change_date     String,
    total_shares    Float64 DEFAULT 0,
    a_shares        Float64 DEFAULT 0,
    b_shares        Float64 DEFAULT 0,
    float_a         Float64 DEFAULT 0,
    restricted_a    Float64 DEFAULT 0,
    tradeable_a     Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, change_date)
";

pub(super) const CREATE_GBJG_CHANGE: &str = "
CREATE TABLE IF NOT EXISTS f10_gbjg_change (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    change_date     String,
    total_after     Float64 DEFAULT 0,
    change_pct      Float64 DEFAULT 0,
    event_code      String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, change_date)
";

pub(super) const CREATE_GBJG_UNLOCK: &str = "
CREATE TABLE IF NOT EXISTS f10_gbjg_unlock (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    unlock_date     String,
    lock_date       String,
    lock_type       String,
    unlock_shares   Float64 DEFAULT 0,
    unlock_reason   String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, unlock_date, lock_date)
";

pub(super) const CREATE_GBJG_BUYBACK: &str = "
CREATE TABLE IF NOT EXISTS f10_gbjg_buyback (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    announce_date   String,
    plan_end_date   String,
    plan_shares     Float64 DEFAULT 0,
    plan_amount     Float64 DEFAULT 0,
    done_shares     Float64 DEFAULT 0,
    done_pct        Float64 DEFAULT 0,
    price_min       Float64 DEFAULT 0,
    price_avg       Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, announce_date)
";
