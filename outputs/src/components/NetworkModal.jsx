import React, { useState, useEffect, useRef } from 'react'
import * as echarts from 'echarts/core'
import { GraphChart } from 'echarts/charts'
import { TooltipComponent, LegendComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import './NetworkModal.css'

echarts.use([GraphChart, TooltipComponent, LegendComponent, CanvasRenderer])

function NetworkModal({ company, onClose }) {
  const chartRef = useRef(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [data, setData] = useState(null)

  useEffect(() => {
    const path = `/${company.name}_${company.code}/${company.name}_supply_chain_data.json`

    fetch(path)
      .then(res => res.json())
      .then(jsonData => {
        setData(jsonData)
        setLoading(false)
      })
      .catch(err => {
        console.error('Failed to load supply chain data:', err)
        setError('供应链数据加载失败')
        setLoading(false)
      })
  }, [company])

  // 单独的useEffect用于图表初始化，等待data和chartRef都ready
  useEffect(() => {
    if (!data || !chartRef.current) {
      return
    }

    renderChart(data)
  }, [data])

  const renderChart = (data) => {
    if (!chartRef.current) {
      console.error('Chart ref is null')
      return
    }

    const chart = echarts.init(chartRef.current)

    // 颜色映射
    const colorMap = {
      'upstream': '#91cc75',
      'competitor': '#5470c6',
      'target': '#ee6666',
      'downstream': '#fac858'
    }

    // 预先计算节点位置 - 层次化布局
    const positions = calculateHierarchicalLayout(data.nodes)

    const nodes = data.nodes.map(node => ({
      id: node.id,
      name: node.name,
      symbolSize: node.symbolSize || 40,
      category: node.category,
      business: node.business || '', // 添加业务描述
      x: positions[node.id]?.x,
      y: positions[node.id]?.y,
      fixed: true, // 固定位置
      itemStyle: {
        color: colorMap[node.category],
        borderWidth: node.category === 'target' ? 3 : 1,
        borderColor: node.category === 'target' ? '#c53030' : '#fff'
      },
      label: {
        show: true,
        fontSize: node.category === 'target' ? 14 : 11,
        fontWeight: node.category === 'target' ? 'bold' : 'normal',
        color: '#333'
      }
    }))

    const edges = data.edges.map(edge => ({
      source: edge.source,
      target: edge.target,
      lineStyle: {
        color: edge.relation === 'compete' ? '#cbd5e0' : '#999',
        type: edge.relation === 'compete' ? 'dashed' : 'solid',
        width: edge.relation === 'compete' ? 1 : 2,
        curveness: edge.relation === 'compete' ? 0.2 : 0
      },
      label: {
        show: false
      }
    }))

    const option = {
      tooltip: {
        trigger: 'item',
        formatter: (params) => {
          if (params.dataType === 'node') {
            return `<strong>${params.name}</strong><br/>${params.data.business || ''}`
          }
          return ''
        }
      },
      legend: [{
        data: ['上游供应商', '竞品公司', '目标公司', '下游客户'],
        orient: 'horizontal',
        bottom: 10,
        itemGap: 20
      }],
      series: [{
        type: 'graph',
        layout: 'none', // 禁用自动布局
        data: nodes,
        links: edges,
        categories: data.categories,
        roam: true,
        draggable: false, // 禁止拖拽
        emphasis: {
          focus: 'adjacency',
          lineStyle: { width: 4 }
        },
        lineStyle: {
          curveness: 0
        }
      }]
    }

    chart.setOption(option)

    // 窗口resize时重绘
    const handleResize = () => chart.resize()
    window.addEventListener('resize', handleResize)

    return () => {
      window.removeEventListener('resize', handleResize)
      chart.dispose()
    }
  }

  // 层次化布局算法 - 垂直分层
  const calculateHierarchicalLayout = (nodes) => {
    const positions = {}
    const width = 800
    const height = 600
    const margin = 100

    // 按类别分组
    const upstream = nodes.filter(n => n.category === 'upstream')
    const competitor = nodes.filter(n => n.category === 'competitor')
    const target = nodes.filter(n => n.category === 'target')
    const downstream = nodes.filter(n => n.category === 'downstream')

    // 层次定义 (y坐标)
    const layers = {
      upstream: margin, // 顶部
      target: height / 2, // 中间
      downstream: height - margin, // 底部
      competitor: height / 2 // 中间偏右侧
    }

    // 上游 - 顶部水平排列
    upstream.forEach((node, i) => {
      positions[node.id] = {
        x: (width / (upstream.length + 1)) * (i + 1),
        y: layers.upstream
      }
    })

    // 目标公司 - 中心
    target.forEach(node => {
      positions[node.id] = {
        x: width / 2,
        y: layers.target
      }
    })

    // 下游 - 底部水平排列
    downstream.forEach((node, i) => {
      positions[node.id] = {
        x: (width / (downstream.length + 1)) * (i + 1),
        y: layers.downstream
      }
    })

    // 竞品 - 中间层右侧垂直排列
    competitor.forEach((node, i) => {
      positions[node.id] = {
        x: width - margin,
        y: layers.target - 80 + (i * 60)
      }
    })

    return positions
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content modal-content-large" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{company.name} ({company.code}) - 供应链网络图</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <div className="modal-body">
          {loading && <div className="loading-spinner">加载中...</div>}
          {error && <div className="error-message">{error}</div>}
          {!loading && !error && (
            <div ref={chartRef} className="chart-container"></div>
          )}
        </div>
      </div>
    </div>
  )
}

export default NetworkModal
