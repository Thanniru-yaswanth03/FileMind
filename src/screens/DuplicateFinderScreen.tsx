import React from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes, copyToClipboard } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { Copy, Sparkles } from 'lucide-react';
import { DuplicateGroup, FileItem } from '@/types';

export const DuplicateFinderScreen: React.FC = () => {
  const { duplicateGroups, setCanonicalDuplicate, showToast, setActiveScreen } = useApp();

  const totalWasted = duplicateGroups.reduce((acc: number, g: DuplicateGroup) => acc + g.wastedBytes, 0);

  const handleAutoSelectNewest = () => {
    duplicateGroups.forEach((group: DuplicateGroup) => {
      // Find the file with the most recent mtime
      const newest = [...group.items].sort((a: FileItem, b: FileItem) => b.mtime.localeCompare(a.mtime))[0];
      if (newest) setCanonicalDuplicate(group.groupId, newest.id);
    });
    showToast('Auto-selected: keeping newest copy in each group.');
  };

  const handleAutoSelectOldest = () => {
    duplicateGroups.forEach((group: DuplicateGroup) => {
      const oldest = [...group.items].sort((a: FileItem, b: FileItem) => a.mtime.localeCompare(b.mtime))[0];
      if (oldest) setCanonicalDuplicate(group.groupId, oldest.id);
    });
    showToast('Auto-selected: keeping oldest copy in each group.');
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', padding: '14px', gap: '12px' }}>
      {/* Header & Helper Actions */}
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
            Deterministic Duplicate Finder
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Multi-stage verified duplicates using 4KB sparse block and BLAKE3 cryptographic stream hashing.
          </span>
        </div>

        {/* Action Helpers */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <Button size="sm" variant="secondary" onClick={handleAutoSelectNewest}>
            Keep Newest Copies
          </Button>
          <Button size="sm" variant="secondary" onClick={handleAutoSelectOldest}>
            Keep Oldest Copies
          </Button>
          <Button
            size="sm"
            variant="primary"
            icon={<Sparkles size={12} />}
            onClick={() => {
              showToast('Added all redundant duplicate copies to Cleanup Plan');
              setActiveScreen('cleanup-planner');
            }}
          >
            Stage Redundant Copies
          </Button>
        </div>
      </div>

      {/* Summary Metrics Bar */}
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
          Identified <strong style={{ color: 'var(--text-primary)' }}>{duplicateGroups.length}</strong> duplicate groups across indexed volumes.
        </span>
        <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--warning-text)' }}>
          Potential Space Recovery: {formatBytes(totalWasted)}
        </span>
      </div>

      {/* Duplicate Groups List */}
      <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {duplicateGroups.map((group: DuplicateGroup, gIdx: number) => (
          <div
            key={group.groupId}
            style={{
              backgroundColor: 'var(--bg-surface-0)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              overflow: 'hidden',
            }}
          >
            {/* Group Header */}
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
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                  Duplicate Set #{gIdx + 1}
                </span>
                <span className="font-mono tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                  Hash: {group.blake3Hash.slice(0, 16)}...
                </span>
                <Badge variant="warning">{group.items.length} exact copies</Badge>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span className="tabular-nums" style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)' }}>
                  File Size: <strong>{formatBytes(group.fileSizeBytes)}</strong>
                </span>
                <span className="tabular-nums" style={{ fontSize: 'var(--text-caption)', color: 'var(--warning-text)', fontWeight: 600 }}>
                  Wasted: {formatBytes(group.wastedBytes)}
                </span>
              </div>
            </div>

            {/* Candidate Files in Group */}
            <div style={{ display: 'flex', flexDirection: 'column' }}>
              {group.items.map((file: FileItem) => {
                const isCanonical = group.selectedCanonicalId === file.id;

                return (
                  <div
                    key={file.id}
                    style={{
                      padding: '8px 12px',
                      borderBottom: '1px solid var(--border-subtle)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      backgroundColor: isCanonical ? 'var(--accent-subtle)' : 'transparent',
                    }}
                  >
                    {/* Left: Selector radio & Path */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flex: 1, minWidth: 0, paddingRight: '12px' }}>
                      <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                        <input
                          type="radio"
                          name={`canonical-${group.groupId}`}
                          checked={isCanonical}
                          onChange={() => setCanonicalDuplicate(group.groupId, file.id)}
                          style={{ cursor: 'pointer' }}
                        />
                        <span
                          style={{
                            fontSize: 'var(--text-micro)',
                            fontWeight: 600,
                            color: isCanonical ? 'var(--accent-base)' : 'var(--text-secondary)',
                          }}
                        >
                          {isCanonical ? 'KEEP THIS' : 'REDUNDANT'}
                        </span>
                      </label>

                      <div className="font-mono truncate" style={{ color: 'var(--text-primary)', fontSize: 'var(--text-caption)' }} title={file.path}>
                        {file.path}
                      </div>
                    </div>

                    {/* Right: Modified Date & Copy Action */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flexShrink: 0 }}>
                      <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                        Modified: {file.mtime}
                      </span>
                      <button
                        type="button"
                        onClick={async () => {
                          await copyToClipboard(file.path);
                          showToast('Path copied to clipboard');
                        }}
                        style={{ color: 'var(--text-secondary)', padding: '2px' }}
                        title="Copy Path"
                      >
                        <Copy size={12} />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
