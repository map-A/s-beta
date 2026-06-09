#!/usr/bin/env python3
"""
HTML可视化生成器

将供应链图数据渲染为交互式HTML
"""
import json
from pathlib import Path
from typing import Dict
from .models import SupplyChainGraph


class HTMLVisualizer:
    """HTML可视化生成器"""

    def __init__(self, template_dir: str = "templates"):
        self.template_dir = Path(template_dir)
        self.template_path = self.template_dir / "supply_chain_graph.html"

        if not self.template_path.exists():
            raise FileNotFoundError(f"模板文件不存在: {self.template_path}")

    def generate(self, graph: SupplyChainGraph, output_path: str) -> str:
        """
        生成HTML可视化文件

        Args:
            graph: 供应链图数据
            output_path: 输出HTML文件路径

        Returns:
            生成的HTML文件路径
        """
        # 读取模板
        with open(self.template_path, 'r', encoding='utf-8') as f:
            template = f.read()

        # 准备数据
        graph_data = {
            'company_code': graph.company_code,
            'company_name': graph.company_name,
            'analysis_date': graph.analysis_date,
            'nodes': [node.to_dict() for node in graph.nodes],
            'edges': [edge.to_dict() for edge in graph.edges],
            'categories': [
                {'name': 'upstream', 'label': '上游供应商'},
                {'name': 'competitor', 'label': '竞品公司'},
                {'name': 'target', 'label': '目标公司'},
                {'name': 'downstream', 'label': '下游客户'},
            ]
        }

        # 替换模板变量
        html = template.replace('{{company_name}}', graph.company_name)
        html = html.replace('{{company_code}}', graph.company_code)
        html = html.replace('{{analysis_date}}', graph.analysis_date)
        html = html.replace('{{graph_data}}', json.dumps(graph_data, ensure_ascii=False))

        # 写入文件
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html)

        return str(output_path)

    def generate_multi_company(self, graphs: list[SupplyChainGraph], output_path: str) -> str:
        """
        生成多公司组合可视化（未来扩展）

        Args:
            graphs: 多个供应链图数据
            output_path: 输出HTML文件路径

        Returns:
            生成的HTML文件路径
        """
        # TODO: 实现多公司组合可视化
        pass


def generate_html_visualization(graph: SupplyChainGraph, output_path: str) -> str:
    """
    便捷函数：生成HTML可视化

    Args:
        graph: 供应链图数据
        output_path: 输出HTML文件路径

    Returns:
        生成的HTML文件路径
    """
    visualizer = HTMLVisualizer()
    return visualizer.generate(graph, output_path)
