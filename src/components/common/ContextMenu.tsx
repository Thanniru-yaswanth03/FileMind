import React, { useEffect, useRef } from 'react';
import { useApp } from '../../context/AppContext';
import { copyToClipboard } from '../../utils/format';
import { Info, Copy, PlusCircle, ShieldAlert, FolderOpen, Trash2 } from 'lucide-react';

export const ContextMenu: React.FC = () => {
  const { contextMenu, closeContextMenu, setSelectedFile, setIsInspectorOpen, showToast, setActiveScreen } = useApp();
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        closeContextMenu();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') closeContextMenu();
    };

    if (contextMenu) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [contextMenu, closeContextMenu]);

  if (!contextMenu) return null;

  const { x, y, file } = contextMenu;

  // Boundary check
  const menuX = Math.min(x, window.innerWidth - 210);
  const menuY = Math.min(y, window.innerHeight - 240);

  return (
    <div
      ref={menuRef}
      className="animate-fade-in"
      style={{
        position: 'fixed',
        left: `${menuX}px`,
        top: `${menuY}px`,
        zIndex: 9999,
        backgroundColor: 'var(--bg-surface-0)',
        border: '1px solid var(--border-strong)',
        borderRadius: 'var(--radius-md)',
        boxShadow: 'var(--shadow-dropdown)',
        padding: '4px',
        width: '200px',
        display: 'flex',
        flexDirection: 'column',
        gap: '2px',
        fontSize: 'var(--text-caption)',
      }}
    >
      <div
        style={{
          padding: '4px 8px',
          borderBottom: '1px solid var(--border-subtle)',
          color: 'var(--text-muted)',
          fontSize: 'var(--text-micro)',
          fontWeight: 600,
          textTransform: 'uppercase',
          letterSpacing: '0.04em',
        }}
      >
        File Actions
      </div>

      <button
        type="button"
        onClick={() => {
          setSelectedFile(file);
          setIsInspectorOpen(true);
          closeContextMenu();
        }}
        style={{
          padding: '5px 8px',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--text-primary)',
          cursor: 'pointer',
          textAlign: 'left',
          width: '100%',
        }}
        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)')}
        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
      >
        <Info size={13} style={{ color: 'var(--accent-base)' }} />
        <span>Inspect Details</span>
      </button>

      <button
        type="button"
        onClick={async () => {
          await copyToClipboard(file.path);
          showToast('Full path copied to clipboard');
          closeContextMenu();
        }}
        style={{
          padding: '5px 8px',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--text-primary)',
          cursor: 'pointer',
          textAlign: 'left',
          width: '100%',
        }}
        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)')}
        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
      >
        <Copy size={13} style={{ color: 'var(--text-secondary)' }} />
        <span>Copy Full Path</span>
      </button>

      <button
        type="button"
        onClick={() => {
          showToast(`Staged "${file.name}" for cleanup review`);
          setActiveScreen('cleanup-planner');
          closeContextMenu();
        }}
        style={{
          padding: '5px 8px',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--text-primary)',
          cursor: 'pointer',
          textAlign: 'left',
          width: '100%',
        }}
        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)')}
        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
      >
        <PlusCircle size={13} style={{ color: 'var(--accent-base)' }} />
        <span>Add to Cleanup Plan</span>
      </button>

      <button
        type="button"
        onClick={() => {
          showToast(`Added rule: exclude path "${file.name}"`);
          closeContextMenu();
        }}
        style={{
          padding: '5px 8px',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--text-primary)',
          cursor: 'pointer',
          textAlign: 'left',
          width: '100%',
        }}
        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)')}
        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
      >
        <ShieldAlert size={13} style={{ color: 'var(--warning-base)' }} />
        <span>Exclude from Scans</span>
      </button>

      <div style={{ height: '1px', backgroundColor: 'var(--border-subtle)', margin: '2px 0' }} />

      <button
        type="button"
        disabled
        title="Requires native Tauri Win32 integration (Phase 4)"
        style={{
          padding: '5px 8px',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--text-muted)',
          cursor: 'not-allowed',
          textAlign: 'left',
          width: '100%',
          opacity: 0.6,
        }}
      >
        <FolderOpen size={13} />
        <span>Reveal in Explorer (Phase 4)</span>
      </button>

      <button
        type="button"
        disabled
        title="Direct permanent delete disabled by safety policy. Staged quarantine required."
        style={{
          padding: '5px 8px',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--danger-base)',
          cursor: 'not-allowed',
          textAlign: 'left',
          width: '100%',
          opacity: 0.6,
        }}
      >
        <Trash2 size={13} />
        <span>Permanent Delete (Blocked)</span>
      </button>
    </div>
  );
};

