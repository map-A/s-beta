//! 步骤 12：公司资讯 — ClickHouse DDL

pub(super) const CREATE_GSZX_NEWS: &str = "
CREATE TABLE IF NOT EXISTS f10_gszx_news (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    news_type       String,
    pub_date        String,
    title           String,
    rec_id          String,
    is_important    Int8 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, news_type, pub_date, rec_id)
";

pub(super) const CREATE_GSZX_REPORT: &str = "
CREATE TABLE IF NOT EXISTS f10_gszx_report (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    pub_date        String,
    title           String,
    rating          String,
    analyst         String,
    org_name        String,
    report_id       String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, pub_date, report_id)
";
