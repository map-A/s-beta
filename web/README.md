# A股供应链「卡点」分析 — 前端展示系统

基于 React 19 + Vite 8 的数据可视化前端，自动识别 `outputs/` 目录中的公司数据。

## 🚀 快速开始

```bash
cd web
npm install
npm run dev
```

访问 http://localhost:3000

## 📂 目录结构

```
web/
├── index.html              # 入口页面
├── package.json            # 依赖和脚本
├── vite.config.js          # Vite 配置
├── scripts/
│   └── generate-companies.js  # 自动扫描生成 companies.json
├── public/
│   └── data -> ../../outputs  # 符号链接，自动包含数据目录
└── src/
    ├── main.jsx
    ├── App.jsx
    └── components/
        ├── CompanyCard.jsx    # 公司卡片组件
        ├── MarkdownModal.jsx  # Markdown 报告弹窗
        └── NetworkModal.jsx   # 供应链网络图弹窗
```

## ➕ 新增公司（自动识别）

1. 将分析文件放入 `outputs/{公司名}_{股票代码}/` 目录
2. 在该目录创建 `meta.json`：
   ```json
   {
     "code": "股票代码",
     "name": "公司名称",
     "rating": "⭐⭐⭐⭐⭐",
     "class": "strong",
     "summary": "一句话投资总结",
     "position": "建议仓位"
   }
   ```
3. 运行 `npm run dev` — 自动扫描并显示新公司

`class` 可选值：`strong`（绿色）、`medium`（橙色）、`weak`（红色）、`avoid`（红色）

## 📜 可用命令

| 命令 | 说明 |
|------|------|
| `npm run dev` | 启动开发服务器（自动扫描数据） |
| `npm run build` | 构建生产版本（自动扫描数据） |
| `npm run sync-data` | 手动扫描并重新生成 companies.json |
| `npm run preview` | 预览生产构建 |

## 🔧 数据目录结构

```
outputs/
├── companies.json              # 自动生成，勿手动编辑
├── 移远通信_603236/
│   ├── meta.json               # 公司元数据
│   ├── 移远通信_Serenity卡点分析.md
│   ├── 移远通信_供应链分析.md
│   └── 移远通信_supply_chain_data.json
└── ...
```
