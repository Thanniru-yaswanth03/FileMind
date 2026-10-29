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
  Sparkles,
  ArrowRight,
} from 'lucide-react';

interface CommandItem {
  id: string;
  title: string;
  category: 'Assistant' | 'Search' | 'Actions' | 'Navigation' | 'Preferences';
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
    setSearchQuery,
    sendCopilotMessage,
  } = useApp();

  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const navigateTo = (screen: NavScreen, label: string) => {
    setActiveScreen(screen);
    setIsCommandPaletteOpen(false);
    showToast(`Navigated to ${label}`);
  };

  const baseCommands: CommandItem[] = [
    // Primary Actions
    {
      id: 'act-scan',
      title: 'Trigger Volume Scan',
      category: 'Actions',
      icon: <Play size={14} style={{ color: 'var(--accent-base)' }} />,
      run: () => {
        startScan();
        setIsCommandPaletteOpen(false);
      },
    },
    {
      id: 'act-planner',
      title: 'Prepare Cleanup Plan',
      category: 'Actions',
      icon: <ListChecks size={14} style={{ color: 'var(--accent-base)' }} />,
      run: () => navigateTo('cleanup-planner', 'Cleanup Planner'),
    },
    {
      id: 'act-duplicates',
      title: 'Inspect & Review Duplicates',
      category: 'Actions',
      icon: <Copy size={14} style={{ color: 'var(--warning-base)' }} />,
      run: () => navigateTo('duplicates', 'Duplicate Finder'),
    },
    {
      id: 'act-activity',
      title: 'Review Recent Operations & Quarantine Vault',
      category: 'Actions',
      icon: <RotateCcw size={14} />,
      run: () => navigateTo('activity-recovery', 'Activity & Recovery'),
    },
    {
      id: 'act-inspector',
      title: 'Toggle Property Inspector',
      category: 'Actions',
      icon: <PanelRight size={14} />,
      run: () => {
        toggleInspector();
        setIsCommandPaletteOpen(false);
      },
    },

    // Navigation
    { id: 'nav-overview', title: 'Go to Overview', category: 'Navigation', icon: <LayoutDashboard size={14} />, run: () => navigateTo('overview', 'Overview') },
    { id: 'nav-storage', title: 'Go to Storage Explorer', category: 'Navigation', icon: <FolderTree size={14} />, run: () => navigateTo('storage-explorer', 'Storage Explorer') },
    { id: 'nav-search', title: 'Go to File Search', category: 'Navigation', icon: <Search size={14} />, run: () => navigateTo('search', 'File Search') },
    { id: 'nav-large', title: 'Go to Large Files', category: 'Navigation', icon: <FileSpreadsheet size={14} />, run: () => navigateTo('large-files', 'Large Files') },
    { id: 'nav-safety', title: 'Go to Safety Center', category: 'Navigation', icon: <ShieldCheck size={14} />, run: () => navigateTo('safety-center', 'Safety Center') },
    { id: 'nav-dev', title: 'Go to Developer Cleanup', category: 'Navigation', icon: <Code2 size={14} />, run: () => navigateTo('developer-cleanup', 'Developer Cleanup') },
    { id: 'nav-auto', title: 'Go to Automations', category: 'Navigation', icon: <Clock size={14} />, run: () => navigateTo('automations', 'Automations') },
    { id: 'nav-jarvis', title: 'Go to JARVIS Copilot', category: 'Navigation', icon: <Bot size={14} />, run: () => navigateTo('jarvis', 'JARVIS Copilot') },
    { id: 'nav-settings', title: 'Go to Settings', category: 'Navigation', icon: <Settings size={14} />, run: () => navigateTo('settings', 'Settings') },

    // Preferences
    { id: 'act-theme', title: 'Toggle Light / Dark Mode', category: 'Preferences', icon: <Sun size={14} />, run: () => { toggleTheme(); setIsCommandPaletteOpen(false); } },
  ];

  // Dynamic context commands based on query input
  const dynamicCommands: CommandItem[] = [];
  const cleanQ = query.trim();

  if (cleanQ.length > 0) {
    const promptText = cleanQ.startsWith('?') || cleanQ.startsWith('>') ? cleanQ.slice(1).trim() : cleanQ;

    // 1. Ask Copilot action
    dynamicCommands.push({
      id: 'copilot-ask',
      title: `Ask JARVIS: "${promptText}"`,
      category: 'Assistant',
      icon: <Sparkles size={14} style={{ color: 'var(--accent-base)' }} />,
      run: () => {
        sendCopilotMessage(promptText);
        navigateTo('jarvis', 'JARVIS Copilot');
      },
    });

    // 2. Search files action
    dynamicCommands.push({
      id: 'search-query',
      title: `Search local index for: "${promptText}"`,
      category: 'Search',
      icon: <Search size={14} style={{ color: 'var(--text-secondary)' }} />,
      run: () => {
        setSearchQuery(promptText);
        navigateTo('search', 'File Search');
      },
    });
  }

  const allCommands = [...dynamicCommands, ...baseCommands];

  const filtered = allCommands.filter((c) =>
    c.title.toLowerCase().includes(query.toLowerCase()) ||
    c.category.toLowerCase().includes(query.toLowerCase())
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
          maxWidth: '540px',
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
            padding: '12px 14px',
            borderBottom: '1px solid var(--border-subtle)',
          }}
        >
          <Search size={15} style={{ color: 'var(--accent-base)' }} />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            placeholder="Type a command, file name, or ask JARVIS a storage question... (Arrows to navigate, Enter to run)"
            style={{
              flex: 1,
              border: 'none',
              backgroundColor: 'transparent',
              fontSize: 'var(--text-body)',
              color: 'var(--text-primary)',
              padding: 0,
              boxShadow: 'none',
              outline: 'none',
            }}
          />
          {query.trim() && (
            <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
              Enter ↵
            </span>
          )}
        </div>

        {/* Command Items List */}
        <div style={{ maxHeight: '360px', overflowY: 'auto', padding: '6px' }}>
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
                    padding: '8px 10px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: isSelected ? 'var(--accent-subtle)' : 'transparent',
                    color: isSelected ? 'var(--accent-base)' : 'var(--text-primary)',
                    cursor: 'pointer',
                    fontSize: 'var(--text-caption)',
                    gap: '10px',
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0, flex: 1 }}>
                    <span style={{ color: isSelected ? 'var(--accent-base)' : 'var(--text-secondary)', flexShrink: 0 }}>
                      {cmd.icon}
                    </span>
                    <span className="truncate" style={{ fontWeight: isSelected ? 600 : 400 }}>
                      {cmd.title}
                    </span>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
                    <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                      {cmd.category}
                    </span>
                    {isSelected && <ArrowRight size={11} style={{ color: 'var(--accent-base)' }} />}
                  </div>
                </div>
              );
            })
          ) : (
            <div style={{ padding: '20px', textAlign: 'center', color: 'var(--text-muted)', fontSize: 'var(--text-caption)' }}>
              No matching commands found. Type a question to ask JARVIS directly.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
