import React, { useState, useEffect } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import './MarkdownModal.css'

function MarkdownModal({ company, type, onClose }) {
  const [content, setContent] = useState('')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const filename = type === 'serenity'
      ? `${company.name}_Serenity卡点分析.md`
      : `${company.name}_供应链分析.md`

    const path = `/data/${company.name}_${company.code}/${filename}`

    fetch(path)
      .then(res => {
        if (!res.ok) {
          throw new Error(`HTTP ${res.status}: ${res.statusText}`)
        }
        return res.text()
      })
      .then(text => {
        // Check if response is HTML (404 page)
        if (text.trim().startsWith('<!DOCTYPE') || text.trim().startsWith('<html')) {
          throw new Error('File not found (404)')
        }
        setContent(text)
        setLoading(false)
      })
      .catch(err => {
        console.error('Failed to load markdown:', err)
        if (err.message.includes('404') || err.message.includes('File not found')) {
          setContent(`# 📄 文件不存在 (404)\n\n**${filename}** 尚未生成。\n\n该公司的分析报告可能还未完成，请稍后再试。`)
        } else {
          setContent('# ⚠️ 加载失败\n\n无法加载分析报告，请检查网络连接或文件是否存在。')
        }
        setLoading(false)
      })
  }, [company, type])

  const title = type === 'serenity'
    ? `${company.name} - Serenity卡点分析`
    : `${company.name} - 供应链分析`

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{title}</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <div className="modal-body">
          {loading ? (
            <div className="loading-spinner">加载中...</div>
          ) : (
            <div className="markdown-content">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {content}
              </ReactMarkdown>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default MarkdownModal
