import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes, copyToClipboard } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { FileText, Copy } from 'lucide-react';
import { FileItem } from '@/types';

export const LargeFilesScreen: React.FC = () => {
  const { files, selectedFile, setSelectedFile, showToast, setActiveScreen } = useApp();
  const [minSizeMB, setMinSizeMB] = useState<number>(1000); // Default > 1 GB

  const thresholds = [
    { label: '> 10 GB', mb: 10000 },
    { label: '> 5 GB', mb: 5000 },
    { label: '> 1 GB', mb: 1000 },
    { label: '> 500 MB', mb: 500 },
  ];

  // Filter and sort descending by size
  const largeFiles = [...files]
    .filter((f: FileItem) => f.sizeBytes >= minSizeMB * 1024 * 1024)
    .sort((a: FileItem, b: FileItem) => b.sizeBytes - a.sizeBytes);

  const totalLargeBytes = largeFiles.reduce((acc: number, curr: FileItem) => acc + curr.sizeBytes, 0);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', padding: '14px', gap: '12px' }}>
      {/* Header & Threshold Selector Ribbon */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: '8px',
        }}
      >
        <div>
          <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
            Large File Rankings
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Inspecting the heaviest files and directories contributing to volume pressure.
          </span>
        </div>

        {/* Size Filter Buttons */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-secondary)', marginRight: '4px' }}>
            Threshold:
          </span>
          {thresholds.map((t) => {
            const isSelected = minSizeMB === t.mb;
            return (
              <button
                key={t.mb}
                type="button"
                onClick={() => setMinSizeMB(t.mb)}
                style={{
                  padding: '3px 8px',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: 'var(--text-micro)',
                  fontWeight: isSelected ? 600 : 400,
                  backgroundColor: isSelected ? 'var(--accent-base)' : 'var(--bg-surface-0)',
                  color: isSelected ? 'var(--accent-contrast)' : 'var(--text-secondary)',
                  border: `1px solid ${isSelected ? 'var(--accent-base)' : 'var(--border-subtle)'}`,
                  cursor: 'pointer',
                }}
              >
                {t.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* Summary Card */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: 'var(--text-caption)',
        }}
      >
        <span style={{ color: 'var(--text-secondary)' }}>
          Found <strong style={{ color: 'var(--text-primary)' }}>{largeFiles.length}</strong> items exceeding {minSizeMB >= 1000 ? `${minSizeMB / 1000} GB` : `${minSizeMB} MB`}.
        </span>
        <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
          Total Footprint: {formatBytes(totalLargeBytes)}
        </span>
      </div>

      {/* Large Files Table */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
        }}
      >
        {largeFiles.map((file: FileItem, idx: number) => {
          const isSelected = selectedFile?.id === file.id;

          return (
            <div
              key={file.id}
              onClick={() => setSelectedFile(file)}
              style={{
                display: 'grid',
                gridTemplateColumns: '40px 1.5fr 2.5fr 110px 110px 110px 100px',
                alignItems: 'center',
                padding: '8px 10px',
                borderBottom: '1px solid var(--border-subtle)',
                backgroundColor: isSelected ? 'var(--accent-subtle)' : 'transparent',
                cursor: 'pointer',
                fontSize: 'var(--text-caption)',
              }}
              onMouseEnter={(e) => {
                if (!isSelected) e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)';
              }}
              onMouseLeave={(e) => {
                if (!isSelected) e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              {/* Rank */}
              <div
                className="tabular-nums"
                style={{
                  fontSize: 'var(--text-caption)',
                  fontWeight: 600,
                  color: idx < 3 ? 'var(--accent-base)' : 'var(--text-muted)',
                }}
              >
                #{idx + 1}
              </div>

              {/* Name */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', minWidth: 0, paddingRight: '8px' }}>
                <FileText size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
                <span className="truncate" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                  {file.name}
                </span>
              </div>

              {/* Path */}
              <div className="font-mono truncate" style={{ color: 'var(--text-muted)', paddingRight: '8px' }} title={file.path}>
                {file.path}
              </div>

              {/* Size */}
              <div className="tabular-nums" style={{ textAlign: 'right', fontWeight: 600, color: 'var(--text-primary)' }}>
                {formatBytes(file.sizeBytes)}
              </div>

              {/* Modified Date */}
              <div className="tabular-nums" style={{ textAlign: 'right', color: 'var(--text-secondary)', fontSize: 'var(--text-micro)' }}>
                {file.mtime.split(' ')[0]}
              </div>

              {/* Safety Badge */}
              <div style={{ display: 'flex', justifyContent: 'center' }}>
                <Badge tier={file.riskTier} />
              </div>

              {/* Actions */}
              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '4px' }}>
                <button
                  type="button"
                  title="Copy Path"
                  onClick={async (e) => {
                    e.stopPropagation();
                    await copyToClipboard(file.path);
                    showToast('Path copied');
                  }}
                  style={{ padding: '3px', color: 'var(--text-secondary)' }}
                >
                  <Copy size={12} />
                </button>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={(e) => {
                    e.stopPropagation();
                    showToast(`Staged "${file.name}" to Cleanup Plan`);
                    setActiveScreen('cleanup-planner');
                  }}
                >
                  Stage
                </Button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
