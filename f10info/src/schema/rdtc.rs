//! 步骤 11：热点题材 — ClickHouse DDL

pub(super) const CREATE_RDTC_THEME: &str = "
CREATE TABLE IF NOT EXISTS f10_rdtc_theme (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    theme_type      String,
    theme_date      String,
    theme_name      String,
    theme_content   String,
    heat            Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, theme_type, theme_date, theme_name)
";

pub(super) const CREATE_RDTC_EVENT: &str = "
CREATE TABLE IF NOT EXISTS f10_rdtc_event (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    event_date      String,
    event_name      String,
    event_type      String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, event_date, event_name)
";

pub(super) const CREATE_RDTC_LOGIC: &str = "
CREATE TABLE IF NOT EXISTS f10_rdtc_logic (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    category        String,
    content         String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, category)
";

pub(super) const CREATE_RDTC_CONCEPT: &str = "
CREATE TABLE IF NOT EXISTS f10_rdtc_concept (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    concept_name    String,
    concept_code    String,
    heat_score      Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, concept_name)
";
