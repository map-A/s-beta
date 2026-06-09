# {{company_name}}（{{company_code}}）供应链分析报告

**生成日期**: {{analysis_date}}  
**所属行业**: {{industry}}  
**主营业务**: {{business_scope}}

---

## 公司概况

**股票代码**: {{company_code}}  
**公司名称**: {{company_name}}  
**上市板块**: {{board}}  
**当前价格**: {{current_price}}  
**市值**: {{market_cap}}

---

## 竞品公司（含自己）

| 公司名称 | 股票代码 | 主营业务 |
|---------|---------|---------|
{{#competitors}}
| {{name}} | {{code}} | {{business}} |
{{/competitors}}

---

## 上游供应商

| 供应商名称 | 类型 | 主营业务 |
|---------|------|---------|
{{#upstream}}
| {{name}} | {{type}} | {{business}} |
{{/upstream}}

**关键依赖**:
{{upstream_notes}}

---

## 下游客户

| 客户名称 | 类型 | 主营业务 |
|---------|------|---------|
{{#downstream}}
| {{name}} | {{type}} | {{business}} |
{{/downstream}}

**终端应用场景**:
{{downstream_notes}}

---

## 供应链关键洞察

### 1. 产业链位置
{{supply_chain_position}}

### 2. 竞争格局
{{competition_landscape}}

### 3. 上下游议价能力
{{bargaining_power}}

---

## 风险因素

### 供应链风险
{{supply_chain_risks}}

### 需求侧风险
{{demand_risks}}

### 其他风险
{{other_risks}}

---

## 结论

{{conclusion}}

---

**报告生成**: {{analysis_date}}  
**数据来源**: TDX F10、公司公告、网络公开信息
