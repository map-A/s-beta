import React from 'react'
import './CompanyCard.css'

function CompanyCard({ company, onOpenModal }) {
  return (
    <div className={`card card-${company.class}`}>
      <div className="card-title">{company.name}</div>
      <div className="card-code">{company.code}</div>
      <div className="card-rating">{company.rating}</div>
      <div className="card-summary">{company.summary}</div>
      <div className="card-position">
        <span className="position-label">建议仓位：</span>
        <span className="position-value">{company.position}</span>
      </div>
      <div className="card-actions">
        <button
          className="btn btn-primary"
          onClick={() => onOpenModal(company, 'serenity')}
        >
          卡点分析
        </button>
        <button
          className="btn btn-secondary"
          onClick={() => onOpenModal(company, 'supply')}
        >
          供应链
        </button>
        <button
          className="btn btn-secondary"
          onClick={() => onOpenModal(company, 'network')}
        >
          网络图
        </button>
      </div>
    </div>
  )
}

export default CompanyCard
