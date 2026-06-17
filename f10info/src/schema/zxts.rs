//! 步骤 1：最新提示 — ClickHouse DDL

/// 公司概要快照
pub(super) const CREATE_ZXTS_OVERVIEW: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_overview (
    stock_code           LowCardinality(String),
    fetched_at           DateTime DEFAULT now(),
    report_date          String,
    eps                  Float64 DEFAULT 0,
    bvps                 Float64 DEFAULT 0,
    revenue              Float64 DEFAULT 0,
    net_profit           Float64 DEFAULT 0,
    gross_margin         Float64 DEFAULT 0,
    debt_ratio           Float64 DEFAULT 0,
    undist_profit_ps     Float64 DEFAULT 0,
    capres_ps            Float64 DEFAULT 0,
    op_cf_ps             Float64 DEFAULT 0,
    revenue_yoy          Float64 DEFAULT 0,
    profit_yoy           Float64 DEFAULT 0,
    deducted_profit_yoy  Float64 DEFAULT 0,
    pe_ttm               Float64 DEFAULT 0,
    pe_lyr               Float64 DEFAULT 0,
    pb                   Float64 DEFAULT 0,
    total_market_cap     Float64 DEFAULT 0,
    total_shares         Float64 DEFAULT 0,
    float_a_shares       Float64 DEFAULT 0,
    company_status       String,
    weighted_roe         Float64 DEFAULT 0,
    industry_1           String,
    industry_2           String,
    main_business        String,
    pledge_date          String,
    pledge_pct           Float64 DEFAULT 0,
    total_pledge_shares  Float64 DEFAULT 0,
    csrc_industry        String,
    industry_pe          Float64 DEFAULT 0,
    industry_pe_date     String,
    market_rank          Int64 DEFAULT 0,
    industry_rank        Int64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code)
";

/// 主要指标历史序列
pub(super) const CREATE_ZXTS_KPI: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_kpi (
    stock_code           LowCardinality(String),
    fetched_at           DateTime DEFAULT now(),
    report_date          String,
    eps                  Float64 DEFAULT 0,
    deducted_eps         Float64 DEFAULT 0,
    roe                  Float64 DEFAULT 0,
    op_cf_ps             Float64 DEFAULT 0,
    undist_profit_ps     Float64 DEFAULT 0,
    capres_ps            Float64 DEFAULT 0,
    bvps                 Float64 DEFAULT 0,
    net_profit           Float64 DEFAULT 0,
    revenue              Float64 DEFAULT 0,
    deducted_net_profit  Float64 DEFAULT 0,
    parent_net_profit    Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date)
";

/// 概念题材
pub(super) const CREATE_ZXTS_CONCEPT: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_concept (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    concept_id      String,
    concept_name    String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, concept_id)
";

/// 公司大事
pub(super) const CREATE_ZXTS_EVENTS: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_events (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    event_date      String,
    event_type      String,
    event_content   String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, event_date, event_type)
";

/// 公司资讯（新闻/公告/研报/路演）
pub(super) const CREATE_ZXTS_NEWS: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_news (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    news_type       String,
    pub_date        String,
    title           String,
    rec_id          String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, news_type, pub_date, rec_id)
";

/// 机构调研
pub(super) const CREATE_ZXTS_JGDY: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_jgdy (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    visit_date      String,
    org_name        String,
    org_type        String,
    visit_type      String,
    contact_person  String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, visit_date, org_name)
";

/// 互动问答
pub(super) const CREATE_ZXTS_HDWD: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_hdwd (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    question_date   String,
    answer_date     String,
    question        String,
    answer          String,
    q_type          String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, answer_date, question)
";

/// 可比公司
pub(super) const CREATE_ZXTS_PEER: &str = "
CREATE TABLE IF NOT EXISTS f10_zxts_peer (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    peer_code       String,
    peer_name       String,
    market_type     String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, peer_code)
";
