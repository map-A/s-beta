import React, { useState, useEffect, useRef } from 'react'
import './GroupPickerModal.css'

function GroupPickerModal({ company, groups, onConfirm, onConfirmNewGroup, onClose }) {
  const [selectedId, setSelectedId] = useState('default')
  const [showNewForm, setShowNewForm] = useState(false)
  const [newGroupName, setNewGroupName] = useState('')
  const inputRef = useRef(null)

  useEffect(() => {
    if (showNewForm && inputRef.current) {
      inputRef.current.focus()
    }
  }, [showNewForm])

  const handleConfirm = () => {
    if (showNewForm) {
      const name = newGroupName.trim()
      if (!name) return
      onConfirmNewGroup(name)
    } else {
      onConfirm(selectedId)
    }
  }

  const handleKeyDown = (e) => {
    if (e.key === 'Enter') handleConfirm()
    if (e.key === 'Escape') onClose()
  }

  const groupEntries = Object.entries(groups)

  return (
    <div className="gpm-overlay" onClick={onClose}>
      <div className="gpm-modal" onClick={e => e.stopPropagation()}>
        <div className="gpm-header">
          <h3>选择自选分组</h3>
          <span className="gpm-company">{company.name} ({company.code})</span>
        </div>

        <div className="gpm-body">
          {!showNewForm ? (
            <>
              <div className="gpm-group-list">
                {groupEntries.map(([id, group]) => (
                  <div
                    key={id}
                    className={`gpm-group-item ${selectedId === id ? 'selected' : ''}`}
                    onClick={() => setSelectedId(id)}
                  >
                    <span className="gpm-radio">{selectedId === id ? '◉' : '○'}</span>
                    <span className="gpm-group-name">{group.name}</span>
                    <span className="gpm-group-count">{group.stocks.length} 只</span>
                  </div>
                ))}
              </div>
              <button
                className="gpm-new-group-btn"
                onClick={() => setShowNewForm(true)}
              >
                + 新建分组
              </button>
            </>
          ) : (
            <div className="gpm-new-form">
              <input
                ref={inputRef}
                type="text"
                className="gpm-new-input"
                value={newGroupName}
                onChange={e => setNewGroupName(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="输入分组名称..."
              />
              <button
                className="gpm-back-btn"
                onClick={() => { setShowNewForm(false); setNewGroupName('') }}
              >
                ← 返回选择
              </button>
            </div>
          )}
        </div>

        <div className="gpm-footer">
          <button className="gpm-cancel" onClick={onClose}>取消</button>
          <button className="gpm-confirm" onClick={handleConfirm}>
            {showNewForm ? '创建并加入' : '确认加入'}
          </button>
        </div>
      </div>
    </div>
  )
}

export default GroupPickerModal
