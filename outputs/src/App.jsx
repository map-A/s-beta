import React, { useState, useEffect } from 'react'
import CompanyCard from './components/CompanyCard'
import NetworkModal from './components/NetworkModal'
import MarkdownModal from './components/MarkdownModal'
import './App.css'

function App() {
  const [companies, setCompanies] = useState([])
  const [selectedCompany, setSelectedCompany] = useState(null)
  const [modalType, setModalType] = useState(null) // 'network' | 'serenity' | 'supply'
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetch('/companies.json')
      .then(res => res.json())
      .then(data => {
        setCompanies(data)
        setLoading(false)
      })
      .catch(err => {
        console.error('Failed to load companies:', err)
        setLoading(false)
      })
  }, [])

  const handleOpenModal = (company, type) => {
    setSelectedCompany(company)
    setModalType(type)
  }

  const handleCloseModal = () => {
    setSelectedCompany(null)
    setModalType(null)
  }

  if (loading) {
    return (
      <div className="loading-container">
        <div className="loading-text">正在加载数据...</div>
      </div>
    )
  }

  return (
    <div className="app">
      <header className="header">
        <h1>🎯 A股供应链「卡点」分析汇总</h1>
        <div className="stats">
          <div className="stat-item">
            <div className="stat-number">{companies.length}</div>
            <div className="stat-label">已完成分析</div>
          </div>
          <div className="stat-item">
            <div className="stat-number">2026/06/09</div>
            <div className="stat-label">分析日期</div>
          </div>
        </div>
      </header>

      <div className="cards-grid">
        {companies.map(company => (
          <CompanyCard
            key={company.code}
            company={company}
            onOpenModal={handleOpenModal}
          />
        ))}
      </div>

      {selectedCompany && modalType === 'network' && (
        <NetworkModal
          company={selectedCompany}
          onClose={handleCloseModal}
        />
      )}

      {selectedCompany && (modalType === 'serenity' || modalType === 'supply') && (
        <MarkdownModal
          company={selectedCompany}
          type={modalType}
          onClose={handleCloseModal}
        />
      )}
    </div>
  )
}

export default App
