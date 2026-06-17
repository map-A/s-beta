# CLAUDE.md — 项目说明

## 项目概述

A股供应链「卡点」分析系统。使用 Serenity 方法论分析 A 股上市公司，识别供应链瓶颈投资机会。

## Skill 使用

本项目包含项目级 skill `stock-analysis`，位于 `.claude/skills/stock-analysis/`。

使用方式：`/stock-analysis <股票代码>` 或描述公司名称即可触发完整分析流程。

## 关键路径

- **Skill 定义**: `.claude/skills/stock-analysis/SKILL.md`
- **数据获取脚本**: `.claude/skills/stock-analysis/fetch_f10_data.py`
- **参考知识库**: `.claude/skills/stock-analysis/references/`
- **F10 抓取工具源码**: `f10info/`（Rust 项目）
- **分析模板**: `templates/`
- **分析输出**: `outputs/{公司名}_{代码}/`
- **前端展示**: `web/`

## 数据流

```
通达信 TQLEX API → qbrs-f10info (Rust) → ClickHouse → fetch_f10_data.py → /tmp/f10_data_*.json → Claude 分析
```

## 开发命令

```bash
# 启动数据库
docker compose up -d

# 编译 F10 工具
cd f10info && cargo build --release

# 安装 Python 依赖
uv sync

# 测试数据获取
python .claude/skills/stock-analysis/fetch_f10_data.py 688720

# 前端开发
cd web && npm install && npm run dev
```

## 环境变量

- `CK_HOST` / `CK_PORT` / `CK_DATABASE` / `CK_USER` / `CK_PASSWORD` — ClickHouse 连接配置
- `QBRS_F10INFO_PATH` — F10 抓取工具二进制路径（默认 `f10info/target/release/qbrs-f10info`）
