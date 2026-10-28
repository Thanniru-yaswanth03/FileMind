import React from 'react';
import { ChevronRight, Copy, Check } from 'lucide-react';
import { copyToClipboard } from '../../utils/format';
import { useApp } from '../../context/AppContext';

interface BreadcrumbsProps {
  currentPath: string;
  onNavigate?: (path: string) => void;
}

export const Breadcrumbs: React.FC<BreadcrumbsProps> = ({ currentPath, onNavigate }) => {
  const { showToast } = useApp();
  const [copied, setCopied] = React.useState(false);

  const segments = currentPath.split('\\').filter(Boolean);

  const handleCopy = async () => {
    const success = await copyToClipboard(currentPath);
    if (success) {
      setCopied(true);
      showToast('Full path copied to clipboard');
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '4px',
        padding: '6px 12px',
        backgroundColor: 'var(--bg-surface-1)',
        borderBottom: '1px solid var(--border-subtle)',
        fontSize: 'var(--text-caption)',
        color: 'var(--text-secondary)',
        overflowX: 'auto',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '4px', flex: 1, minWidth: 0 }}>
        {segments.map((seg, idx) => {
          const isLast = idx === segments.length - 1;
          const targetPath = segments.slice(0, idx + 1).join('\\');

          return (
            <React.Fragment key={idx}>
              <button
                type="button"
                onClick={() => onNavigate && onNavigate(targetPath)}
                className="truncate"
                style={{
                  color: isLast ? 'var(--text-primary)' : 'var(--text-secondary)',
                  fontWeight: isLast ? 600 : 400,
                  padding: '2px 4px',
                  borderRadius: 'var(--radius-xs)',
                  maxWidth: '180px',
                }}
                title={targetPath}
              >
                {seg}
              </button>
              {!isLast && <ChevronRight size={12} style={{ color: 'var(--text-muted)', flexShrink: 0 }} />}
            </React.Fragment>
          );
        })}
      </div>

      <button
        type="button"
        onClick={handleCopy}
        title="Copy full path (Ctrl+Shift+C)"
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: '4px',
          padding: '2px 6px',
          borderRadius: 'var(--radius-xs)',
          fontSize: 'var(--text-micro)',
          color: copied ? 'var(--safe-base)' : 'var(--text-secondary)',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          flexShrink: 0,
        }}
      >
        {copied ? <Check size={11} /> : <Copy size={11} />}
        {copied ? 'Copied' : 'Copy Path'}
      </button>
    </div>
  );
};

