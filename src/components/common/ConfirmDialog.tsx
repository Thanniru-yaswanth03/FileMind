import React, { useEffect } from 'react';
import { Button } from './Button';
import { AlertTriangle, X } from 'lucide-react';

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  impactNote?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  isDestructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  isOpen,
  title,
  message,
  impactNote,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  isDestructive = false,
  onConfirm,
  onCancel,
}) => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onCancel();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown);
    }
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onCancel]);

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 10001,
        backgroundColor: 'rgba(0, 0, 0, 0.45)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '16px',
      }}
      onClick={onCancel}
    >
      <div
        className="animate-fade-in"
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        style={{
          width: '100%',
          maxWidth: '440px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-strong)',
          borderRadius: 'var(--radius-lg)',
          boxShadow: 'var(--shadow-modal)',
          padding: '18px 20px',
          display: 'flex',
          flexDirection: 'column',
          gap: '12px',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {isDestructive && <AlertTriangle size={18} style={{ color: 'var(--danger-base)' }} />}
            <span id="dialog-title" style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
              {title}
            </span>
          </div>
          <button
            type="button"
            onClick={onCancel}
            style={{ color: 'var(--text-muted)', padding: '2px', cursor: 'pointer' }}
          >
            <X size={16} />
          </button>
        </div>

        <p style={{ fontSize: 'var(--text-body)', color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          {message}
        </p>

        {impactNote && (
          <div
            style={{
              padding: '8px 10px',
              backgroundColor: isDestructive ? 'var(--danger-subtle)' : 'var(--bg-surface-1)',
              border: `1px solid ${isDestructive ? 'var(--danger-border)' : 'var(--border-subtle)'}`,
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-caption)',
              color: isDestructive ? 'var(--danger-base)' : 'var(--text-secondary)',
            }}
          >
            <strong>Note: </strong> {impactNote}
          </div>
        )}

        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px', marginTop: '6px' }}>
          <Button variant="secondary" onClick={onCancel}>
            {cancelLabel}
          </Button>
          <Button variant={isDestructive ? 'danger' : 'primary'} onClick={onConfirm}>
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>
  );
};

