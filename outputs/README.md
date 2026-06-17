# A股供应链「卡点」分析 — 数据目录

本目录存放公司分析数据，前端项目位于 `../web/`。

## 📂 目录结构

```
outputs/
├── companies.json              # 自动生成（由 web/scripts/generate-companies.js）
│
├── 移远通信_603236/
│   ├── meta.json               # 公司元数据（rating/class/summary/position）
│   ├── 移远通信_Serenity卡点分析.md
│   ├── 移远通信_供应链分析.md
│   └── 移远通信_supply_chain_data.json
│
└── ... (更多公司)
```

## ➕ 新增公司

1. 创建目录 `outputs/{公司名}_{6位股票代码}/`
2. 生成 3 个分析文件：
   - `{公司名}_Serenity卡点分析.md`
   - `{公司名}_供应链分析.md`
   - `{公司名}_supply_chain_data.json`
3. 创建 `meta.json`：
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

`class` 可选值：`strong`、`medium`、`weak`、`avoid`

4. 运行 `cd ../web && npm run dev` — 自动识别新公司

## 📊 查看数据

前端项目位于 `../web/`，启动后自动加载本目录数据。

```bash
cd ../web
npm install
npm run dev
```
