import React, { useState, useEffect } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { ConfirmDialog } from '@/components/common/ConfirmDialog';
import {
  Archive,
  RefreshCw,
  ShieldCheck,
  AlertTriangle,
  Play,
  CheckCircle,
} from 'lucide-react';
import {
  CleanupPlanDto,
  DryRunResponse,
  ExecutePlanResponse,
} from '@/types';
import {
  createCleanupPlan,
  executeDryRun,
  executeCleanupPlan,
} from '@/services/api';

export const CleanupPlannerScreen: React.FC = () => {
  const { showToast } = useApp();

  const [goal, setGoal] = useState<string>('maximize_safe');
  const [targetBytes, setTargetBytes] = useState<number>(5 * 1024 * 1024 * 1024); // 5 GB default
  const [folderPath] = useState<string>('C:\\Projects');

  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isDryRunning, setIsDryRunning] = useState<boolean>(false);
  const [isExecuting, setIsExecuting] = useState<boolean>(false);

  const [plan, setPlan] = useState<CleanupPlanDto | null>(null);
  const [dryRunResult, setDryRunResult] = useState<DryRunResponse | null>(null);
  const [executionResult, setExecutionResult] = useState<ExecutePlanResponse | null>(null);

  const [isConfirmOpen, setIsConfirmOpen] = useState<boolean>(false);

  // Generate initial plan on mount
  useEffect(() => {
    handleGeneratePlan('maximize_safe');
  }, []);

  const handleGeneratePlan = async (requestedGoal?: string) => {
    const selectedGoal = requestedGoal || goal;
    setIsLoading(true);
    setDryRunResult(null);
    setExecutionResult(null);

    try {
      const generated = await createCleanupPlan({
        goal: selectedGoal,
        targetBytes: selectedGoal === 'free_bytes' ? targetBytes : undefined,
        folderPath: selectedGoal === 'clean_folder' ? folderPath : undefined,
      });
      setPlan(generated);
      showToast(`Generated cleanup plan with ${generated.items.length} candidate items`);
    } catch (e: any) {
      showToast(e.message || 'Failed to generate plan');
    } finally {
      setIsLoading(false);
    }
  };

  const handleRunDryRun = async () => {
    if (!plan) return;
    setIsDryRunning(true);
    try {
      const res = await executeDryRun(plan.planId);
      setDryRunResult(res);
      showToast(`Dry-run simulation completed: ${res.validatedItemsCount} items validated.`);
    } catch (e: any) {
      showToast(e.message || 'Dry run failed');
    } finally {
      setIsDryRunning(false);
    }
  };

  const handleExecutePlan = async () => {
    if (!plan) return;
    setIsExecuting(true);
    try {
      const selectedIds = plan.items
        .filter((i) => i.isSelected && !i.isExcluded)
        .map((i) => i.itemId);

      const res = await executeCleanupPlan(plan.planId, selectedIds);
      setExecutionResult(res);
      showToast(
        `Execution finished: ${res.succeededCount} items reclaimed (${formatBytes(res.totalBytesReclaimed)}).`
      );
      // Refresh plan state
      handleGeneratePlan();
    } catch (e: any) {
      showToast(e.message || 'Plan execution failed');
    } finally {
      setIsExecuting(false);
      setIsConfirmOpen(false);
    }
  };

  const toggleItemSelection = (itemId: string) => {
    if (!plan) return;
    setPlan({
      ...plan,
      items: plan.items.map((i) =>
        i.itemId === itemId ? { ...i, isSelected: !i.isSelected } : i
      ),
    });
  };

  const excludeItem = (itemId: string) => {
    if (!plan) return;
    setPlan({
      ...plan,
      items: plan.items.map((i) =>
        i.itemId === itemId ? { ...i, isExcluded: true, isSelected: false } : i
      ),
    });
    showToast('Candidate item excluded from plan');
  };

  const activeItems = plan ? plan.items.filter((i) => !i.isExcluded) : [];
  const selectedItems = activeItems.filter((i) => i.isSelected);
  const totalSelectedBytes = selectedItems.reduce((acc, curr) => acc + curr.physicalSizeBytes, 0);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '14px' }}>
      {/* Header & Impact Overview */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
        <div>
          <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
            Intelligent Cleanup Planning & Recoverable Operations
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Formulate defensible cleanup plans with strict overlap avoidance, dry-run simulation, and quarantine restoration.
          </span>
        </div>

        {/* Global Action Ribbon */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Button
            size="sm"
            variant="secondary"
            icon={<Play size={12} />}
            disabled={!plan || isDryRunning || activeItems.length === 0}
            onClick={handleRunDryRun}
          >
            {isDryRunning ? 'Simulating Dry-Run...' : 'Simulate Dry-Run'}
          </Button>

          <Button
            size="sm"
            variant="primary"
            icon={<Archive size={13} />}
            disabled={!plan || selectedItems.length === 0 || isExecuting}
            onClick={() => setIsConfirmOpen(true)}
          >
            Stage & Execute Selected ({selectedItems.length})
          </Button>
        </div>
      </div>

      {/* Goal Formulation Card */}
      <div
        style={{
          padding: '12px 14px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          flexDirection: 'column',
          gap: '10px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
            Planning Objective & Target Formulation
          </span>
          <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
            Planning objective only — will never delete unverified files indiscriminately
          </span>
        </div>

        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' }}>
          <button
            type="button"
            onClick={() => {
              setGoal('maximize_safe');
              handleGeneratePlan('maximize_safe');
            }}
            style={{
              padding: '6px 12px',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-caption)',
              fontWeight: 500,
              cursor: 'pointer',
              backgroundColor: goal === 'maximize_safe' ? 'var(--bg-surface-2)' : 'var(--bg-surface-1)',
              color: goal === 'maximize_safe' ? 'var(--text-primary)' : 'var(--text-secondary)',
              border: '1px solid var(--border-subtle)',
            }}
          >
            Maximize Safe Practical Space
          </button>

          <button
            type="button"
            onClick={() => {
              setGoal('clean_developer_caches');
              handleGeneratePlan('clean_developer_caches');
            }}
            style={{
              padding: '6px 12px',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-caption)',
              fontWeight: 500,
              cursor: 'pointer',
              backgroundColor: goal === 'clean_developer_caches' ? 'var(--bg-surface-2)' : 'var(--bg-surface-1)',
              color: goal === 'clean_developer_caches' ? 'var(--text-primary)' : 'var(--text-secondary)',
              border: '1px solid var(--border-subtle)',
            }}
          >
            Clean Developer Caches & Build Outputs
          </button>

          <button
            type="button"
            onClick={() => {
              setGoal('review_duplicates');
              handleGeneratePlan('review_duplicates');
            }}
            style={{
              padding: '6px 12px',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-caption)',
              fontWeight: 500,
              cursor: 'pointer',
              backgroundColor: goal === 'review_duplicates' ? 'var(--bg-surface-2)' : 'var(--bg-surface-1)',
              color: goal === 'review_duplicates' ? 'var(--text-primary)' : 'var(--text-secondary)',
              border: '1px solid var(--border-subtle)',
            }}
          >
            Review Redundant Duplicates
          </button>

          <button
            type="button"
            onClick={() => {
              setGoal('free_bytes');
              setTargetBytes(5 * 1024 * 1024 * 1024);
              handleGeneratePlan('free_bytes');
            }}
            style={{
              padding: '6px 12px',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-caption)',
              fontWeight: 500,
              cursor: 'pointer',
              backgroundColor: goal === 'free_bytes' && targetBytes === 5 * 1024 * 1024 * 1024 ? 'var(--bg-surface-2)' : 'var(--bg-surface-1)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-subtle)',
            }}
          >
            Target: Free 5 GB
          </button>

          <button
            type="button"
            onClick={() => {
              setGoal('free_bytes');
              setTargetBytes(10 * 1024 * 1024 * 1024);
              handleGeneratePlan('free_bytes');
            }}
            style={{
              padding: '6px 12px',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-caption)',
              fontWeight: 500,
              cursor: 'pointer',
              backgroundColor: goal === 'free_bytes' && targetBytes === 10 * 1024 * 1024 * 1024 ? 'var(--bg-surface-2)' : 'var(--bg-surface-1)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-subtle)',
            }}
          >
            Target: Free 10 GB
          </button>

          <Button
            size="sm"
            variant="secondary"
            icon={<RefreshCw size={12} className={isLoading ? 'animate-spin' : ''} />}
            onClick={() => handleGeneratePlan()}
            disabled={isLoading}
          >
            Refresh Plan
          </Button>
        </div>
      </div>

      {/* Target Attainability Alert Banner */}
      {plan && !plan.isTargetAttainableSafely && plan.unmetTargetExplanation && (
        <div
          style={{
            padding: '10px 14px',
            backgroundColor: 'rgba(239, 68, 68, 0.08)',
            border: '1px solid var(--danger-base)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
          }}
        >
          <AlertTriangle size={20} style={{ color: 'var(--danger-base)', flexShrink: 0 }} />
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--danger-base)' }}>
              Storage Target Not Fully Attainable Safely
            </span>
            <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)' }}>
              {plan.unmetTargetExplanation}
            </span>
          </div>
        </div>
      )}

      {/* Dry Run Simulation Results Banner */}
      {dryRunResult && (
        <div
          style={{
            padding: '12px 14px',
            backgroundColor: dryRunResult.canExecute ? 'rgba(34, 197, 94, 0.08)' : 'rgba(234, 179, 8, 0.08)',
            border: `1px solid ${dryRunResult.canExecute ? 'var(--safe-base)' : 'var(--warning-base)'}`,
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            gap: '6px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <ShieldCheck size={18} style={{ color: dryRunResult.canExecute ? 'var(--safe-base)' : 'var(--warning-base)' }} />
              <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                Genuine Dry-Run Simulation Results (Zero Data Touched)
              </span>
            </div>
            <Badge variant={dryRunResult.canExecute ? 'safe' : 'warning'}>
              {dryRunResult.canExecute ? 'Safe to Execute' : 'Conflicts Detected'}
            </Badge>
          </div>
          <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)' }}>
            Validated {dryRunResult.validatedItemsCount} operations with projected physical space recovery of <strong>{formatBytes(dryRunResult.totalExpectedFreedBytes)}</strong>.
            Permissions verified, locks checked, and OS protection confirmed.
          </p>
        </div>
      )}

      {/* Execution Results Banner */}
      {executionResult && (
        <div
          style={{
            padding: '12px 14px',
            backgroundColor: executionResult.failedCount === 0 ? 'rgba(34, 197, 94, 0.08)' : 'rgba(234, 179, 8, 0.08)',
            border: `1px solid ${executionResult.failedCount === 0 ? 'var(--safe-base)' : 'var(--warning-base)'}`,
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            gap: '6px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <CheckCircle size={18} style={{ color: executionResult.failedCount === 0 ? 'var(--safe-base)' : 'var(--warning-base)' }} />
              <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                Cleanup Execution Completed
              </span>
            </div>
            <Badge variant={executionResult.failedCount === 0 ? 'safe' : 'warning'}>
              {executionResult.status.toUpperCase()}
            </Badge>
          </div>
          <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)' }}>
            Successfully processed {executionResult.succeededCount} items ({formatBytes(executionResult.totalBytesReclaimed)} reclaimed).
            {executionResult.quarantinedCount > 0 && ` ${executionResult.quarantinedCount} moved to Quarantine Vault.`}
            {executionResult.recycledCount > 0 && ` ${executionResult.recycledCount} moved to Recycle Bin.`}
            {executionResult.failedCount > 0 && ` ${executionResult.failedCount} skipped or failed.`}
          </p>
        </div>
      )}

      {/* Plan Metrics Summary Card */}
      {plan && (
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
            gap: '12px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
            <div>
              <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Selected Reclaimable (On Disk)</span>
              <div className="tabular-nums" style={{ fontSize: 'var(--text-title)', fontWeight: 600, color: 'var(--accent-base)' }}>
                {formatBytes(totalSelectedBytes)}
              </div>
            </div>
            <div style={{ height: '24px', width: '1px', backgroundColor: 'var(--border-subtle)' }} />
            <div>
              <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Total Safe Available</span>
              <div className="tabular-nums" style={{ fontSize: 'var(--text-title)', fontWeight: 600, color: 'var(--text-primary)' }}>
                {formatBytes(plan.totalPhysicalReclaimableBytes)}
              </div>
            </div>
            <div style={{ height: '24px', width: '1px', backgroundColor: 'var(--border-subtle)' }} />
            <div>
              <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Selected Candidates</span>
              <div className="tabular-nums" style={{ fontSize: 'var(--text-title)', fontWeight: 600, color: 'var(--text-primary)' }}>
                {selectedItems.length} of {activeItems.length}
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <Badge tier="safe">Low-Risk Safe: {plan.riskCounts['safe'] || 0}</Badge>
            <Badge tier="caution">Review Required: {plan.riskCounts['caution'] || 0}</Badge>
          </div>
        </div>
      )}

      {/* Candidate Items Review Table */}
      <div
        style={{
          flex: 1,
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <div
          style={{
            padding: '10px 14px',
            backgroundColor: 'var(--bg-surface-1)',
            borderBottom: '1px solid var(--border-subtle)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
            Candidate Actions Review ({activeItems.length} items)
          </span>
          <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
            No destructive actions are pre-selected on Review Required files
          </span>
        </div>

        <div style={{ flex: 1, overflowY: 'auto' }}>
          {activeItems.map((item) => (
            <div
              key={item.itemId}
              style={{
                padding: '10px 14px',
                borderBottom: '1px solid var(--border-subtle)',
                display: 'flex',
                alignItems: 'flex-start',
                justifyContent: 'space-between',
                gap: '12px',
                backgroundColor: item.isSelected ? 'var(--bg-surface-1)' : 'transparent',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', flex: 1, minWidth: 0 }}>
                <input
                  type="checkbox"
                  checked={item.isSelected}
                  onChange={() => toggleItemSelection(item.itemId)}
                  style={{ marginTop: '4px', cursor: 'pointer' }}
                />

                <div style={{ display: 'flex', flexDirection: 'column', gap: '3px', flex: 1, minWidth: 0 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                    <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                      {item.path.split(/[/\\]/).pop()}
                    </span>
                    <Badge tier={item.riskTier} />
                    <Badge variant="neutral">{item.operation.toUpperCase()}</Badge>
                    <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                      {formatBytes(item.physicalSizeBytes)} on disk
                    </span>
                    <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                      ({formatBytes(item.logicalSizeBytes)} logical)
                    </span>
                  </div>

                  <div className="font-mono truncate" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }} title={item.path}>
                    {item.path}
                  </div>

                  <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)', marginTop: '2px' }}>
                    <strong>Evidence: </strong> {item.evidenceSummary}
                  </div>

                  <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                    <strong>Reversibility: </strong> {item.isReversible ? 'Reversible via Quarantine / Recycle Bin' : 'Irreversible'}
                  </div>
                </div>
              </div>

              {/* Exclusion and Inspection Buttons */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => excludeItem(item.itemId)}
                >
                  Exclude
                </Button>
              </div>
            </div>
          ))}

          {activeItems.length === 0 && (
            <div style={{ padding: '32px', textAlign: 'center', color: 'var(--text-muted)', fontSize: 'var(--text-caption)' }}>
              No candidates generated. Try selecting a different planning objective above.
            </div>
          )}
        </div>
      </div>

      {/* Confirmation Dialog */}
      <ConfirmDialog
        isOpen={isConfirmOpen}
        title="Confirm Intelligent Cleanup Execution"
        message={`Execute cleanup on ${selectedItems.length} selected items (${formatBytes(totalSelectedBytes)} on disk)?`}
        impactNote="Quarantined files will be staged into the recoverable Quarantine Vault with preserved metadata. You can restore them at any time from the Activity Ledger & Quarantine tab."
        confirmLabel="Execute Cleanup"
        onConfirm={handleExecutePlan}
        onCancel={() => setIsConfirmOpen(false)}
      />
    </div>
  );
};
