//! 步骤 14：行业分析 — ClickHouse DDL

pub(super) const CREATE_HYFX_INDUSTRY_NEWS: &str = "
CREATE TABLE IF NOT EXISTS f10_hyfx_industry_news (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    pub_date        String,
    title           String,
    rec_id          String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, pub_date, rec_id)
";

pub(super) const CREATE_HYFX_INDUSTRY_REPORT: &str = "
CREATE TABLE IF NOT EXISTS f10_hyfx_industry_report (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    pub_date        String,
    title           String,
    rec_id          String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, pub_date, rec_id)
";

pub(super) const CREATE_HYFX_MARKET_RANK: &str = "
CREATE TABLE IF NOT EXISTS f10_hyfx_market_rank (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    peer_code       String,
    peer_name       String,
    exchange        String,
    chg_day         Float64 DEFAULT 0,
    chg_week        Float64 DEFAULT 0,
    chg_month       Float64 DEFAULT 0,
    chg_quarter     Float64 DEFAULT 0,
    chg_half_year   Float64 DEFAULT 0,
    chg_year        Float64 DEFAULT 0,
    self_rank       Int32 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, peer_code)
";

pub(super) const CREATE_HYFX_SIZE_RANK: &str = "
CREATE TABLE IF NOT EXISTS f10_hyfx_size_rank (
    stock_code       LowCardinality(String),
    fetched_at       DateTime DEFAULT now(),
    peer_code        String,
    peer_name        String,
    total_market_cap Float64 DEFAULT 0,
    float_market_cap Float64 DEFAULT 0,
    total_shares     Float64 DEFAULT 0,
    revenue          Float64 DEFAULT 0,
    price            Float64 DEFAULT 0,
    self_rank        Int32 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, peer_code)
";

pub(super) const CREATE_HYFX_VALUATION_RANK: &str = "
CREATE TABLE IF NOT EXISTS f10_hyfx_valuation_rank (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    peer_code       String,
    peer_name       String,
    price           Float64 DEFAULT 0,
    pe_ttm          Float64 DEFAULT 0,
    pe_lyr          Float64 DEFAULT 0,
    pb              Float64 DEFAULT 0,
    ps              Float64 DEFAULT 0,
    pcf             Float64 DEFAULT 0,
    self_rank       Int32 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, peer_code)
";

pub(super) const CREATE_HYFX_FINANCIAL_RANK: &str = "
CREATE TABLE IF NOT EXISTS f10_hyfx_financial_rank (
    stock_code        LowCardinality(String),
    fetched_at        DateTime DEFAULT now(),
    peer_code         String,
    peer_name         String,
    eps               Float64 DEFAULT 0,
    bvps              Float64 DEFAULT 0,
    revenue_growth    Float64 DEFAULT 0,
    roe               Float64 DEFAULT 0,
    net_profit_growth Float64 DEFAULT 0,
    self_rank         Int32 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, peer_code)
";

pub(super) const CREATE_HYFX_DIVIDEND_RANK: &str = "
CREATE TABLE IF NOT EXISTS f10_hyfx_dividend_rank (
    stock_code              LowCardinality(String),
    fetched_at              DateTime DEFAULT now(),
    peer_code               String,
    peer_name               String,
    rank                    Int32 DEFAULT 0,
    dividend_per_share      Float64 DEFAULT 0,
    ipo_funding             Float64 DEFAULT 0,
    total_dividend          Float64 DEFAULT 0,
    ipo_amount              Float64 DEFAULT 0,
    addissue_count          Int32 DEFAULT 0,
    addissue_amount         Float64 DEFAULT 0,
    rights_count            Int32 DEFAULT 0,
    rights_amount           Float64 DEFAULT 0,
    cb_amount               Float64 DEFAULT 0,
    dividend_funding_ratio  Float64 DEFAULT 0,
    self_rank               Int32 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, peer_code)
";
