import React, { useState, useEffect, useMemo, useCallback, useRef } from 'react'
import CompanyCard from './components/CompanyCard'
import NetworkModal from './components/NetworkModal'
import MarkdownModal from './components/MarkdownModal'
import Sidebar from './components/Sidebar'
import GroupPickerModal from './components/GroupPickerModal'
import './App.css'

const PAGE_SIZE = 20
const STORAGE_KEY = 'ink-watchlist'

function getRatingScore(rating) {
  if (!rating) return 0
  return (rating.match(/⭐/g) || []).length
}

function loadWatchlist() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch {}
  return { default: { name: '默认自选', stocks: [] } }
}

function saveWatchlist(data) {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify(data)) } catch {}
}

let nextGroupId = Date.now()

function App() {
  const [companies, setCompanies] = useState([])
  const [selectedCompany, setSelectedCompany] = useState(null)
  const [modalType, setModalType] = useState(null)
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [sortDesc, setSortDesc] = useState(true)
  const [displayCount, setDisplayCount] = useState(PAGE_SIZE)
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [watchlist, setWatchlist] = useState(loadWatchlist)
  const [animCode, setAnimCode] = useState(null)
  const [groupPickerCompany, setGroupPickerCompany] = useState(null)
  const lastClickRef = useRef({})

  // Persist watchlist
  useEffect(() => { saveWatchlist(watchlist) }, [watchlist])

  useEffect(() => {
    fetch('/data/companies.json')
      .then(res => res.json())
      .then(data => { setCompanies(data); setLoading(false) })
      .catch(err => { console.error('Failed to load companies:', err); setLoading(false) })
  }, [])

  // Collect all favorited codes
  const favoriteCodes = useMemo(() => {
    const set = new Set()
    Object.values(watchlist).forEach(g => g.stocks.forEach(s => set.add(s.code)))
    return set
  }, [watchlist])

  // ── Filter & sort ──

  const filtered = useMemo(() => {
    let list = companies
    if (searchQuery.trim()) {
      const q = searchQuery.trim().toLowerCase()
      list = list.filter(c =>
        c.name.toLowerCase().includes(q) || c.code.toLowerCase().includes(q)
      )
    }
    return [...list].sort((a, b) => {
      const diff = getRatingScore(a.rating) - getRatingScore(b.rating)
      return sortDesc ? -diff : diff
    })
  }, [companies, searchQuery, sortDesc])

  const displayed = useMemo(() => filtered.slice(0, displayCount), [filtered, displayCount])
  const hasMore = displayCount < filtered.length

  useEffect(() => { setDisplayCount(PAGE_SIZE) }, [searchQuery, sortDesc])

  // ── Watchlist actions ──

  const addToGroup = useCallback((groupId, company) => {
    setWatchlist(prev => {
      const group = prev[groupId]
      if (!group || group.stocks.some(s => s.code === company.code)) return prev
      return { ...prev, [groupId]: { ...group, stocks: [...group.stocks, { name: company.name, code: company.code }] } }
    })
  }, [])

  const handleToggleFavorite = useCallback((company) => {
    if (favoriteCodes.has(company.code)) {
      // Remove from all groups
      setWatchlist(prev => {
        const next = {}
        Object.entries(prev).forEach(([id, g]) => {
          next[id] = { ...g, stocks: g.stocks.filter(s => s.code !== company.code) }
        })
        return next
      })
    } else {
      // Show group picker
      setGroupPickerCompany(company)
    }
  }, [favoriteCodes])



  // Single-click
  const handleCardClick = useCallback((company, type) => {

    setSelectedCompany(company)
    setModalType(type)
  })

  const handleCloseModal = () => {
    setSelectedCompany(null)
    setModalType(null)
  }

  // Sidebar actions
  const handleAddGroup = (name) => {
    const id = 'group-' + (nextGroupId++)
    setWatchlist(prev => ({ ...prev, [id]: { name, stocks: [] } }))
  }

  const handleRemoveGroup = (id) => {
    setWatchlist(prev => {
      const next = { ...prev }
      delete next[id]
      return next
    })
  }

  const handleRenameGroup = (id, name) => {
    setWatchlist(prev => {
      if (!prev[id]) return prev
      return { ...prev, [id]: { ...prev[id], name } }
    })
  }

  const handleRemoveStock = (groupId, code) => {
    setWatchlist(prev => {
      const group = prev[groupId]
      if (!group) return prev
      return { ...prev, [groupId]: { ...group, stocks: group.stocks.filter(s => s.code !== code) } }
    })
  }

  const handleSelectStock = (stock) => {
    const company = companies.find(c => c.code === stock.code)
    if (company) {
      setSelectedCompany(company)
      setModalType('serenity')
    }
  }

  // Group picker actions
  const handleGroupPickerConfirm = useCallback((groupId) => {
    if (!groupPickerCompany) return
    addToGroup(groupId, groupPickerCompany)
    if (!sidebarOpen) setSidebarOpen(true)
    setAnimCode(groupPickerCompany.code)
    setTimeout(() => setAnimCode(null), 600)
    setGroupPickerCompany(null)
  }, [groupPickerCompany, addToGroup, sidebarOpen])

  const handleGroupPickerNewGroup = useCallback((name) => {
    if (!groupPickerCompany) return
    const id = 'group-' + (nextGroupId++)
    setWatchlist(prev => ({
      ...prev,
      [id]: { name, stocks: [{ name: groupPickerCompany.name, code: groupPickerCompany.code }] }
    }))
    if (!sidebarOpen) setSidebarOpen(true)
    setAnimCode(groupPickerCompany.code)
    setTimeout(() => setAnimCode(null), 600)
    setGroupPickerCompany(null)
  }, [groupPickerCompany, sidebarOpen])

  if (loading) {
    return (
      <div className="loading-container">
        <div className="loading-text">正在加载数据...</div>
      </div>
    )
  }

  return (
    <>
      <Sidebar
        isOpen={sidebarOpen}
        onToggle={() => setSidebarOpen(v => !v)}
        groups={watchlist}
        onAddGroup={handleAddGroup}
        onRemoveGroup={handleRemoveGroup}
        onRenameGroup={handleRenameGroup}
        onRemoveStock={handleRemoveStock}
        onSelectStock={handleSelectStock}
      />

      <div className={`main-content ${sidebarOpen ? 'sidebar-open' : ''}`}>
        <header className="header">
          <h1>A股供应链「卡点」分析</h1>
          <div className="toolbar">
            <div className="search-box">
              <span className="search-icon">🔍</span>
              <input
                type="text"
                className="search-input"
                placeholder="搜索股票代码或名称..."
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
              />
              {searchQuery && (
                <button className="search-clear" onClick={() => setSearchQuery('')}>✕</button>
              )}
            </div>
            <button
              className={`sort-btn ${sortDesc ? 'sort-desc' : 'sort-asc'}`}
              onClick={() => setSortDesc(v => !v)}
            >
              卡点评分 {sortDesc ? '↓' : '↑'}
            </button>
          </div>
          <div className="header-hint">双击卡片或点击星号选择分组加入自选 · 右键分组可重命名</div>
        </header>

        {filtered.length === 0 ? (
          <div className="empty-state">
            {searchQuery ? '没有找到匹配的股票' : '暂无数据'}
          </div>
        ) : (
          <>
            <div className="result-info">
              共 {filtered.length} 只股票{searchQuery ? ` · 搜索「${searchQuery}」` : ''}
            </div>
            <div className="cards-grid">
              {displayed.map(company => (
                <CompanyCard
                  key={company.code}
                  company={company}
                  onOpenModal={handleCardClick}
                  onToggleFavorite={handleToggleFavorite}
                  isFavorite={favoriteCodes.has(company.code)}
                  animating={animCode === company.code}
                />
              ))}
            </div>
            {hasMore && (
              <div className="load-more">
                <button className="load-more-btn" onClick={() => setDisplayCount(v => v + PAGE_SIZE)}>
                  加载更多 ({displayed.length}/{filtered.length})
                </button>
              </div>
            )}
          </>
        )}
      </div>

      {selectedCompany && modalType === 'network' && (
        <NetworkModal company={selectedCompany} onClose={handleCloseModal} />
      )}
      {selectedCompany && (modalType === 'serenity' || modalType === 'supply') && (
        <MarkdownModal company={selectedCompany} type={modalType} onClose={handleCloseModal} />
      )}
      {groupPickerCompany && (
        <GroupPickerModal
          company={groupPickerCompany}
          groups={watchlist}
          onConfirm={handleGroupPickerConfirm}
          onConfirmNewGroup={handleGroupPickerNewGroup}
          onClose={() => setGroupPickerCompany(null)}
        />
      )}
    </>
  )
}

export default App
