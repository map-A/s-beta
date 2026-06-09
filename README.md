# A股供应链「卡点」分析系统

**版本**: 2.0  
**更新日期**: 2026年6月9日  
**分析框架**: Serenity「卡点」方法论 + 供应链分析  
**技术栈**: Python (uv) + ECharts + Serenity Skill

---

## 🎯 系统特色

### ✅ 模板化分析流程
- **统一Markdown模板**：`serenity_chokepoint_template.md` 和 `supply_chain_template.md`
- **标准化JSON数据**：供应链节点和边的统一格式
- **交互式HTML可视化**：基于ECharts的动态网络图

### ✅ 数据源整合
- **AKShare**：获取A股实时行情（涨幅排名）
- **TDX F10接口**：`http://180.168.205.46:7626/site/tdxf10/gg_zxts.html?gp={code}`
- **Serenity Skill**：深度卡点分析（需调用 `/serenity-a-share`）

### ✅ 批量处理能力
- 自动分析涨幅前50股票
- 并行生成：卡点分析 + 供应链分析 + 可视化图表
- 自动生成索引页面

---

## 📂 项目结构

```
stock_prediction/
├── src/                          # 核心代码
│   ├── models.py                     # 数据模型（Node, Edge, SupplyChainGraph）
│   ├── visualizer.py                 # HTML可视化生成器
│   ├── data_fetcher.py               # 数据获取（AKShare）
│   └── __init__.py
│
├── templates/                    # Markdown模板
│   ├── serenity_chokepoint_template.md   # Serenity卡点分析模板
│   ├── supply_chain_template.md          # 供应链分析模板
│   └── supply_chain_graph.html           # ECharts可视化HTML模板
│
├── outputs/                      # 分析输出
│   └── YYYYMMDD/                     # 按日期分组
│       ├── index.html                    # 索引页面
│       └── 公司名_代码/
│           ├── 公司名_Serenity卡点分析.md
│           ├── 公司名_供应链分析.md
│           ├── 公司名_supply_chain_data.json
│           └── 公司名_供应链网络图.html
│
├── main_engine.py                # 主分析引擎
├── cli.py                        # 命令行工具
├── demo_analysis.py              # 演示脚本（模拟数据）
├── pyproject.toml                # uv项目配置
└── README.md                     # 本文档
```

---

## 🚀 快速开始

### 1. 安装依赖

```bash
# 使用uv安装依赖
uv sync
```

### 2. 演示模式（推荐）

```bash
# 使用模拟数据快速体验
uv run python cli.py demo

# 或者直接运行
uv run python demo_analysis.py
```

生成的文件在 `outputs/YYYYMMDD_demo/`，打开 `index.html` 查看。

### 3. 分析单个股票

```bash
# 分析天齐锂业
uv run python cli.py analyze 002466
```

### 4. 批量分析

```bash
# 分析今日涨幅前10股票
uv run python cli.py batch --limit 10

# 分析今日涨幅前50股票（默认）
uv run python cli.py batch
```

---

## 📊 已完成案例

### 1. 天齐锂业（002466）— 锂资源

**卡点评分**: ★★★★☆（强卡点）  
**投资建议**: 【建仓/左侧加仓】

**文档**:
- `天齐锂业_Serenity卡点分析.md` — 7步完整分析
- `天齐锂业_供应链分析.md` — 传统供应链报告
- `天齐锂业_供应链网络图.html` — 交互式可视化

**结论**: 全球锂资源准寡头，控制15-20%供应，盈利拐点确认。

### 2. 豪恩汽电（301488）— 汽车传感器

**卡点评分**: ★☆☆☆☆（弱卡点）  
**投资建议**: 【规避/观察】

**文档**:
- `豪恩汽电_Serenity卡点分析.md`
- `豪恩汽电_供应链分析.md`
- `豪恩汽电_供应链网络图.html`

**结论**: 供给分散，易被绕开，无估值错配。

---

## 🎨 可视化特色

### ECharts交互式网络图

**特性**:
- ✅ **力导向布局**：自动优化节点位置
- ✅ **拖拽交互**：可自由拖动节点
- ✅ **缩放平移**：鼠标滚轮缩放，拖拽平移
- ✅ **悬停提示**：显示节点详细信息
- ✅ **关联高亮**：点击节点高亮相邻节点

**颜色方案**:
- 🟢 **绿色**：上游供应商
- 🔵 **蓝色**：竞品公司
- 🔴 **红色**：目标公司（更大节点）
- 🟡 **黄色**：下游客户

**相比Python matplotlib的优势**:
- ❌ matplotlib：静态图片，无交互，布局固定
- ✅ ECharts：动态交互，自适应布局，Web友好

---

## 📝 Markdown模板说明

### Serenity卡点分析模板

位置：`templates/serenity_chokepoint_template.md`

**核心章节**:
1. **一句话定位** — 公司在产业链的角色
2. **4项标准评估** — 必需性、供给集中度、市值错配、失效模式
3. **信息差评估** — 机构覆盖、散户情绪、北向资金
4. **投资策略框架** — 仓位建议、催化剂、风险因素
5. **证据链** — 财报会、供应链验证、大宗数据

**变量占位符**:
- `{{company_name}}` — 公司名称
- `{{rating}}` — 卡点评分（★★★★☆）
- `{{necessity_result}}` — 通过/不通过
- 更多见模板文件

### 供应链分析模板

位置：`templates/supply_chain_template.md`

**核心章节**:
1. **公司概况** — 基本信息
2. **竞品公司** — 含自己的竞争对手列表
3. **上游供应商** — 依赖关系
4. **下游客户** — 服务对象
5. **供应链关键洞察** — 产业链位置、竞争格局
6. **风险因素** — 供应链风险、需求风险

---

## 🔧 技术架构

### 数据模型（`src/models.py`）

```python
@dataclass
class Node:
    id: str           # 唯一标识
    name: str         # 显示名称
    category: str     # upstream/competitor/target/downstream
    business: str     # 主营业务
    symbolSize: int   # 节点大小

@dataclass
class Edge:
    source: str       # 源节点ID
    target: str       # 目标节点ID
    relation: str     # supply/compete/customer

@dataclass
class SupplyChainGraph:
    company_code: str
    company_name: str
    analysis_date: str
    nodes: List[Node]
    edges: List[Edge]

    def to_json(self) -> str
    def save(self, filepath: str)
```

### 可视化流程

```
供应链数据 (CompanyAnalysis)
    ↓
to_supply_chain_graph()
    ↓
SupplyChainGraph (JSON数据)
    ↓
generate_html_visualization()
    ↓
交互式HTML (ECharts)
```

---

## 🛠️ 扩展指南

### 添加新数据源

编辑 `main_engine.py`，在 `AnalysisEngine` 类中添加方法：

```python
def fetch_new_datasource(self, code: str) -> Dict:
    """新数据源"""
    # 实现数据抓取逻辑
    pass
```

### 调用真实Serenity Skill

替换 `analyze_with_serenity()` 方法中的模拟数据：

```python
def analyze_with_serenity(self, code: str, name: str, tdx_data: Dict) -> Dict:
    # 实际调用
    result = self.call_skill('serenity-a-share', f'{name} {code} {industry}')
    return result
```

### 自定义Markdown模板

1. 复制模板文件到 `templates/`
2. 修改占位符 `{{variable_name}}`
3. 在 `main_engine.py` 中调用新模板

---

## 📚 数据源

1. **AKShare** — A股实时行情、个股信息
2. **TDX F10** — `http://180.168.205.46:7626/site/tdxf10/gg_zxts.html?gp={code}`
3. **Serenity Skill** — 调用 `/serenity-a-share` 进行深度分析
4. **网络搜索** — 财报会、新闻、研报

---

## ⚠️ 注意事项

### 网络问题

如果 AKShare 获取数据失败（代理/防火墙），使用演示模式：

```bash
uv run python demo_analysis.py
```

### TDX接口

`http://180.168.205.46:7626` 是内网IP，需要确保网络可达。

### Serenity Skill调用

当前版本使用模拟数据，完整分析需：

```bash
# 在Claude Code中手动调用
/serenity-a-share 天齐锂业 002466 锂资源 新能源电池 上游原材料 卡点分析
```

---

## 🎯 路线图

### ✅ 已完成

- [x] 数据模型设计
- [x] ECharts可视化模板
- [x] Markdown模板（Serenity + 供应链）
- [x] 批量处理引擎
- [x] CLI工具
- [x] 演示脚本

### 🚧 进行中

- [ ] 集成真实Serenity Skill调用
- [ ] TDX数据解析器
- [ ] Jinja2模板引擎替换字符串替换

### 📅 计划中

- [ ] 多公司组合可视化
- [ ] 实时数据更新（定时任务）
- [ ] Web界面（Flask/FastAPI）
- [ ] 数据库存储（SQLite）
- [ ] 导出PDF报告

---

## 📄 许可证

本项目仅供学习研究使用，不构成投资建议。

**风险提示**：股市有风险，投资需谨慎。

---

## 🙏 致谢

- **Serenity** (@aleabitoreddit) — 卡点方法论
- **AKShare** — 开源金融数据接口
- **ECharts** — 开源可视化库
- **Claude Code** — AI编程助手

---

**最后更新**: 2026年6月9日  
**系统版本**: 2.0 (模板化 + ECharts)  
**作者**: AI + Human Collaboration
