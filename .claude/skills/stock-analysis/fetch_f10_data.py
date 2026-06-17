#!/usr/bin/env python3
"""
F10全量数据查询工具
从ClickHouse数据库读取 qbrs-f10info 抓取的全部14个页面、66张表的数据。
输出结构化JSON，按页面分组，供 Serenity skill 分析使用。
"""

import os
import sys
import json
import subprocess
from pathlib import Path
import clickhouse_connect

# 自动推算项目根目录（fetch_f10_data.py 在 .claude/skills/stock-analysis/ 下）
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent

CK_CONFIG = {
    'host': os.getenv('CK_HOST', 'localhost'),
    'port': int(os.getenv('CK_PORT', '8123')),
    'database': os.getenv('CK_DATABASE', 'f10'),
    'username': os.getenv('CK_USER', 'default'),
    'password': os.getenv('CK_PASSWORD', ''),
}

QBRS_F10INFO_PATH = os.getenv(
    'QBRS_F10INFO_PATH',
    str(PROJECT_ROOT / 'f10info' / 'target' / 'release' / 'qbrs-f10info'),
)

# 全部66张表定义：(表名, 显示名, ORDER BY字段, LIMIT)
# 按14个F10页面分组
PAGES = [
    ("1_最新提示", [
        ("f10_zxts_overview",       "公司概要快照",    "fetched_at",    1),
        ("f10_zxts_kpi",            "主要指标历史",    "report_date",  20),
        ("f10_zxts_concept",        "概念题材",        "fetched_at",  10),
        ("f10_zxts_events",         "公司大事",        "event_date",  10),
        ("f10_zxts_news",           "公司资讯新闻",    "pub_date",    10),
        ("f10_zxts_jgdy",           "机构调研",        "visit_date",  10),
        ("f10_zxts_hdwd",           "互动问答",        "answer_date", 10),
        ("f10_zxts_peer",           "可比公司",        "fetched_at", 10),
    ]),
    ("2_资金动向", [
        ("f10_jyds_block_trade",    "大宗交易",        "trade_date",  5),
        ("f10_jyds_margin",         "融资融券",        "trade_date",  20),
        ("f10_jyds_moneyflow",      "资金流向",        "trade_date",  10),
        ("f10_jyds_dragon_tiger",   "龙虎榜",          "trade_date",  30),
        # ("f10_jyds_northbound",     "北向资金明细",    "trade_date",  60),
    ]),
    ("3_基本情况", [
        ("f10_gsgk_basic",          "公司基本信息",    "fetched_at",    1),
        ("f10_gsgk_employee",       "员工数据",        "year_date",    10),
        ("f10_gsgk_emp_struct",     "员工结构",        "report_date",  10),
        ("f10_gsgk_rd",             "研发投入",        "year_date",    10),
        ("f10_gsgk_subsidiary",     "子公司",          "fetched_at",  10),
    ]),
    ("4_股本结构", [
        ("f10_gbjg_share_struct",   "股本变动",        "change_date",  10),
        ("f10_gbjg_change",         "股本变更事件",    "change_date",  10),
        ("f10_gbjg_unlock",         "限售解禁",        "unlock_date",  10),
        ("f10_gbjg_buyback",        "回购",            "announce_date", 10),
    ]),
    ("5_股东研究", [
        ("f10_gdyj_controlling_holder", "实控人/控股股东", "report_date",  10),
        ("f10_gdyj_holder_count",   "股东人数变化",    "report_date",  10),
        ("f10_gdyj_industry_holders","同行股东数",      "fetched_at",  10),
        ("f10_gdyj_top10_float",    "前10大流通股东",   "report_date",  10),
        ("f10_gdyj_top10_all",      "前10大股东",      "report_date",  10),
        ("f10_gdyj_hold_change",    "股东增减持",      "end_date",     10),
        ("f10_gdyj_inst_trend",     "机构持仓趋势",    "report_date",  20),
        ("f10_gdyj_inst_summary",   "机构持仓汇总",    "report_date",  50),
        ("f10_gdyj_inst_detail",    "机构持仓明细",    "report_date", 10),
    ]),
    # ("6_分红融资", [
    #     ("f10_fhrz_dividend",       "分红",            "report_date",  20),
    #     ("f10_fhrz_rights",         "配股",            "report_date",  10),
    #     ("f10_fhrz_addissue",       "增发",            "subscribe_date", 10),
    # ]),
    ("7_财务分析", [
        ("f10_cwfx_indicator",      "财务指标",        "report_date",  20),
        ("f10_cwfx_report",         "财务报告列表",    "report_date",  20),
        ("f10_cwfx_profit",         "盈利能力指标",    "report_date",  20),
        ("f10_cwfx_research",       "研究报告",        "report_date",  20),
    ]),
    ("8_经营分析", [
        ("f10_jyfx_main_biz",       "主营业务构成",    "report_date", 10),
        ("f10_jyfx_top5_customer",  "前5大客户",       "report_date",  30),
        ("f10_jyfx_top5_supplier",  "前5大供应商",     "report_date",  30),
        ("f10_jyfx_oper_data",      "经营数据",        "report_date", 10),
    ]),
    ("9_资本运作", [
        ("f10_zbyz_fundraise",      "募集资金使用",    "report_date",  5),
        ("f10_zbyz_violation",      "违规",            "event_date",   10),
        ("f10_zbyz_major_event",    "重大事项",        "event_date",   10),
        ("f10_zbyz_share_transfer", "股权转让",        "complete_date", 10),
        ("f10_zbyz_share_control",  "股权控制变动",    "change_date",  10),
    ]),
    ("10_研报评级", [
        ("f10_ybpj_rating_stat",    "评级统计",        "stat_date",     5),
        ("f10_ybpj_forecast",       "机构预测",        "report_date",  30),
        ("f10_ybpj_report",         "研报列表",        "report_date",  30),
    ]),
    ("11_热点题材", [
        ("f10_rdtc_theme",          "热点题材",        "theme_date",   50),
        ("f10_rdtc_event",          "题材事件",        "event_date",   5),
        ("f10_rdtc_logic",          "投资逻辑",        "fetched_at",  10),
        # ("f10_rdtc_concept",        "概念板块",        "fetched_at",  100),
    ]),
    ("12_公司资讯", [
        ("f10_gszx_news",           "公司新闻",        "pub_date",     5),
        ("f10_gszx_report",         "公司研报",        "pub_date",     10),
    ]),
    # ("13_主力持仓", [
    #     ("f10_zlcc_inst_timeline",  "机构持仓时间线",  "report_date",  20),
    #     ("f10_zlcc_inst_by_type",   "机构分类持仓",    "report_date",  50),
    #     ("f10_zlcc_inst_detail",    "机构持仓明细",    "report_date", 100),
    # ]),
    ("14_行业分析", [
        ("f10_hyfx_industry_news",  "行业新闻",        "pub_date",     10),
        ("f10_hyfx_industry_report","行业研报",        "pub_date",     10),
        ("f10_hyfx_market_rank",    "行情排名",        "fetched_at",  10),
        # ("f10_hyfx_size_rank",      "规模排名",        "fetched_at",  200),
        # ("f10_hyfx_valuation_rank", "估值排名",        "fetched_at",  200),
        # ("f10_hyfx_financial_rank", "财务排名",        "fetched_at",  200),
        # ("f10_hyfx_dividend_rank",  "分红排名",        "fetched_at",  200),
    ]),
]


def fetch_stock_data(stock_code: str) -> dict:
    """调用 qbrs-f10info 抓取股票数据到数据库"""
    print(f"正在抓取股票 {stock_code} 的F10数据...")

    try:
        env = os.environ.copy()
        env.update({
            'CK_URL': f"http://{CK_CONFIG['host']}:{CK_CONFIG['port']}",
            'CK_DATABASE': CK_CONFIG['database'],
            'CK_USER': CK_CONFIG['username'],
            'CK_PASSWORD': CK_CONFIG['password'],
        })

        result = subprocess.run(
            [QBRS_F10INFO_PATH, '--stock', stock_code],
            env=env,
            capture_output=True,
            text=True,
            timeout=300,
        )

        if result.returncode == 0:
            print(f"  ✓ 抓取成功")
            return {'success': True}
        else:
            print(f"  ✗ 抓取失败: {result.stderr}")
            return {'success': False, 'error': result.stderr}

    except subprocess.TimeoutExpired:
        return {'success': False, 'error': '抓取超时（>5分钟）'}
    except FileNotFoundError:
        return {'success': False, 'error': f'未找到二进制: {QBRS_F10INFO_PATH}'}
    except Exception as e:
        return {'success': False, 'error': str(e)}


def get_client():
    """获取ClickHouse客户端"""
    try:
        return clickhouse_connect.get_client(**CK_CONFIG)
    except Exception as e:
        print(f"数据库连接失败: {e}")
        print(f"配置: host={CK_CONFIG['host']}:{CK_CONFIG['port']} db={CK_CONFIG['database']}")
        sys.exit(1)


def rows_to_dicts(result) -> list:
    """将 clickhouse_connect 查询结果转为 dict 列表"""
    cols = result.column_names
    return [dict(zip(cols, row)) for row in result.result_rows]


def query_table(client, stock_code: str, table: str, order_by: str, limit: int) -> list:
    """查询单张表的全部数据"""
    query = f"""
    SELECT *
    FROM {table}
    WHERE stock_code = %(code)s
    ORDER BY {order_by} DESC
    LIMIT {limit}
    """
    result = client.query(query, parameters={'code': stock_code})
    return rows_to_dicts(result)


def query_all(stock_code: str) -> dict:
    """查询全部66张表，按页面分组返回"""
    print(f"\n正在从数据库读取股票 {stock_code} 的全部F10数据...")

    client = get_client()
    data = {'stock_code': stock_code}
    total_tables = 0
    total_rows = 0

    for page_name, tables in PAGES:
        page_data = {}
        for table, label, order_by, limit in tables:
            try:
                rows = query_table(client, stock_code, table, order_by, limit)
                page_data[label] = rows
                total_tables += 1
                total_rows += len(rows)
            except Exception as e:
                # 表可能为空或不存在，跳过
                page_data[label] = []
                print(f"  ⚠ {table}: {e}")
        data[page_name] = page_data

    print(f"  ✓ 读取完成: {total_tables} 张表, {total_rows} 行数据")
    return data


def print_summary(data: dict):
    """打印数据概况"""
    print(f"\n{'='*50}")
    print(f"股票代码: {data['stock_code']}")
    print(f"{'='*50}")

    for page_name, tables in PAGES:
        page = data.get(page_name, {})
        parts = []
        for _, label, _, _ in tables:
            rows = page.get(label, [])
            if rows:
                parts.append(f"{label}:{len(rows)}")
        status = '✓' if any(len(page.get(l, [])) > 0 for _, l, _, _ in tables) else '✗'
        print(f"  {status} {page_name}")
        for p in parts:
            print(f"      {p}")


def main():
    if len(sys.argv) < 2:
        print("用法: python fetch_f10_data.py <股票代码> [--fetch-only|--query-only]")
        print()
        print("示例:")
        print("  python fetch_f10_data.py 688720           # 抓取+查询")
        print("  python fetch_f10_data.py 688720 --query-only  # 仅查询已有数据")
        print("  python fetch_f10_data.py 688720 --fetch-only  # 仅抓取到数据库")
        print()
        print("输出: /tmp/f10_data_<股票代码>.json")
        sys.exit(1)

    stock_code = sys.argv[1]
    mode = sys.argv[2] if len(sys.argv) > 2 else 'both'

    # 抓取数据
    if mode in ('both', '--fetch-only'):
        result = fetch_stock_data(stock_code)
        if not result['success']:
            print(f"抓取失败: {result['error']}")
            if mode == '--fetch-only':
                sys.exit(1)

    # 查询数据
    if mode in ('both', '--query-only'):
        data = query_all(stock_code)

        output_file = f'/tmp/f10_data_{stock_code}.json'
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2, default=str)

        print_summary(data)
        print(f"\n数据已保存到: {output_file}")


if __name__ == '__main__':
    main()
