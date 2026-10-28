import React from 'react';
import { useApp } from '../../context/AppContext';
import { formatBytes } from '../../utils/format';
import { ShieldCheck, Activity } from 'lucide-react';

export const StatusBar: React.FC = () => {
  const { currentRoot, isScanning, scanProgress } = useApp();

  const usedPercent = Math.round((currentRoot.usedBytes / currentRoot.totalBytes) * 100);

  return (
    <footer
      style={{
        height: '28px',
        backgroundColor: 'var(--bg-surface-1)',
        borderTop: '1px solid var(--border-subtle)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 12px',
        fontSize: 'var(--text-micro)',
        color: 'var(--text-secondary)',
        flexShrink: 0,
      }}
    >
      {/* Left: Volume & Capacity Info */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{currentRoot.driveLetter}</span>
          <span style={{ color: 'var(--text-muted)' }}>[{currentRoot.filesystem} - {currentRoot.clusterSizeBytes}b cluster]</span>
        </div>

        {/* Capacity Meter */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <div
            style={{
              width: '80px',
              height: '6px',
              backgroundColor: 'var(--bg-surface-2)',
              borderRadius: 'var(--radius-xs)',
              overflow: 'hidden',
              display: 'flex',
            }}
          >
            <div
              style={{
                width: `${usedPercent}%`,
                backgroundColor: usedPercent > 85 ? 'var(--danger-base)' : 'var(--accent-base)',
                transition: 'width 250ms ease',
              }}
            />
          </div>
          <span className="tabular-nums" style={{ color: 'var(--text-primary)', fontWeight: 500 }}>
            {formatBytes(currentRoot.freeBytes)} free of {formatBytes(currentRoot.totalBytes)} ({usedPercent}% used)
          </span>
        </div>
      </div>

      {/* Center: Live Scanning Status Bar */}
      {isScanning && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            color: 'var(--accent-base)',
            fontWeight: 500,
          }}
        >
          <Activity size={12} className="animate-spin" />
          <span className="tabular-nums">
            Scanning: {scanProgress.percent}% ({scanProgress.filesCount.toLocaleString()} files indexed)
          </span>
          <span className="truncate font-mono" style={{ maxWidth: '200px', opacity: 0.8 }}>
            {scanProgress.currentPath}
          </span>
        </div>
      )}

      {/* Right: Security & Guard Status */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px', color: 'var(--safe-text)' }}>
          <ShieldCheck size={12} />
          <span>Safety Guard: Active</span>
        </div>
        <span style={{ color: 'var(--text-muted)' }}>|</span>
        <span style={{ color: 'var(--text-muted)' }}>FileMind v0.1.0-phase1</span>
      </div>
    </footer>
  );
};

