#!/usr/bin/env python3
"""
供应链数据模型

统一的JSON数据格式，用于前端可视化
"""
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional
import json


@dataclass
class Node:
    """图节点"""
    id: str  # 唯一标识，如 "002466"
    name: str  # 显示名称，如 "天齐锂业"
    category: str  # 类别：upstream, competitor, target, downstream
    business: str  # 主营业务
    symbolSize: Optional[int] = None  # 节点大小（目标公司更大）

    def to_dict(self) -> Dict:
        d = asdict(self)
        if self.symbolSize is None:
            d['symbolSize'] = 50 if self.category == 'target' else 30
        return d


@dataclass
class Edge:
    """图的边"""
    source: str  # 源节点ID
    target: str  # 目标节点ID
    relation: str  # 关系类型：supply, compete, customer

    def to_dict(self) -> Dict:
        return asdict(self)


@dataclass
class SupplyChainGraph:
    """供应链图数据结构"""
    company_code: str  # 股票代码
    company_name: str  # 公司名称
    analysis_date: str  # 分析日期
    nodes: List[Node]
    edges: List[Edge]

    def to_json(self) -> str:
        """转换为JSON字符串"""
        data = {
            'company_code': self.company_code,
            'company_name': self.company_name,
            'analysis_date': self.analysis_date,
            'nodes': [node.to_dict() for node in self.nodes],
            'edges': [edge.to_dict() for edge in self.edges],
            'categories': [
                {'name': 'upstream', 'label': '上游供应商'},
                {'name': 'competitor', 'label': '竞品公司'},
                {'name': 'target', 'label': '目标公司'},
                {'name': 'downstream', 'label': '下游客户'},
            ]
        }
        return json.dumps(data, ensure_ascii=False, indent=2)

    def save(self, filepath: str):
        """保存为JSON文件"""
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(self.to_json())


@dataclass
class CompanyAnalysis:
    """公司分析数据"""
    code: str
    name: str
    industry: str
    business: str
    competitors: List[Dict[str, str]]  # [{"name": "...", "business": "..."}]
    upstream: List[Dict[str, str]]
    downstream: List[Dict[str, str]]

    def to_supply_chain_graph(self, analysis_date: str) -> SupplyChainGraph:
        """转换为供应链图数据"""
        nodes = []
        edges = []

        # 目标公司节点
        target_id = f"target_{self.code}"
        nodes.append(Node(
            id=target_id,
            name=f"{self.name}\n{self.code}",
            category='target',
            business=self.business,
            symbolSize=60
        ))

        # 上游供应商
        for i, supplier in enumerate(self.upstream):
            node_id = f"upstream_{i}"
            nodes.append(Node(
                id=node_id,
                name=supplier['name'],
                category='upstream',
                business=supplier['business']
            ))
            edges.append(Edge(
                source=node_id,
                target=target_id,
                relation='supply'
            ))

        # 竞品公司
        for i, competitor in enumerate(self.competitors):
            node_id = f"competitor_{i}"
            nodes.append(Node(
                id=node_id,
                name=competitor['name'],
                category='competitor',
                business=competitor['business']
            ))
            # 竞品之间的竞争关系（可选）
            if i > 0:
                edges.append(Edge(
                    source=f"competitor_{i-1}",
                    target=node_id,
                    relation='compete'
                ))

        # 下游客户
        for i, customer in enumerate(self.downstream):
            node_id = f"downstream_{i}"
            nodes.append(Node(
                id=node_id,
                name=customer['name'],
                category='downstream',
                business=customer['business']
            ))
            edges.append(Edge(
                source=target_id,
                target=node_id,
                relation='customer'
            ))

        return SupplyChainGraph(
            company_code=self.code,
            company_name=self.name,
            analysis_date=analysis_date,
            nodes=nodes,
            edges=edges
        )
