import React, { useEffect, useRef } from 'react';
import { useApp } from '@/context/AppContext';
import { Bell, AlertCircle, Info, CheckCircle2, X } from 'lucide-react';

export const NotificationCenter: React.FC = () => {
  const {
    notifications,
    isNotificationOpen,
    setIsNotificationOpen,
    markAllNotificationsRead,
    setActiveScreen,
  } = useApp();

  const popoverRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
        setIsNotificationOpen(false);
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setIsNotificationOpen(false);
      }
    };
    if (isNotificationOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isNotificationOpen, setIsNotificationOpen]);

  if (!isNotificationOpen) return null;

  return (
    <div
      ref={popoverRef}
      className="animate-fade-in"
      style={{
        position: 'fixed',
        top: '44px',
        right: '54px',
        width: '320px',
        zIndex: 9999,
        backgroundColor: 'var(--bg-surface-0)',
        border: '1px solid var(--border-strong)',
        borderRadius: 'var(--radius-md)',
        boxShadow: 'var(--shadow-dropdown)',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-1)',
          borderBottom: '1px solid var(--border-subtle)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <Bell size={13} style={{ color: 'var(--accent-base)' }} />
          <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
            System Notifications
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <button
            type="button"
            onClick={markAllNotificationsRead}
            title="Mark all as read"
            style={{ fontSize: 'var(--text-micro)', color: 'var(--accent-base)', cursor: 'pointer' }}
          >
            Mark read
          </button>
          <button
            type="button"
            onClick={() => setIsNotificationOpen(false)}
            style={{ color: 'var(--text-muted)', cursor: 'pointer' }}
          >
            <X size={13} />
          </button>
        </div>
      </div>

      {/* List */}
      <div style={{ maxHeight: '280px', overflowY: 'auto', padding: '6px', display: 'flex', flexDirection: 'column', gap: '6px' }}>
        {notifications.map((notif) => {
          const getIcon = () => {
            switch (notif.type) {
              case 'warning':
                return <AlertCircle size={14} style={{ color: 'var(--warning-base)', flexShrink: 0 }} />;
              case 'danger':
                return <AlertCircle size={14} style={{ color: 'var(--danger-base)', flexShrink: 0 }} />;
              case 'success':
                return <CheckCircle2 size={14} style={{ color: 'var(--safe-base)', flexShrink: 0 }} />;
              default:
                return <Info size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />;
            }
          };

          return (
            <div
              key={notif.id}
              onClick={() => {
                if (notif.actionScreen) {
                  setActiveScreen(notif.actionScreen);
                  setIsNotificationOpen(false);
                }
              }}
              style={{
                padding: '8px 10px',
                borderRadius: 'var(--radius-sm)',
                backgroundColor: notif.read ? 'transparent' : 'var(--bg-surface-1)',
                border: '1px solid var(--border-subtle)',
                display: 'flex',
                gap: '8px',
                cursor: notif.actionScreen ? 'pointer' : 'default',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)')}
              onMouseLeave={(e) =>
                (e.currentTarget.style.backgroundColor = notif.read ? 'transparent' : 'var(--bg-surface-1)')
              }
            >
              {getIcon()}
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
                    {notif.title}
                  </span>
                  <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                    {notif.timestamp}
                  </span>
                </div>
                <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)', marginTop: '2px', lineHeight: 1.3 }}>
                  {notif.message}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
