# A股供应链「卡点」分析系统

基于 **Serenity「卡点」方法论** 的 A 股供应链分析工具。通过 Claude Code 的 skill 机制，自动获取股票 F10 数据并生成深度供应链分析报告。

> *核心理念：不炒 xxx 概念股，炒 xxx 离不开的东西*

---

## 前置条件

| 工具 | 用途 | 安装方式 |
|------|------|----------|
| **Docker** | 运行 ClickHouse 数据库 | [docker.com](https://docs.docker.com/get-docker/) |
| **Rust toolchain** | 编译 F10 数据抓取工具 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Python 3.11+** | 数据查询脚本 | [python.org](https://www.python.org/) |
| **uv** | Python 包管理 | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| **Claude Code** | AI 分析引擎 | `npm install -g @anthropic-ai/claude-code` |
| **Node.js 18+** | 前端展示（可选） | [nodejs.org](https://nodejs.org/) |

**数据源说明**：F10 数据通过通达信 TQLEX 接口获取，需要能访问 `180.168.205.46:7626`。如果你没有该数据源访问权限，仍可使用 skill 的 WebSearch 模式进行分析（跳过 F10 数据获取步骤）。

---

## 快速开始

### 1. 启动 ClickHouse 数据库

```bash
docker compose up -d
```

等待健康检查通过（约 30 秒）：

```bash
docker compose ps  # 确认 STATUS 为 healthy
```

### 2. 编译 F10 数据抓取工具

```bash
cd f10info
cargo build --release
cd ..
```

首次编译需要下载依赖，约 2-5 分钟。编译产物在 `f10info/target/release/qbrs-f10info`。

### 3. 安装 Python 依赖

```bash
uv sync
```

### 4. 测试数据获取（可选）

```bash
python .claude/skills/stock-analysis/fetch_f10_data.py 688720
```

成功后会在 `/tmp/f10_data_688720.json` 生成 F10 数据文件。

---

## 使用方法

### 方式一：Claude Code Skill（推荐）

在项目目录下启动 Claude Code，然后使用 skill：

```bash
cd stock_beta
claude
```

在 Claude Code 中输入：

```
/stock-analysis 688720
```

或直接描述：

```
分析一下中芯国际
```

Claude 会自动执行完整的 9 步分析流程：
1. 获取 F10 数据（从 ClickHouse）
2. WebSearch 补充最新信息
3. Serenity 卡点分析（7 步流程）
4. 生成 Serenity 卡点分析报告
5. 生成供应链分析报告
6. 生成供应链 JSON 数据
7. 生成公司索引 meta.json
8. 验证输出完整性

分析结果保存在 `outputs/{公司名}_{代码}/` 目录下。

### 方式二：手动运行数据获取

```bash
# 抓取数据到 ClickHouse 并导出 JSON
python .claude/skills/stock-analysis/fetch_f10_data.py 688720

# 仅查询已有数据（跳过抓取）
python .claude/skills/stock-analysis/fetch_f10_data.py 688720 --query-only

# 仅抓取到数据库
python .claude/skills/stock-analysis/fetch_f10_data.py 688720 --fetch-only
```

### 查看前端展示（可选）

```bash
cd web
npm install
npm run dev
```

访问 http://localhost:3000 查看所有已分析公司的可视化展示。

---

## 项目结构

```
stock_beta/
├── .claude/
│   ├── settings.local.json        # Claude Code 权限配置
│   └── skills/
│       └── stock-analysis/        # Skill 定义（项目内）
│           ├── SKILL.md           # Skill 主文件（9步分析流程）
│           ├── fetch_f10_data.py  # F10 数据获取/查询脚本
│           └── references/        # Serenity 方法论知识库
│               ├── methodology.md
│               ├── verification.md
│               ├── chain_map.md
│               ├── cjk_addendum.md
│               └── supply_chain_graph.json
│
├── f10info/                       # F10 数据抓取工具（Rust）
│   ├── Cargo.toml
│   └── src/                       # qbrs-f10info 源码
│       ├── main.rs                # CLI 入口
│       ├── client.rs              # 通达信 TQLEX HTTP 客户端
│       ├── runner.rs              # 批量抓取调度
│       ├── schema.rs              # ClickHouse 建表
│       ├── pages/                 # 14 个 F10 页面模块
│       └── schema/                # 66 张表的 DDL
│
├── templates/                     # 分析报告模板
│   ├── serenity_chokepoint_template.md
│   ├── supply_chain_template.md
│   ├── supply_chain_json_template.json
│   └── supply_chain_graph.html
│
├── outputs/                       # 分析输出（每公司一个目录）
│   ├── companies.json             # 自动生成的公司索引
│   └── {公司名}_{代码}/          # 每个公司的分析结果
│       ├── meta.json
│       ├── *_Serenity卡点分析.md
│       ├── *_供应链分析.md
│       └── *_supply_chain_data.json
│
├── web/                           # React 前端展示
│   ├── src/
│   ├── scripts/generate-companies.js
│   └── package.json
│
├── docker-compose.yml             # ClickHouse 数据库
├── data/                          # ClickHouse 数据持久化（gitignore）
├── pyproject.toml                 # Python 项目配置
└── README.md
```

---

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `CK_HOST` | `localhost` | ClickHouse 地址 |
| `CK_PORT` | `8123` | ClickHouse HTTP 端口 |
| `CK_DATABASE` | `f10` | 数据库名 |
| `CK_USER` | `default` | 数据库用户 |
| `CK_PASSWORD` | (空) | 数据库密码 |
| `QBRS_F10INFO_PATH` | `f10info/target/release/qbrs-f10info` | F10 抓取工具路径 |

---

## 常见问题

### Q: 没有通达信 TQLEX 数据源怎么办？

Skill 的核心分析能力（Serenity 方法论）不依赖 F10 数据。你可以：
- 跳过第 1 步（F10 数据获取）
- Claude 会通过 WebSearch 获取公开数据进行分析
- 分析质量会略有下降（缺少前5大客户/供应商等年报数据）

### Q: ClickHouse 连接失败？

```bash
# 检查容器状态
docker compose ps

# 查看日志
docker compose logs clickhouse

# 手动测试连接
curl 'http://localhost:8123/?query=SELECT%201'
```

### Q: Rust 编译失败？

确保 Rust 工具链是最新的：

```bash
rustup update stable
cd f10info && cargo build --release
```

### Q: 如何更新已分析公司的数据？

重新运行 skill 即可，输出文件会被覆盖：

```
/stock-analysis 688720
```

### Q: 如何批量分析多只股票？

在 Claude Code 中依次运行：

```
/stock-analysis 688720
/stock-analysis 688008
/stock-analysis 300476
```

---

## 分析框架

本系统使用 Serenity「卡点」方法论，核心是 4 条标准（必须全部满足）：

1. **霍尔木兹海峡级必需性** — 人人需要，几乎无人能造
2. **供给高度集中 + 扩产极难** — 寡头供给，2-5 年认证周期
3. **市值严重错配** — 市值 vs 下游 BOM 贡献严重失衡
4. **失效模式测试** — 不会被设计绕开，收入影响实质

详见 `.claude/skills/stock-analysis/references/methodology.md`。

---

## License

MIT
