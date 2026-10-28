import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { ConfirmDialog } from '@/components/common/ConfirmDialog';
import { Archive } from 'lucide-react';
import { CleanupPlanItem } from '@/types';

export const CleanupPlannerScreen: React.FC = () => {
  const { cleanupPlan, togglePlanItemSelection, excludePlanItem, stagePlanToQuarantine, showToast } = useApp();
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);

  const activeItems = cleanupPlan.items.filter((i: CleanupPlanItem) => !i.isExcluded);
  const selectedItems = activeItems.filter((i: CleanupPlanItem) => i.isSelected);
  const totalSelectedBytes = selectedItems.reduce((acc: number, curr: CleanupPlanItem) => acc + curr.file.sizeBytes, 0);

  const hasCautionSelected = selectedItems.some((i: CleanupPlanItem) => i.riskTier === 'caution');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', padding: '14px', gap: '12px' }}>
      {/* Header & Impact Overview */}
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
            Intelligent Cleanup Plan (Dry-Run Preview)
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Review projected storage recovery. Files will be safely staged into the recoverable Quarantine Vault.
          </span>
        </div>

        {/* Approval and Staging Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Button
            size="sm"
            variant="primary"
            icon={<Archive size={13} />}
            disabled={selectedItems.length === 0}
            onClick={() => setIsConfirmOpen(true)}
          >
            Stage Selected ({selectedItems.length}) into Quarantine
          </Button>
        </div>
      </div>

      {/* Plan Summary Card */}
      <div
        style={{
          padding: '10px 14px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: '8px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div>
            <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Reclaimable Space</span>
            <div className="tabular-nums" style={{ fontSize: 'var(--text-title)', fontWeight: 600, color: 'var(--accent-base)' }}>
              {formatBytes(totalSelectedBytes)}
            </div>
          </div>
          <div style={{ height: '24px', width: '1px', backgroundColor: 'var(--border-subtle)' }} />
          <div>
            <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Selected Items</span>
            <div className="tabular-nums" style={{ fontSize: 'var(--text-title)', fontWeight: 600, color: 'var(--text-primary)' }}>
              {selectedItems.length} of {activeItems.length}
            </div>
          </div>
        </div>

        {/* Risk Breakdown Indicators */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <Badge tier="safe">Safe: {cleanupPlan.riskCounts.safe}</Badge>
          <Badge tier="caution">Caution: {cleanupPlan.riskCounts.caution}</Badge>
          {cleanupPlan.riskCounts.high > 0 && <Badge tier="high">High Risk: {cleanupPlan.riskCounts.high}</Badge>}
        </div>
      </div>

      {/* Itemized Plan Table */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
        }}
      >
        {activeItems.map((item: CleanupPlanItem) => (
          <div
            key={item.id}
            style={{
              padding: '10px 12px',
              borderBottom: '1px solid var(--border-subtle)',
              display: 'flex',
              alignItems: 'flex-start',
              justifyContent: 'space-between',
              gap: '12px',
              backgroundColor: item.isSelected ? 'var(--bg-surface-1)' : 'transparent',
            }}
          >
            {/* Checkbox and Target Info */}
            <div style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', flex: 1, minWidth: 0 }}>
              <input
                type="checkbox"
                checked={item.isSelected}
                onChange={() => togglePlanItemSelection(item.id)}
                style={{ marginTop: '3px', cursor: 'pointer' }}
              />

              <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', minWidth: 0 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                    {item.file.name}
                  </span>
                  <Badge tier={item.riskTier} />
                  <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                    {formatBytes(item.file.sizeBytes)}
                  </span>
                </div>

                <div className="font-mono truncate" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }} title={item.file.path}>
                  {item.file.path}
                </div>

                {/* Evidence and Impact Explanation */}
                <div style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', marginTop: '4px' }}>
                  <strong>Safety Rationale: </strong> {item.safetyRationale}
                </div>
                <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                  <strong>Downstream Impact: </strong> {item.potentialImpact}
                </div>
              </div>
            </div>

            {/* Actions: Exclude Button */}
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
              <Button size="sm" variant="secondary" onClick={() => excludePlanItem(item.id)}>
                Exclude
              </Button>
            </div>
          </div>
        ))}
      </div>

      {/* Confirmation Dialog for Staging */}
      <ConfirmDialog
        isOpen={isConfirmOpen}
        title="Confirm Quarantine Staging"
        message={`Are you ready to move ${selectedItems.length} selected items (${formatBytes(totalSelectedBytes)}) into the volume Quarantine Vault?`}
        impactNote={
          hasCautionSelected
            ? 'Caution items are included in this batch. Files are safely restorable with original timestamps and ACLs from the Activity & Recovery tab.'
            : 'Zero data loss: Files will be staged in .filemind_quarantine and can be restored at any time.'
        }
        confirmLabel="Stage to Vault"
        onConfirm={() => {
          stagePlanToQuarantine();
          setIsConfirmOpen(false);
          showToast(`Staged ${selectedItems.length} items (${formatBytes(totalSelectedBytes)}) to Quarantine Vault with zero data loss.`);
        }}
        onCancel={() => setIsConfirmOpen(false)}
      />
    </div>
  );
};
