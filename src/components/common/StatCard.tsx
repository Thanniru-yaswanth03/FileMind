import React from 'react';

interface StatCardProps {
  label: string;
  value: string | number;
  subValue?: string;
  badge?: React.ReactNode;
  icon?: React.ReactNode;
  hint?: string;
}

export const StatCard: React.FC<StatCardProps> = ({
  label,
  value,
  subValue,
  badge,
  icon,
  hint,
}) => {
  return (
    <div
      style={{
        backgroundColor: 'var(--bg-surface-0)',
        border: '1px solid var(--border-subtle)',
        borderRadius: 'var(--radius-md)',
        padding: '12px 14px',
        display: 'flex',
        flexDirection: 'column',
        gap: '4px',
        minWidth: '160px',
        flex: 1,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          color: 'var(--text-secondary)',
          fontSize: 'var(--text-caption)',
          fontWeight: 500,
        }}
      >
        <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          {icon && <span style={{ opacity: 0.8 }}>{icon}</span>}
          {label}
        </span>
        {badge}
      </div>

      <div
        className="tabular-nums"
        style={{
          fontSize: 'var(--text-title)',
          fontWeight: 600,
          color: 'var(--text-primary)',
          letterSpacing: '-0.02em',
        }}
      >
        {value}
      </div>

      {(subValue || hint) && (
        <div
          style={{
            fontSize: 'var(--text-micro)',
            color: 'var(--text-muted)',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          {subValue && <span>{subValue}</span>}
          {hint && <span style={{ fontStyle: 'italic' }}>{hint}</span>}
        </div>
      )}
    </div>
  );
};
