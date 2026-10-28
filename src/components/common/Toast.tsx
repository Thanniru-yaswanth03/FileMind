import React from 'react';
import { useApp } from '../../context/AppContext';
import { Info } from 'lucide-react';

export const Toast: React.FC = () => {
  const { toastMessage } = useApp();

  if (!toastMessage) return null;

  return (
    <div
      className="animate-fade-in"
      style={{
        position: 'fixed',
        bottom: '38px',
        right: '24px',
        zIndex: 10000,
        backgroundColor: 'var(--text-primary)',
        color: 'var(--text-inverse)',
        padding: '8px 14px',
        borderRadius: 'var(--radius-md)',
        fontSize: 'var(--text-caption)',
        fontWeight: 500,
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        boxShadow: 'var(--shadow-dropdown)',
        maxWidth: '380px',
      }}
    >
      <Info size={14} style={{ color: 'var(--accent-contrast)', flexShrink: 0 }} />
      <span>{toastMessage}</span>
    </div>
  );
};

