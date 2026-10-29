import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { ConfirmDialog } from '@/components/common/ConfirmDialog';
import { EmptyState } from '@/components/common/EmptyState';
import { RotateCcw, Trash2, RefreshCw, AlertTriangle, ShieldCheck } from 'lucide-react';
import type { QuarantineRecord, ActivityLog } from '@/types';

export const ActivityRecoveryScreen: React.FC = () => {
  const {
    quarantineRecords,
    restoreQuarantineRecord,
    purgeQuarantineRecord,
    fetchQuarantineVault,
    activityLogs,
  } = useApp();

  const [activeTab, setActiveTab] = useState<'quarantine' | 'audit'>('quarantine');
  const [selectedPurgeRecord, setSelectedPurgeRecord] = useState<string | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [auditFilter, setAuditFilter] = useState<'all' | 'quarantine' | 'restore' | 'purge' | 'scan'>('all');

  const activeQuarantine = quarantineRecords.filter((q: QuarantineRecord) => q.status === 'quarantined');

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await fetchQuarantineVault();
    setIsRefreshing(false);
  };

  const filteredLogs = activityLogs.filter((log: ActivityLog) => {
    if (auditFilter === 'all') return true;
    return log.type === auditFilter;
  });

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
            Zero-data-loss restoration engine, per-volume quarantine vault, and append-only audit ledger.
          </span>
        </div>

        {/* Tab Switcher & Actions */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Button
            size="sm"
            variant="ghost"
            icon={<RefreshCw size={13} className={isRefreshing ? 'spin' : ''} />}
            onClick={handleRefresh}
            disabled={isRefreshing}
          >
            Refresh Vault
          </Button>

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
      </div>

      {/* Recovery Scope & Hardware Advisory */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '6px 12px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-sm)',
          fontSize: 'var(--text-micro)',
          color: 'var(--text-secondary)',
        }}
      >
        <ShieldCheck size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
        <span>
          <strong>Operational Notice:</strong> The Quarantine Vault is an isolated local staging and rollback mechanism with cryptographic manifests. It is not an archival backup service and does not guarantee recovery against physical drive failure, filesystem corruption, or partition wiping. Irreplaceable data should always be mirrored to an independent external backup.
        </span>
      </div>

      {activeTab === 'quarantine' ? (
        /* Quarantine Vault View */
        quarantineRecords.length === 0 ? (
          <EmptyState
            icon={<ShieldCheck size={24} />}
            title="Quarantine Vault is Empty"
            description="No files are currently staged in isolation. Staged cleanups and reversible operations will appear here for 1-click safe restoration."
            actionLabel="Refresh Vault"
            onAction={handleRefresh}
          />
        ) : (
          <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {quarantineRecords.map((item: QuarantineRecord) => (
              <div
                key={item.id}
                style={{
                  backgroundColor: 'var(--bg-surface-0)',
                  border: item.restoreError ? '1px solid var(--danger-border)' : '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-md)',
                  padding: '12px 14px',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '8px',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '12px' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '3px', flex: 1, minWidth: 0 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
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

                    <div style={{ display: 'flex', gap: '16px', fontSize: 'var(--text-micro)', color: 'var(--text-muted)', flexWrap: 'wrap' }}>
                      <span className="tabular-nums">Staged: {item.quarantineDate}</span>
                      <span className="font-mono">Vault ID: {item.manifestId || item.id}</span>
                      {item.blake3Hash && item.blake3Hash !== 'N/A' && (
                        <span className="font-mono">BLAKE3: {item.blake3Hash.slice(0, 16)}...</span>
                      )}
                      {item.reason && <span>Reason: {item.reason}</span>}
                    </div>

                    {item.quarantinePath && (
                      <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
                        Vault location: {item.quarantinePath}
                      </div>
                    )}
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

                {/* Restoration Conflict Warning */}
                {item.restoreError && (
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                      padding: '6px 10px',
                      borderRadius: 'var(--radius-sm)',
                      backgroundColor: 'var(--danger-subtle)',
                      border: '1px solid var(--danger-border)',
                      fontSize: 'var(--text-micro)',
                      color: 'var(--danger-text)',
                    }}
                  >
                    <AlertTriangle size={14} style={{ flexShrink: 0 }} />
                    <span>
                      <strong>Restoration Blocked:</strong> {item.restoreError}. Silent overwrite is strictly prohibited by safety policy.
                    </span>
                  </div>
                )}
              </div>
            ))}
          </div>
        )
      ) : (
        /* Audit Ledger View */
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: '8px', overflow: 'hidden' }}>
          {/* Audit Filters */}
          <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
            {(['all', 'quarantine', 'restore', 'purge', 'scan'] as const).map((filter) => (
              <button
                key={filter}
                type="button"
                onClick={() => setAuditFilter(filter)}
                style={{
                  padding: '3px 8px',
                  fontSize: 'var(--text-micro)',
                  borderRadius: 'var(--radius-xs)',
                  border: '1px solid var(--border-subtle)',
                  backgroundColor: auditFilter === filter ? 'var(--accent-subtle)' : 'var(--bg-surface-0)',
                  color: auditFilter === filter ? 'var(--accent-base)' : 'var(--text-secondary)',
                  fontWeight: auditFilter === filter ? 600 : 400,
                  cursor: 'pointer',
                  textTransform: 'capitalize',
                }}
              >
                {filter}
              </button>
            ))}
          </div>

          <div
            style={{
              flex: 1,
              overflowY: 'auto',
              backgroundColor: 'var(--bg-surface-0)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
            }}
          >
            {filteredLogs.length === 0 ? (
              <div style={{ padding: '32px', textAlign: 'center', color: 'var(--text-muted)' }}>
                No audit entries match filter "{auditFilter}".
              </div>
            ) : (
              filteredLogs.map((log: ActivityLog) => (
                <div
                  key={log.id}
                  style={{
                    padding: '8px 12px',
                    borderBottom: '1px solid var(--border-subtle)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    fontSize: 'var(--text-caption)',
                    gap: '12px',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0, flex: 1 }}>
                    <span className="tabular-nums" style={{ color: 'var(--text-muted)', fontSize: 'var(--text-micro)', width: '70px', flexShrink: 0 }}>
                      {log.timestamp.includes('T') ? log.timestamp.split('T')[1].slice(0, 8) : log.timestamp.split(' ')[0]}
                    </span>
                    <Badge variant={log.severity === 'info' ? 'accent' : log.severity === 'error' ? 'danger' : 'warning'}>
                      {log.type.toUpperCase()}
                    </Badge>
                    <span style={{ fontWeight: 600, color: 'var(--text-primary)', whiteSpace: 'nowrap' }}>
                      {log.title}
                    </span>
                    <span className="truncate" style={{ color: 'var(--text-secondary)', fontSize: 'var(--text-micro)' }}>
                      — {log.description}
                    </span>
                  </div>

                  {log.affectedBytes !== undefined && log.affectedBytes > 0 && (
                    <span className="tabular-nums" style={{ fontWeight: 500, color: 'var(--text-secondary)', flexShrink: 0 }}>
                      {formatBytes(log.affectedBytes)}
                    </span>
                  )}
                </div>
              ))
            )}
          </div>
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
