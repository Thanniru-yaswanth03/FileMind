import React, { useState, useEffect, useRef } from 'react';
import { useApp } from '@/context/AppContext';
import { NavScreen } from '@/types';
import {
  Search,
  LayoutDashboard,
  FolderTree,
  FileSpreadsheet,
  Copy,
  ListChecks,
  ShieldCheck,
  Code2,
  Clock,
  Bot,
  RotateCcw,
  Settings,
  Sun,
  Play,
  PanelRight,
} from 'lucide-react';

interface CommandItem {
  id: string;
  title: string;
  category: 'Navigation' | 'Actions' | 'Preferences';
  icon: React.ReactNode;
  shortcut?: string;
  run: () => void;
}

export const CommandPalette: React.FC = () => {
  const {
    isCommandPaletteOpen,
    setIsCommandPaletteOpen,
    setActiveScreen,
    toggleTheme,
    startScan,
    toggleInspector,
    showToast,
  } = useApp();

  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const navigateTo = (screen: NavScreen, label: string) => {
    setActiveScreen(screen);
    setIsCommandPaletteOpen(false);
    showToast(`Navigated to ${label}`);
  };

  const commands: CommandItem[] = [
    { id: 'nav-overview', title: 'Go to Overview', category: 'Navigation', icon: <LayoutDashboard size={14} />, run: () => navigateTo('overview', 'Overview') },
    { id: 'nav-storage', title: 'Go to Storage Explorer', category: 'Navigation', icon: <FolderTree size={14} />, run: () => navigateTo('storage-explorer', 'Storage Explorer') },
    { id: 'nav-large', title: 'Go to Large Files', category: 'Navigation', icon: <FileSpreadsheet size={14} />, run: () => navigateTo('large-files', 'Large Files') },
    { id: 'nav-duplicates', title: 'Go to Duplicate Finder', category: 'Navigation', icon: <Copy size={14} />, run: () => navigateTo('duplicates', 'Duplicate Finder') },
    { id: 'nav-planner', title: 'Go to Cleanup Planner', category: 'Navigation', icon: <ListChecks size={14} />, run: () => navigateTo('cleanup-planner', 'Cleanup Planner') },
    { id: 'nav-safety', title: 'Go to Safety Center', category: 'Navigation', icon: <ShieldCheck size={14} />, run: () => navigateTo('safety-center', 'Safety Center') },
    { id: 'nav-dev', title: 'Go to Developer Cleanup', category: 'Navigation', icon: <Code2 size={14} />, run: () => navigateTo('developer-cleanup', 'Developer Cleanup') },
    { id: 'nav-auto', title: 'Go to Automations', category: 'Navigation', icon: <Clock size={14} />, run: () => navigateTo('automations', 'Automations') },
    { id: 'nav-jarvis', title: 'Go to JARVIS Copilot', category: 'Navigation', icon: <Bot size={14} />, run: () => navigateTo('jarvis', 'JARVIS Copilot') },
    { id: 'nav-activity', title: 'Go to Activity & Recovery', category: 'Navigation', icon: <RotateCcw size={14} />, run: () => navigateTo('activity-recovery', 'Activity & Recovery') },
    { id: 'nav-settings', title: 'Go to Settings', category: 'Navigation', icon: <Settings size={14} />, run: () => navigateTo('settings', 'Settings') },

    { id: 'act-scan', title: 'Trigger Volume Scan', category: 'Actions', icon: <Play size={14} />, run: () => { startScan(); setIsCommandPaletteOpen(false); } },
    { id: 'act-inspector', title: 'Toggle Property Inspector', category: 'Actions', icon: <PanelRight size={14} />, run: () => { toggleInspector(); setIsCommandPaletteOpen(false); } },
    { id: 'act-theme', title: 'Toggle Light / Dark Mode', category: 'Preferences', icon: <Sun size={14} />, run: () => { toggleTheme(); setIsCommandPaletteOpen(false); } },
  ];

  const filtered = commands.filter((c) =>
    c.title.toLowerCase().includes(query.toLowerCase()) || c.category.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    if (isCommandPaletteOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isCommandPaletteOpen]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setIsCommandPaletteOpen(!isCommandPaletteOpen);
      }
      if (!isCommandPaletteOpen) return;

      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % (filtered.length || 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + filtered.length) % (filtered.length || 1));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (filtered[selectedIndex]) {
          filtered[selectedIndex].run();
        }
      } else if (e.key === 'Escape') {
        setIsCommandPaletteOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isCommandPaletteOpen, filtered, selectedIndex, setIsCommandPaletteOpen]);

  if (!isCommandPaletteOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 10002,
        backgroundColor: 'rgba(0, 0, 0, 0.45)',
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'center',
        paddingTop: '80px',
      }}
      onClick={() => setIsCommandPaletteOpen(false)}
    >
      <div
        className="animate-fade-in"
        role="dialog"
        aria-modal="true"
        aria-label="Command Palette"
        style={{
          width: '100%',
          maxWidth: '520px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-strong)',
          borderRadius: 'var(--radius-lg)',
          boxShadow: 'var(--shadow-modal)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input Bar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            padding: '10px 14px',
            borderBottom: '1px solid var(--border-subtle)',
          }}
        >
          <Search size={15} style={{ color: 'var(--text-muted)' }} />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            placeholder="Type a screen name or action... (Arrows to navigate, Enter to run)"
            style={{
              flex: 1,
              border: 'none',
              backgroundColor: 'transparent',
              fontSize: 'var(--text-body)',
              color: 'var(--text-primary)',
              padding: 0,
              boxShadow: 'none',
            }}
          />
        </div>

        {/* Command Items List */}
        <div style={{ maxHeight: '340px', overflowY: 'auto', padding: '6px' }}>
          {filtered.length > 0 ? (
            filtered.map((cmd, idx) => {
              const isSelected = idx === selectedIndex;
              return (
                <div
                  key={cmd.id}
                  onClick={() => cmd.run()}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '7px 10px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: isSelected ? 'var(--accent-subtle)' : 'transparent',
                    color: isSelected ? 'var(--accent-base)' : 'var(--text-primary)',
                    cursor: 'pointer',
                    fontSize: 'var(--text-caption)',
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span style={{ color: isSelected ? 'var(--accent-base)' : 'var(--text-secondary)' }}>
                      {cmd.icon}
                    </span>
                    <span style={{ fontWeight: isSelected ? 600 : 400 }}>{cmd.title}</span>
                  </div>
                  <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                    {cmd.category}
                  </span>
                </div>
              );
            })
          ) : (
            <div style={{ padding: '16px', textAlign: 'center', color: 'var(--text-muted)', fontSize: 'var(--text-caption)' }}>
              No matching commands found.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
