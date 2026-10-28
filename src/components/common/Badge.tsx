import React from 'react';
import { RiskTier } from '../../types';
import { getRiskTierLabel } from '../../utils/format';

interface BadgeProps {
  variant?: 'safe' | 'caution' | 'high' | 'danger' | 'neutral' | 'accent' | 'warning';
  tier?: RiskTier;
  children?: React.ReactNode;
  className?: string;
}

export const Badge: React.FC<BadgeProps> = ({ variant, tier, children, className = '' }) => {
  let bg = 'var(--bg-surface-2)';
  let text = 'var(--text-secondary)';
  let border = 'var(--border-subtle)';

  const activeVariant = tier || variant || 'neutral';

  switch (activeVariant) {
    case 'safe':
      bg = 'var(--safe-subtle)';
      text = 'var(--safe-text)';
      border = 'var(--safe-border)';
      break;
    case 'caution':
    case 'warning':
      bg = 'var(--warning-subtle)';
      text = 'var(--warning-text)';
      border = 'var(--warning-border)';
      break;
    case 'high':
    case 'danger':
      bg = 'var(--danger-subtle)';
      text = 'var(--danger-base)';
      border = 'var(--danger-border)';
      break;
    case 'accent':
      bg = 'var(--accent-subtle)';
      text = 'var(--accent-base)';
      border = 'var(--accent-border)';
      break;
    default:
      bg = 'var(--bg-surface-1)';
      text = 'var(--text-secondary)';
      border = 'var(--border-subtle)';
      break;
  }

  const content = children || (tier ? getRiskTierLabel(tier) : '');

  return (
    <span
      className={className}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '4px',
        padding: '2px 6px',
        fontSize: 'var(--text-micro)',
        fontWeight: 500,
        lineHeight: 1.2,
        borderRadius: 'var(--radius-sm)',
        backgroundColor: bg,
        color: text,
        border: `1px solid ${border}`,
        whiteSpace: 'nowrap',
        letterSpacing: '0.01em',
      }}
    >
      {content}
    </span>
  );
};

