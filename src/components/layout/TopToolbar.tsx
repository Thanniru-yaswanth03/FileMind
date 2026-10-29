import React from 'react';
import { useApp } from '../../context/AppContext';
import {
  Search,
  HardDrive,
  Play,
  Square,
  Sun,
  Moon,
  Bell,
  Command,
  PanelRight,
  PanelRightClose,
} from 'lucide-react';

export const TopToolbar: React.FC = () => {
  const {
    scanRoots,
    selectedRootId,
    setSelectedRootId,
    currentRoot,
    isScanning,
    startScan,
    cancelScan,
    theme,
    toggleTheme,
    notifications,
    isNotificationOpen,
    setIsNotificationOpen,
    setIsCommandPaletteOpen,
    searchQuery,
    setSearchQuery,
    isInspectorOpen,
    toggleInspector,
    setActiveScreen,
  } = useApp();

  const unreadNotifs = notifications.filter((n) => !n.read).length;

  return (
    <header
      style={{
        height: '40px',
        backgroundColor: 'var(--bg-surface-0)',
        borderBottom: '1px solid var(--border-subtle)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 12px',
        gap: '12px',
        flexShrink: 0,
      }}
    >
      {/* Left: Drive Selector & Scan Button */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <HardDrive size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
          <select
            value={selectedRootId}
            onChange={(e) => setSelectedRootId(e.target.value)}
            style={{
              padding: '3px 8px',
              fontSize: 'var(--text-caption)',
              fontWeight: 500,
              backgroundColor: 'var(--bg-surface-1)',
              borderColor: 'var(--border-subtle)',
              cursor: 'pointer',
            }}
          >
            {scanRoots.map((root) => (
              <option key={root.id} value={root.id}>
                {root.driveLetter} ({root.label.split(' ')[0]}) [{Math.round((root.usedBytes / root.totalBytes) * 100)}% Used]
              </option>
            ))}
          </select>
        </div>

        {/* Scan Action Button */}
        {isScanning ? (
          <button
            type="button"
            onClick={cancelScan}
            style={{
              padding: '3px 8px',
              fontSize: 'var(--text-caption)',
              backgroundColor: 'var(--danger-subtle)',
              color: 'var(--danger-base)',
              border: '1px solid var(--danger-border)',
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
            }}
          >
            <Square size={11} fill="currentColor" />
            <span>Cancel Scan</span>
          </button>
        ) : (
          <button
            type="button"
            onClick={() => startScan()}
            style={{
              padding: '3px 8px',
              fontSize: 'var(--text-caption)',
              backgroundColor: 'var(--accent-subtle)',
              color: 'var(--accent-base)',
              border: '1px solid var(--accent-border)',
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
            }}
          >
            <Play size={11} fill="currentColor" />
            <span>Scan {currentRoot.driveLetter}</span>
          </button>
        )}
      </div>

      {/* Center: Global Search Input */}
      <div style={{ display: 'flex', alignItems: 'center', flex: 1, maxWidth: '440px', position: 'relative' }}>
        <Search
          size={13}
          style={{
            position: 'absolute',
            left: '8px',
            color: 'var(--text-muted)',
            pointerEvents: 'none',
          }}
        />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              setActiveScreen('search');
            }
          }}
          placeholder="Filter files, caches, or extensions... (Enter for Search)"
          style={{
            width: '100%',
            padding: '4px 8px 4px 26px',
            fontSize: 'var(--text-caption)',
            backgroundColor: 'var(--bg-surface-1)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
          }}
        />
      </div>

      {/* Right: Utility Controls (Command Palette, Notifs, Theme, Inspector) */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
        {/* Command Palette Trigger */}
        <button
          type="button"
          onClick={() => setIsCommandPaletteOpen(true)}
          title="Command Palette (Ctrl+K)"
          style={{
            padding: '4px 7px',
            fontSize: 'var(--text-micro)',
            color: 'var(--text-secondary)',
            backgroundColor: 'var(--bg-surface-1)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-sm)',
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
          }}
        >
          <Command size={12} />
          <span>Ctrl+K</span>
        </button>

        {/* Notification Popover Toggle */}
        <button
          type="button"
          onClick={() => setIsNotificationOpen(!isNotificationOpen)}
          title="Notifications"
          aria-label={unreadNotifs > 0 ? `${unreadNotifs} unread notifications` : 'Notifications'}
          style={{
            padding: '4px 6px',
            color: unreadNotifs > 0 ? 'var(--warning-base)' : 'var(--text-secondary)',
            position: 'relative',
          }}
        >
          <Bell size={15} />
          {unreadNotifs > 0 && (
            <span
              style={{
                position: 'absolute',
                top: '2px',
                right: '2px',
                width: '6px',
                height: '6px',
                borderRadius: '50%',
                backgroundColor: 'var(--warning-base)',
              }}
            />
          )}
        </button>

        {/* Theme Toggle (Warm Ivory <-> Warm Charcoal) */}
        <button
          type="button"
          onClick={toggleTheme}
          title={theme === 'light' ? 'Switch to Warm Charcoal Dark Theme' : 'Switch to Warm Ivory Light Theme'}
          aria-label={theme === 'light' ? 'Switch to Warm Charcoal Dark Theme' : 'Switch to Warm Ivory Light Theme'}
          style={{ padding: '4px 6px', color: 'var(--text-secondary)' }}
        >
          {theme === 'light' ? <Moon size={15} /> : <Sun size={15} />}
        </button>

        {/* Property Inspector Drawer Toggle */}
        <button
          type="button"
          onClick={toggleInspector}
          title={isInspectorOpen ? 'Collapse Details Inspector' : 'Expand Details Inspector'}
          aria-label={isInspectorOpen ? 'Collapse Details Inspector' : 'Expand Details Inspector'}
          style={{
            padding: '4px 6px',
            color: isInspectorOpen ? 'var(--accent-base)' : 'var(--text-secondary)',
          }}
        >
          {isInspectorOpen ? <PanelRightClose size={15} /> : <PanelRight size={15} />}
        </button>
      </div>
    </header>
  );
};

