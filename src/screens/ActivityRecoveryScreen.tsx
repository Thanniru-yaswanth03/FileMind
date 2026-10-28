import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { ConfirmDialog } from '@/components/common/ConfirmDialog';
import { RotateCcw, Trash2 } from 'lucide-react';
import type { QuarantineRecord, ActivityLog } from '@/types';

export const ActivityRecoveryScreen: React.FC = () => {
  const {
    quarantineRecords,
    restoreQuarantineRecord,
    purgeQuarantineRecord,
    activityLogs,
  } = useApp();

  const [activeTab, setActiveTab] = useState<'quarantine' | 'audit'>('quarantine');
  const [selectedPurgeRecord, setSelectedPurgeRecord] = useState<string | null>(null);

  const activeQuarantine = quarantineRecords.filter((q: QuarantineRecord) => q.status === 'quarantined');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', padding: '14px', gap: '12px' }}>
      {/* Header & Tabs */}
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
            Activity Ledger & Quarantine Recovery
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Zero-data-loss restoration engine and append-only operational audit log.
          </span>
        </div>

        {/* Tab Switcher */}
        <div style={{ display: 'flex', gap: '4px', backgroundColor: 'var(--bg-surface-0)', padding: '2px', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
          <button
            type="button"
            onClick={() => setActiveTab('quarantine')}
            style={{
              padding: '4px 10px',
              fontSize: 'var(--text-caption)',
              fontWeight: activeTab === 'quarantine' ? 600 : 400,
              backgroundColor: activeTab === 'quarantine' ? 'var(--accent-subtle)' : 'transparent',
              color: activeTab === 'quarantine' ? 'var(--accent-base)' : 'var(--text-secondary)',
              borderRadius: 'var(--radius-xs)',
              cursor: 'pointer',
            }}
          >
            Quarantine Vault ({activeQuarantine.length})
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('audit')}
            style={{
              padding: '4px 10px',
              fontSize: 'var(--text-caption)',
              fontWeight: activeTab === 'audit' ? 600 : 400,
              backgroundColor: activeTab === 'audit' ? 'var(--accent-subtle)' : 'transparent',
              color: activeTab === 'audit' ? 'var(--accent-base)' : 'var(--text-secondary)',
              borderRadius: 'var(--radius-xs)',
              cursor: 'pointer',
            }}
          >
            Audit Ledger ({activityLogs.length})
          </button>
        </div>
      </div>

      {activeTab === 'quarantine' ? (
        /* Quarantine Vault View */
        <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {quarantineRecords.map((item: QuarantineRecord) => (
            <div
              key={item.id}
              style={{
                backgroundColor: 'var(--bg-surface-0)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-md)',
                padding: '10px 14px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: '12px',
              }}
            >
              <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', flex: 1, minWidth: 0 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span className="font-mono truncate" style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                    {item.originalPath}
                  </span>
                  <Badge variant={item.status === 'quarantined' ? 'warning' : item.status === 'restored' ? 'safe' : 'neutral'}>
                    {item.status.toUpperCase()}
                  </Badge>
                  <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                    {formatBytes(item.sizeBytes)}
                  </span>
                </div>

                <div style={{ display: 'flex', gap: '16px', fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                  <span className="tabular-nums">Staged: {item.quarantineDate}</span>
                  <span className="font-mono">Manifest: {item.manifestId}</span>
                  <span className="font-mono">BLAKE3: {item.blake3Hash.slice(0, 14)}...</span>
                </div>
              </div>

              {/* Action Buttons */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
                {item.status === 'quarantined' && (
                  <>
                    <Button
                      size="sm"
                      variant="primary"
                      icon={<RotateCcw size={12} />}
                      onClick={() => restoreQuarantineRecord(item.id)}
                    >
                      Restore File
                    </Button>
                    <Button
                      size="sm"
                      variant="danger"
                      icon={<Trash2 size={12} />}
                      onClick={() => setSelectedPurgeRecord(item.id)}
                    >
                      Purge
                    </Button>
                  </>
                )}
                {item.status === 'restored' && (
                  <span style={{ fontSize: 'var(--text-caption)', color: 'var(--safe-text)', fontWeight: 500 }}>
                    Restored to Origin
                  </span>
                )}
                {item.status === 'purged' && (
                  <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
                    Permanently Purged
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>
      ) : (
        /* Audit Ledger View */
        <div
          style={{
            flex: 1,
            overflowY: 'auto',
            backgroundColor: 'var(--bg-surface-0)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
          }}
        >
          {activityLogs.map((log: ActivityLog) => (
            <div
              key={log.id}
              style={{
                padding: '8px 12px',
                borderBottom: '1px solid var(--border-subtle)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                fontSize: 'var(--text-caption)',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0, flex: 1 }}>
                <span className="tabular-nums" style={{ color: 'var(--text-muted)', fontSize: 'var(--text-micro)', width: '70px' }}>
                  {log.timestamp.split(' ')[0]}
                </span>
                <Badge variant={log.severity === 'info' ? 'accent' : 'warning'}>{log.type.toUpperCase()}</Badge>
                <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{log.title}</span>
                <span className="truncate" style={{ color: 'var(--text-secondary)', fontSize: 'var(--text-micro)' }}>
                  — {log.description}
                </span>
              </div>

              {log.affectedBytes && (
                <span className="tabular-nums" style={{ fontWeight: 500, color: 'var(--text-secondary)' }}>
                  {formatBytes(log.affectedBytes)}
                </span>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Purge Confirmation Dialog */}
      <ConfirmDialog
        isOpen={!!selectedPurgeRecord}
        title="Permanently Purge Quarantined File"
        message="This operation permanently deletes the selected file from the volume quarantine vault. This cannot be undone."
        impactNote="Space will be permanently reclaimed on disk."
        confirmLabel="Purge Permanently"
        isDestructive
        onConfirm={() => {
          if (selectedPurgeRecord) {
            purgeQuarantineRecord(selectedPurgeRecord);
            setSelectedPurgeRecord(null);
          }
        }}
        onCancel={() => setSelectedPurgeRecord(null)}
      />
    </div>
  );
};
