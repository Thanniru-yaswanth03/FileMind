import React from 'react';
import { useApp } from '@/context/AppContext';
import type { NavScreen } from '@/types';
import {
  LayoutDashboard,
  FolderTree,
  Search,
  FileSpreadsheet,
  Copy,
  ListChecks,
  ShieldCheck,
  Code2,
  Clock,
  Bot,
  RotateCcw,
  Settings,
  ChevronLeft,
  ChevronRight,
  HardDrive,
} from 'lucide-react';

interface NavItemDef {
  id: NavScreen;
  label: string;
  icon: React.ReactNode;
  badge?: string | number;
  badgeVariant?: 'accent' | 'warning' | 'neutral';
}

interface NavGroup {
  category: string;
  items: NavItemDef[];
}

export const Sidebar: React.FC = () => {
  const { activeScreen, setActiveScreen, isSidebarCollapsed, toggleSidebar, cleanupPlan, realDuplicateGroups } = useApp();

  const navGroups: NavGroup[] = [
    {
      category: 'EXPLORE',
      items: [
        { id: 'overview', label: 'Overview', icon: <LayoutDashboard size={15} /> },
        { id: 'storage-explorer', label: 'Storage Explorer', icon: <FolderTree size={15} /> },
        { id: 'search', label: 'File Search', icon: <Search size={15} /> },
        { id: 'large-files', label: 'Large Files', icon: <FileSpreadsheet size={15} /> },
        {
          id: 'duplicates',
          label: 'Duplicate Finder',
          icon: <Copy size={15} />,
          badge: realDuplicateGroups.length,
          badgeVariant: 'warning',
        },
      ],
    },
    {
      category: 'RECLAIM',
      items: [
        {
          id: 'cleanup-planner',
          label: 'Cleanup Planner',
          icon: <ListChecks size={15} />,
          badge: cleanupPlan.items.length,
          badgeVariant: 'accent',
        },
        { id: 'developer-cleanup', label: 'Developer Cleanup', icon: <Code2 size={15} /> },
        { id: 'automations', label: 'Automations', icon: <Clock size={15} /> },
      ],
    },
    {
      category: 'SYSTEM',
      items: [
        { id: 'safety-center', label: 'Safety Center', icon: <ShieldCheck size={15} /> },
        { id: 'jarvis', label: 'JARVIS Copilot', icon: <Bot size={15} /> },
        { id: 'activity-recovery', label: 'Activity & Recovery', icon: <RotateCcw size={15} /> },
        { id: 'settings', label: 'Settings', icon: <Settings size={15} /> },
      ],
    },
  ];

  return (
    <aside
      style={{
        width: isSidebarCollapsed ? '48px' : '204px',
        height: '100%',
        backgroundColor: 'var(--bg-surface-1)',
        borderRight: '1px solid var(--border-subtle)',
        display: 'flex',
        flexDirection: 'column',
        flexShrink: 0,
        transition: 'width 150ms cubic-bezier(0.16, 1, 0.3, 1)',
        overflow: 'hidden',
      }}
    >
      {/* Brand Header */}
      <div
        style={{
          height: '40px',
          padding: '0 12px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: isSidebarCollapsed ? 'center' : 'space-between',
          borderBottom: '1px solid var(--border-subtle)',
        }}
      >
        {!isSidebarCollapsed ? (
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '20px',
                height: '20px',
                borderRadius: 'var(--radius-sm)',
                backgroundColor: 'var(--accent-base)',
                color: 'var(--accent-contrast)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <HardDrive size={12} />
            </div>
            <span
              style={{
                fontWeight: 600,
                fontSize: 'var(--text-ui)',
                letterSpacing: '-0.02em',
                color: 'var(--text-primary)',
              }}
            >
              FileMind
            </span>
          </div>
        ) : (
          <div
            style={{
              width: '20px',
              height: '20px',
              borderRadius: 'var(--radius-sm)',
              backgroundColor: 'var(--accent-base)',
              color: 'var(--accent-contrast)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <HardDrive size={12} />
          </div>
        )}

        <button
          type="button"
          onClick={toggleSidebar}
          title={isSidebarCollapsed ? 'Expand Sidebar (Ctrl+B)' : 'Collapse Sidebar (Ctrl+B)'}
          aria-label={isSidebarCollapsed ? 'Expand Sidebar' : 'Collapse Sidebar'}
          style={{
            padding: '4px',
            color: 'var(--text-secondary)',
            cursor: 'pointer',
            borderRadius: 'var(--radius-sm)',
            display: isSidebarCollapsed ? 'none' : 'flex',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
        >
          <ChevronLeft size={14} />
        </button>
      </div>

      {/* Grouped Navigation List */}
      <nav
        style={{
          flex: 1,
          padding: '6px 4px',
          display: 'flex',
          flexDirection: 'column',
          gap: '6px',
          overflowY: 'auto',
          overflowX: 'hidden',
        }}
      >
        {navGroups.map((group: NavGroup, groupIndex: number) => (
          <div key={group.category} style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
            {/* Category Header or Divider */}
            {!isSidebarCollapsed ? (
              <div
                style={{
                  fontSize: 'var(--text-micro)',
                  fontWeight: 600,
                  letterSpacing: '0.05em',
                  color: 'var(--text-muted)',
                  padding: groupIndex === 0 ? '4px 8px 2px' : '8px 8px 2px',
                  textTransform: 'uppercase',
                }}
              >
                {group.category}
              </div>
            ) : (
              groupIndex > 0 && (
                <div
                  style={{
                    height: '1px',
                    backgroundColor: 'var(--border-subtle)',
                    margin: '4px 8px',
                  }}
                />
              )
            )}

            {group.items.map((item: NavItemDef) => {
              const isActive = activeScreen === item.id;

              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => setActiveScreen(item.id)}
                  title={isSidebarCollapsed ? item.label : undefined}
                  aria-label={item.label}
                  aria-current={isActive ? 'page' : undefined}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: isSidebarCollapsed ? 'center' : 'flex-start',
                    gap: '8px',
                    padding: isSidebarCollapsed ? '7px 0' : '6px 8px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: isActive ? 'var(--accent-subtle)' : 'transparent',
                    color: isActive ? 'var(--accent-base)' : 'var(--text-secondary)',
                    fontWeight: isActive ? 600 : 400,
                    fontSize: 'var(--text-body)',
                    borderLeft: isActive ? '2px solid var(--accent-base)' : '2px solid transparent',
                    cursor: 'pointer',
                    textAlign: 'left',
                    width: '100%',
                  }}
                  onMouseEnter={(e) => {
                    if (!isActive) e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)';
                  }}
                  onMouseLeave={(e) => {
                    if (!isActive) e.currentTarget.style.backgroundColor = 'transparent';
                  }}
                >
                  <span style={{ color: isActive ? 'var(--accent-base)' : 'inherit', flexShrink: 0 }}>
                    {item.icon}
                  </span>

                  {!isSidebarCollapsed && (
                    <>
                      <span className="truncate" style={{ flex: 1 }}>
                        {item.label}
                      </span>

                      {item.badge !== undefined && (
                        <span
                          className="tabular-nums"
                          style={{
                            padding: '1px 5px',
                            fontSize: 'var(--text-micro)',
                            borderRadius: 'var(--radius-sm)',
                            fontWeight: 600,
                            backgroundColor:
                              item.badgeVariant === 'accent'
                                ? 'var(--accent-base)'
                                : item.badgeVariant === 'warning'
                                  ? 'var(--warning-base)'
                                  : 'var(--bg-surface-2)',
                            color:
                              item.badgeVariant === 'accent' || item.badgeVariant === 'warning'
                                ? 'var(--accent-contrast)'
                                : 'var(--text-secondary)',
                          }}
                        >
                          {item.badge}
                        </span>
                      )}
                    </>
                  )}
                </button>
              );
            })}
          </div>
        ))}
      </nav>

      {/* Collapsed Expand Toggle */}
      {isSidebarCollapsed && (
        <div style={{ padding: '4px', borderTop: '1px solid var(--border-subtle)', display: 'flex', justifyContent: 'center' }}>
          <button
            type="button"
            onClick={toggleSidebar}
            title="Expand Sidebar (Ctrl+B)"
            style={{ padding: '6px', color: 'var(--text-secondary)' }}
          >
            <ChevronRight size={14} />
          </button>
        </div>
      )}
    </aside>
  );
};
