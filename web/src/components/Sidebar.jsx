import React, { useState } from 'react'
import './Sidebar.css'

function Sidebar({
  isOpen,
  onToggle,
  groups,
  onAddGroup,
  onRemoveGroup,
  onRenameGroup,
  onRemoveStock,
  onSelectStock,
}) {
  const [expandedGroups, setExpandedGroups] = useState(() => {
    const map = {}
    Object.keys(groups).forEach(id => { map[id] = true })
    return map
  })
  const [showAddForm, setShowAddForm] = useState(false)
  const [newGroupName, setNewGroupName] = useState('')
  const [editingId, setEditingId] = useState(null)
  const [editName, setEditName] = useState('')
  const [contextMenu, setContextMenu] = useState(null) // { groupId, x, y }

  // Keep newly created groups expanded
  React.useEffect(() => {
    setExpandedGroups(prev => {
      const next = { ...prev }
      Object.keys(groups).forEach(id => {
        if (!(id in next)) next[id] = true
      })
      return next
    })
  }, [groups])

  const toggleGroup = (id) => {
    setExpandedGroups(prev => ({ ...prev, [id]: !prev[id] }))
  }

  const handleAddGroup = () => {
    if (!newGroupName.trim()) return
    onAddGroup(newGroupName.trim())
    setNewGroupName('')
    setShowAddForm(false)
  }

  const handleStartRename = (id, name) => {
    setEditingId(id)
    setEditName(name)
    setContextMenu(null)
  }

  const handleConfirmRename = () => {
    if (editName.trim()) {
      onRenameGroup(editingId, editName.trim())
    }
    setEditingId(null)
    setEditName('')
  }

  const handleContextMenu = (e, groupId) => {
    e.preventDefault()
    setContextMenu({ groupId, x: e.clientX, y: e.clientY })
  }

  // Close context menu on outside click
  React.useEffect(() => {
    if (!contextMenu) return
    const handler = () => setContextMenu(null)
    document.addEventListener('click', handler)
    return () => document.removeEventListener('click', handler)
  }, [contextMenu])

  const groupEntries = Object.entries(groups)
  const totalStocks = groupEntries.reduce((sum, [, g]) => sum + g.stocks.length, 0)

  return (
    <>
      {/* Toggle button — always visible */}
      <button
        className={`sidebar-toggle ${isOpen ? 'open' : ''}`}
        onClick={onToggle}
        title={isOpen ? '收起自选' : '展开自选'}
      >
        {isOpen ? '◂' : '▸'}
        {!isOpen && <span className="sidebar-toggle-label">自选</span>}
      </button>

      <aside className={`sidebar ${isOpen ? 'open' : ''}`}>
        <div className="sidebar-header">
          <span className="sidebar-title">⭐ 自选股票</span>
          <button
            className="sidebar-add-group-btn"
            onClick={() => setShowAddForm(v => !v)}
            title="新建分组"
          >
            +
          </button>
        </div>

        {showAddForm && (
          <div className="add-group-form">
            <input
              type="text"
              value={newGroupName}
              onChange={e => setNewGroupName(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleAddGroup()}
              placeholder="分组名称..."
              autoFocus
            />
            <button className="add-group-confirm" onClick={handleAddGroup}>确定</button>
          </div>
        )}

        <div className="sidebar-groups">
          {groupEntries.length === 0 ? (
            <div className="sidebar-empty">暂无自选分组</div>
          ) : (
            groupEntries.map(([id, group]) => (
              <div key={id} className="sidebar-group">
                <div
                  className="sidebar-group-header"
                  onClick={() => toggleGroup(id)}
                  onContextMenu={e => handleContextMenu(e, id)}
                >
                  <span className="sidebar-group-arrow">
                    {expandedGroups[id] ? '▾' : '▸'}
                  </span>
                  {editingId === id ? (
                    <input
                      className="sidebar-group-rename"
                      value={editName}
                      onChange={e => setEditName(e.target.value)}
                      onKeyDown={e => {
                        if (e.key === 'Enter') handleConfirmRename()
                        if (e.key === 'Escape') { setEditingId(null); setEditName('') }
                      }}
                      onBlur={handleConfirmRename}
                      onClick={e => e.stopPropagation()}
                      autoFocus
                    />
                  ) : (
                    <span className="sidebar-group-name">{group.name}</span>
                  )}
                  <span className="sidebar-group-count">{group.stocks.length}</span>
                </div>

                {expandedGroups[id] && (
                  <div className="sidebar-stocks">
                    {group.stocks.length === 0 ? (
                      <div className="sidebar-stocks-empty">
                        双击卡片添加至此
                      </div>
                    ) : (
                      group.stocks.map(stock => (
                        <div
                          key={stock.code}
                          className="sidebar-stock-item"
                          onClick={() => onSelectStock(stock)}
                        >
                          <span className="sidebar-stock-name">{stock.name}</span>
                          <span className="sidebar-stock-code">{stock.code}</span>
                          <button
                            className="sidebar-stock-remove"
                            onClick={e => {
                              e.stopPropagation()
                              onRemoveStock(id, stock.code)
                            }}
                            title="移除"
                          >
                            ✕
                          </button>
                        </div>
                      ))
                    )}
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        <div className="sidebar-footer">
          {totalStocks} 只自选
        </div>

        {/* Context menu */}
        {contextMenu && (
          <div
            className="sidebar-context-menu"
            style={{ top: contextMenu.y, left: contextMenu.x }}
          >
            <button onClick={() => handleStartRename(contextMenu.groupId, groups[contextMenu.groupId].name)}>
              ✏️ 重命名
            </button>
            {contextMenu.groupId !== 'default' && (
              <button
                className="ctx-delete"
                onClick={() => { onRemoveGroup(contextMenu.groupId); setContextMenu(null) }}
              >
                🗑️ 删除分组
              </button>
            )}
          </div>
        )}
      </aside>
    </>
  )
}

export default Sidebar
