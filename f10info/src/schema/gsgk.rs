//! 步骤 3：基本情况 — ClickHouse DDL

pub(super) const CREATE_GSGK_BASIC: &str = "
CREATE TABLE IF NOT EXISTS f10_gsgk_basic (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    company_name    String,
    full_name       String,
    english_name    String,
    reg_capital     String,
    setup_date      String,
    list_date       String,
    legal_person    String,
    secretary       String,
    address         String,
    website         String,
    phone           String,
    fax             String,
    email           String,
    main_business   String,
    business_scope  String,
    reg_no          String,
    org_code        String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code)
";

pub(super) const CREATE_GSGK_EMPLOYEE: &str = "
CREATE TABLE IF NOT EXISTS f10_gsgk_employee (
    stock_code        LowCardinality(String),
    fetched_at        DateTime DEFAULT now(),
    year_date         String,
    value_per_person  Float64 DEFAULT 0,
    total_revenue     Float64 DEFAULT 0,
    total_profit      Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, year_date)
";

pub(super) const CREATE_GSGK_EMP_STRUCT: &str = "
CREATE TABLE IF NOT EXISTS f10_gsgk_emp_struct (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    report_date     String,
    category_type   String,
    item_name       String,
    head_count      Int64 DEFAULT 0,
    pct             Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, report_date, category_type, item_name)
";

pub(super) const CREATE_GSGK_RD: &str = "
CREATE TABLE IF NOT EXISTS f10_gsgk_rd (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    year_date       String,
    rd_staff        Int64 DEFAULT 0,
    rd_amount       Float64 DEFAULT 0,
    rd_pct_revenue  Float64 DEFAULT 0
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, year_date)
";

pub(super) const CREATE_GSGK_SUBSIDIARY: &str = "
CREATE TABLE IF NOT EXISTS f10_gsgk_subsidiary (
    stock_code      LowCardinality(String),
    fetched_at      DateTime DEFAULT now(),
    sub_name        String,
    reg_capital     String,
    hold_pct        Float64 DEFAULT 0,
    hold_type       String,
    main_business   String
) ENGINE = ReplacingMergeTree(fetched_at)
ORDER BY (stock_code, sub_name)
";
