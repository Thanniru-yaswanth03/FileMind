import React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md';
  icon?: React.ReactNode;
  children?: React.ReactNode;
  unavailableReason?: string;
}

export const Button: React.FC<ButtonProps> = ({
  variant = 'secondary',
  size = 'md',
  icon,
  children,
  disabled,
  unavailableReason,
  className = '',
  style,
  ...props
}) => {
  let bg = 'var(--bg-surface-0)';
  let color = 'var(--text-primary)';
  let border = '1px solid var(--border-strong)';

  switch (variant) {
    case 'primary':
      bg = 'var(--accent-base)';
      color = 'var(--accent-contrast)';
      border = '1px solid var(--accent-base)';
      break;
    case 'danger':
      bg = 'var(--danger-base)';
      color = 'var(--danger-contrast)';
      border = '1px solid var(--danger-base)';
      break;
    case 'ghost':
      bg = 'transparent';
      color = 'var(--text-secondary)';
      border = '1px solid transparent';
      break;
    case 'secondary':
    default:
      bg = 'var(--bg-surface-1)';
      color = 'var(--text-primary)';
      border = '1px solid var(--border-subtle)';
      break;
  }

  const padding = size === 'sm' ? '3px 8px' : '5px 12px';
  const fontSize = size === 'sm' ? 'var(--text-caption)' : 'var(--text-body)';

  return (
    <button
      {...props}
      disabled={disabled || !!unavailableReason}
      title={unavailableReason || props.title}
      className={className}
      style={{
        backgroundColor: bg,
        color: color,
        border: border,
        padding: padding,
        fontSize: fontSize,
        ...style,
      }}
    >
      {icon && <span style={{ display: 'inline-flex', alignItems: 'center' }}>{icon}</span>}
      {children}
    </button>
  );
};

