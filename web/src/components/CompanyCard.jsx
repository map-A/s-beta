import React, { useState } from 'react'
import './CompanyCard.css'

function CompanyCard({ company, onOpenModal, onToggleFavorite, isFavorite, animating }) {
  const [ripple, setRipple] = useState(false)

  const handleFavoriteClick = (e) => {
    e.stopPropagation()
    setRipple(true)
    setTimeout(() => setRipple(false), 500)
    onToggleFavorite?.(company)
  }

  const handleCardDoubleClick = () => {
    onOpenModal(company, 'double-click')
  }

  return (
    <div 
      className={`ink-card ink-card-${company.class} ${animating ? 'ink-card-fly' : ''}`}
      onDoubleClick={handleCardDoubleClick}
    >
      {/* Ink wash decorations */}
      <div className="ink-wash-tl" />
      <div className="ink-wash-br" />

      {/* Top section */}
      <div className="ink-card-head">
        <div className="ink-card-title-area">
          <div className="ink-card-name">{company.name}</div>
          <div className="ink-card-code">{company.code}</div>
        </div>
        <button
          className={`ink-card-fav ${isFavorite ? 'active' : ''} ${ripple ? 'ink-ripple' : ''}`}
          onClick={handleFavoriteClick}
          title={isFavorite ? '点击取消自选' : '加入自选'}
        >
          {isFavorite ? '⭐' : '☆'}
        </button>
      </div>

      {/* Rating */}
      <div className="ink-card-rating">{company.rating}</div>

      {/* Divider — brush stroke style */}
      <div className="ink-divider" />

      {/* Summary */}
      <div className="ink-card-summary">{company.summary}</div>

      {/* Position */}
      <div className="ink-card-position">
        <span className="ink-pos-label">建议仓位</span>
        <span className="ink-pos-value">{company.position}</span>
      </div>

      {/* Actions */}
      <div className="ink-card-actions" onDoubleClick={e => e.stopPropagation()}>
        <button
          className="ink-btn ink-btn-primary"
          onClick={() => onOpenModal(company, 'serenity')}
        >
          卡点分析
        </button>
        <button
          className="ink-btn"
          onClick={() => onOpenModal(company, 'supply')}
        >
          供应链
        </button>
        <button
          className="ink-btn"
          onClick={() => onOpenModal(company, 'network')}
        >
          网络图
        </button>
      </div>
    </div>
  )
}

export default CompanyCard
