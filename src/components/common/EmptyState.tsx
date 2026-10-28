import React from 'react';
import { Button } from './Button';

interface EmptyStateProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
  secondaryActionLabel?: string;
  onSecondaryAction?: () => void;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  icon,
  title,
  description,
  actionLabel,
  onAction,
  secondaryActionLabel,
  onSecondaryAction,
}) => {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '48px 24px',
        textAlign: 'center',
        height: '100%',
        color: 'var(--text-secondary)',
      }}
    >
      <div
        style={{
          width: '44px',
          height: '44px',
          borderRadius: 'var(--radius-lg)',
          backgroundColor: 'var(--bg-surface-1)',
          border: '1px solid var(--border-subtle)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'var(--accent-base)',
          marginBottom: '12px',
        }}
      >
        {icon}
      </div>

      <div
        style={{
          fontSize: 'var(--text-subhead)',
          fontWeight: 600,
          color: 'var(--text-primary)',
          marginBottom: '6px',
        }}
      >
        {title}
      </div>

      <p
        style={{
          fontSize: 'var(--text-caption)',
          color: 'var(--text-muted)',
          maxWidth: '380px',
          lineHeight: 1.5,
          marginBottom: '16px',
        }}
      >
        {description}
      </p>

      {(actionLabel || secondaryActionLabel) && (
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          {actionLabel && (
            <Button variant="primary" onClick={onAction}>
              {actionLabel}
            </Button>
          )}
          {secondaryActionLabel && (
            <Button variant="secondary" onClick={onSecondaryAction}>
              {secondaryActionLabel}
            </Button>
          )}
        </div>
      )}
    </div>
  );
};

