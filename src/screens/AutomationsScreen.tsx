import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import {
  Clock,
  Plus,
  Play,
  History,
  ShieldCheck,
  RotateCcw,
  Trash2,
  Eye,
  Server,
  X,
} from 'lucide-react';
import type { AutomationRule, AutomationExecutionHistory } from '@/types';

export const AutomationsScreen: React.FC = () => {
  const {
    automations,
    automationHistory,
    schedulerStatus,
    toggleAutomation,
    runAutomation,
    deleteAutomation,
    createAutomation,
    showToast,
    setActiveScreen,
  } = useApp();

  const [activeTab, setActiveTab] = useState<'rules' | 'history'>('rules');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [selectedRuleFilter, setSelectedRuleFilter] = useState<string | null>(null);

  // Form State for New Rule
  const [ruleName, setRuleName] = useState('');
  const [ruleDesc, setRuleDesc] = useState('');
  const [triggerType, setTriggerType] = useState<string>('schedule');
  const [diskThresholdGb, setDiskThresholdGb] = useState<number>(10);
  const [scheduleFreq, setScheduleFreq] = useState<string>('daily');
  const [scopePath, setScopePath] = useState<string>('C:\\');
  const [actionType, setActionType] = useState<string>('create_plan');
  const [dryRunMode, setDryRunMode] = useState<boolean>(true);
  const [requireApproval, setRequireApproval] = useState<boolean>(true);
  const [cooldownSec, setCooldownSec] = useState<number>(300);

  const handleCreateRule = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!ruleName.trim()) {
      showToast('Please specify a rule name.');
      return;
    }

    let triggerObj: any = { type: 'manual' };
    let scheduleExpr = 'Manual Trigger';

    if (triggerType === 'schedule') {
      triggerObj = {
        type: 'schedule',
        frequency: scheduleFreq,
        interval_minutes: scheduleFreq === 'hourly' ? 60 : scheduleFreq === 'daily' ? 1440 : 10080,
      };
      scheduleExpr = `Runs ${scheduleFreq}`;
    } else if (triggerType === 'disk_space_threshold') {
      triggerObj = {
        type: 'disk_space_threshold',
        drive_letter: 'C',
        threshold_gb: diskThresholdGb,
      };
      scheduleExpr = `When free space on C: < ${diskThresholdGb} GB`;
    } else if (triggerType === 'post_scan') {
      triggerObj = {
        type: 'post_scan',
        min_files_scanned: 100,
      };
      scheduleExpr = 'Triggered after every storage scan';
    }

    let actionsList: any[] = [];
    if (actionType === 'create_plan') {
      actionsList.push({
        type: 'create_cleanup_plan',
        goal_type: 'developer_artifacts',
        target_bytes: null,
      });
    } else if (actionType === 'notify') {
      actionsList.push({
        type: 'notify_user',
        title: ruleName,
        message: ruleDesc || 'FileMind automation condition triggered.',
        severity: 'info',
      });
    } else if (actionType === 'stage_quarantine') {
      actionsList.push({
        type: 'stage_to_quarantine',
        dry_run: dryRunMode,
      });
    }

    await createAutomation({
      name: ruleName,
      description: ruleDesc,
      trigger: triggerObj,
      conditions: triggerType === 'disk_space_threshold' ? [{ type: 'disk_free_below_bytes', bytes: diskThresholdGb * 1024 * 1024 * 1024 }] : [],
      actions: actionsList,
      scope_paths: [scopePath],
      exclusion_paths: [],
      schedule_expression: scheduleExpr,
      requires_explicit_approval: requireApproval,
      dry_run_mode: dryRunMode,
      execution_limits: {
        max_files_per_run: 100,
        max_bytes_per_run: 10 * 1024 * 1024 * 1024,
        cooldown_seconds: cooldownSec,
      },
    });

    setIsModalOpen(false);
    setRuleName('');
    setRuleDesc('');
  };

  const filteredHistory = selectedRuleFilter
    ? automationHistory.filter((h) => h.ruleId === selectedRuleFilter)
    : automationHistory;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '16px' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
        <div>
          <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
            Local Automation Engine & Scheduler
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Configurable, safety-governed rules reacting to disk pressure, filesystem events, and schedules.
          </span>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          {/* Tab Selector */}
          <div style={{ display: 'flex', backgroundColor: 'var(--bg-surface-0)', padding: '2px', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
            <button
              type="button"
              onClick={() => setActiveTab('rules')}
              style={{
                padding: '4px 10px',
                fontSize: 'var(--text-caption)',
                fontWeight: activeTab === 'rules' ? 600 : 400,
                backgroundColor: activeTab === 'rules' ? 'var(--bg-surface-2)' : 'transparent',
                color: activeTab === 'rules' ? 'var(--text-primary)' : 'var(--text-muted)',
                borderRadius: 'var(--radius-xs)',
                cursor: 'pointer',
              }}
            >
              Active Rules ({automations.length})
            </button>
            <button
              type="button"
              onClick={() => setActiveTab('history')}
              style={{
                padding: '4px 10px',
                fontSize: 'var(--text-caption)',
                fontWeight: activeTab === 'history' ? 600 : 400,
                backgroundColor: activeTab === 'history' ? 'var(--bg-surface-2)' : 'transparent',
                color: activeTab === 'history' ? 'var(--text-primary)' : 'var(--text-muted)',
                borderRadius: 'var(--radius-xs)',
                cursor: 'pointer',
              }}
            >
              Audit History ({automationHistory.length})
            </button>
          </div>

          <Button size="sm" variant="primary" icon={<Plus size={12} />} onClick={() => setIsModalOpen(true)}>
            New Rule
          </Button>
        </div>
      </div>

      {/* Scheduler & Windows Background Execution Status */}
      <div
        style={{
          padding: '12px 14px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          flexDirection: 'column',
          gap: '8px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Server size={16} style={{ color: 'var(--accent-base)' }} />
            <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
              Scheduler Engine Operating Mode
            </span>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            {schedulerStatus?.isRunning ? (
              <Badge variant="safe">Native Daemon Loop Active</Badge>
            ) : (
              <Badge variant="caution">In-Process UI Mode</Badge>
            )}
            <Badge variant="neutral">
              {schedulerStatus?.windowsScheduledTaskConfigured
                ? 'Windows Scheduled Task Configured'
                : 'Background Daemon Listening (Port 41732)'}
            </Badge>
          </div>
        </div>

        <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4 }}>
          <strong>Background Lifecycle Guarantee:</strong> Scheduled tasks execute via the local FileMind Core daemon. Active filesystem event watchers monitor{' '}
          <span className="font-mono">{schedulerStatus?.activeWatchersCount || 1} folders</span> with 1500ms debouncing and queue overflow fallback. Destructive actions strictly enforce the Safety Guardian and cannot override OS protections.
        </div>
      </div>

      {/* RULES VIEW */}
      {activeTab === 'rules' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {automations.map((auto: AutomationRule) => {
            const ruleId = auto.ruleId || auto.id;
            const isDry = auto.dryRunMode !== false;

            return (
              <div
                key={ruleId}
                style={{
                  backgroundColor: 'var(--bg-surface-0)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-md)',
                  padding: '14px',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '10px',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '8px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Clock size={16} style={{ color: 'var(--accent-base)' }} />
                    <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                      {auto.name}
                    </span>
                    <Badge variant="accent">
                      {typeof auto.trigger === 'string' ? auto.trigger.toUpperCase() : (auto.trigger?.type || 'RULE').toUpperCase()}
                    </Badge>
                    <Badge variant={isDry ? 'neutral' : 'warning'}>
                      {isDry ? 'Dry Run Mode' : 'Direct Action Mode'}
                    </Badge>
                    {auto.requiresExplicitApproval !== false && (
                      <Badge variant="safe">
                        <ShieldCheck size={11} style={{ marginRight: '3px' }} />
                        Explicit Approval Required
                      </Badge>
                    )}
                  </div>

                  {/* Toggle switch */}
                  <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer', fontSize: 'var(--text-caption)' }}>
                    <input
                      type="checkbox"
                      checked={auto.isEnabled}
                      onChange={() => toggleAutomation(ruleId)}
                      style={{ cursor: 'pointer' }}
                    />
                    <span style={{ fontWeight: 500, color: auto.isEnabled ? 'var(--text-primary)' : 'var(--text-muted)' }}>
                      {auto.isEnabled ? 'Active' : 'Disabled'}
                    </span>
                  </label>
                </div>

                <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', margin: 0 }}>
                  {auto.description || auto.criteriaDescription}
                </p>

                <div
                  style={{
                    display: 'flex',
                    flexWrap: 'wrap',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    paddingTop: '8px',
                    borderTop: '1px solid var(--border-subtle)',
                    fontSize: 'var(--text-micro)',
                    color: 'var(--text-muted)',
                    gap: '8px',
                  }}
                >
                  <div style={{ display: 'flex', gap: '14px', flexWrap: 'wrap' }}>
                    <span className="tabular-nums">
                      <strong>Schedule:</strong> {auto.scheduleExpression}
                    </span>
                    <span className="tabular-nums">
                      <strong>Last Run:</strong> {auto.lastExecutedAt || 'Never'}
                    </span>
                    {auto.lastStatus && (
                      <span className="tabular-nums">
                        <strong>Last Status:</strong>{' '}
                        <span style={{ color: auto.lastStatus === 'success' || auto.lastStatus === 'dry_run_completed' ? 'var(--safe-base)' : 'var(--danger-base)' }}>
                          {auto.lastStatus}
                        </span>
                      </span>
                    )}
                  </div>

                  <div style={{ display: 'flex', gap: '6px' }}>
                    <Button
                      size="sm"
                      variant="secondary"
                      icon={<Eye size={11} />}
                      onClick={() => {
                        setSelectedRuleFilter(ruleId);
                        setActiveTab('history');
                      }}
                    >
                      History
                    </Button>

                    <Button
                      size="sm"
                      variant="secondary"
                      icon={<RotateCcw size={11} />}
                      onClick={() => runAutomation(ruleId, true)}
                      title="Run dry-run simulation without touching files"
                    >
                      Simulate Dry Run
                    </Button>

                    <Button
                      size="sm"
                      variant="primary"
                      icon={<Play size={11} />}
                      onClick={() => runAutomation(ruleId, false)}
                    >
                      Run Now
                    </Button>

                    <Button
                      size="sm"
                      variant="ghost"
                      icon={<Trash2 size={11} />}
                      onClick={() => deleteAutomation(ruleId)}
                    />
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* AUDIT HISTORY VIEW */}
      {activeTab === 'history' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {selectedRuleFilter && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 12px', backgroundColor: 'var(--bg-surface-0)', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
              <span style={{ fontSize: 'var(--text-caption)' }}>
                Filtering history by rule: <strong className="font-mono">{selectedRuleFilter}</strong>
              </span>
              <Button size="sm" variant="ghost" onClick={() => setSelectedRuleFilter(null)}>
                Clear Filter
              </Button>
            </div>
          )}

          {filteredHistory.length === 0 ? (
            <div style={{ padding: '32px', textAlign: 'center', color: 'var(--text-muted)', backgroundColor: 'var(--bg-surface-0)', borderRadius: 'var(--radius-md)', border: '1px solid var(--border-subtle)' }}>
              <History size={32} style={{ margin: '0 auto 8px', opacity: 0.5 }} />
              <div>No execution logs recorded yet.</div>
              <div style={{ fontSize: 'var(--text-micro)', marginTop: '4px' }}>
                Run an automation or wait for scheduled triggers to inspect results.
              </div>
            </div>
          ) : (
            filteredHistory.map((hist: AutomationExecutionHistory) => {
              const isSuccess = hist.status === 'success' || hist.status === 'dry_run_completed';
              const isBlocked = hist.status === 'blocked';

              return (
                <div
                  key={hist.runId}
                  style={{
                    backgroundColor: 'var(--bg-surface-0)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: 'var(--radius-md)',
                    padding: '12px 14px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '6px',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span className="font-mono" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                        {hist.runId}
                      </span>
                      <strong style={{ fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                        {hist.ruleName}
                      </strong>
                      <span className="font-mono" style={{ fontSize: 'var(--text-micro)', color: 'var(--accent-base)' }}>
                        via {hist.triggerSource}
                      </span>
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <Badge variant={isSuccess ? (hist.status === 'dry_run_completed' ? 'neutral' : 'safe') : isBlocked ? 'caution' : 'danger'}>
                        {hist.status.toUpperCase()}
                      </Badge>
                      <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                        {new Date(hist.startedAt).toLocaleTimeString()}
                      </span>
                    </div>
                  </div>

                  <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', margin: 0 }}>
                    {hist.resultSummary}
                  </p>

                  {hist.policyDecisions && hist.policyDecisions.length > 0 && (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', padding: '6px 8px', backgroundColor: 'var(--bg-surface-1)', borderRadius: 'var(--radius-xs)', fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                      {hist.policyDecisions.map((p: string, pIdx: number) => (
                        <div key={pIdx}>• {p}</div>
                      ))}
                    </div>
                  )}

                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', paddingTop: '4px', borderTop: '1px solid var(--border-subtle)', fontSize: 'var(--text-micro)' }}>
                    <span className="tabular-nums" style={{ color: 'var(--text-muted)' }}>
                      Files Affected: {hist.filesAffectedCount} | Space: {(hist.bytesAffected / (1024 * 1024)).toFixed(2)} MB
                    </span>

                    {hist.planId && (
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => setActiveScreen('cleanup-planner')}
                      >
                        Inspect Plan {hist.planId}
                      </Button>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}

      {/* NEW RULE DESIGNER MODAL */}
      {isModalOpen && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.4)',
            backdropFilter: 'blur(3px)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
            padding: '16px',
          }}
        >
          <div
            style={{
              backgroundColor: 'var(--bg-surface-0)',
              border: '1px solid var(--border-strong)',
              borderRadius: 'var(--radius-lg)',
              width: '100%',
              maxWidth: '540px',
              maxHeight: '90vh',
              overflowY: 'auto',
              padding: '18px',
              display: 'flex',
              flexDirection: 'column',
              gap: '14px',
              boxShadow: 'var(--shadow-modal)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Clock size={18} style={{ color: 'var(--accent-base)' }} />
                <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                  Create Persistent Automation Rule
                </span>
              </div>
              <button
                type="button"
                onClick={() => setIsModalOpen(false)}
                style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-muted)' }}
              >
                <X size={16} />
              </button>
            </div>

            <form onSubmit={handleCreateRule} style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              <div>
                <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
                  Rule Name
                </label>
                <input
                  type="text"
                  value={ruleName}
                  onChange={(e) => setRuleName(e.target.value)}
                  placeholder="e.g. Daily Inactive Node Modules Cleanup"
                  required
                  style={{ width: '100%' }}
                />
              </div>

              <div>
                <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
                  Description & Objective
                </label>
                <input
                  type="text"
                  value={ruleDesc}
                  onChange={(e) => setRuleDesc(e.target.value)}
                  placeholder="e.g. Scans Git-clean projects and stages stale build outputs into quarantine"
                  style={{ width: '100%' }}
                />
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
                    Trigger Type
                  </label>
                  <select
                    value={triggerType}
                    onChange={(e) => setTriggerType(e.target.value)}
                    style={{ width: '100%', padding: '6px' }}
                  >
                    <option value="schedule">Scheduled Time</option>
                    <option value="disk_space_threshold">Disk Space Below Threshold</option>
                    <option value="post_scan">After Scan Session</option>
                    <option value="manual">Manual Execution Only</option>
                  </select>
                </div>

                <div>
                  <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
                    Schedule / Frequency
                  </label>
                  {triggerType === 'disk_space_threshold' ? (
                    <input
                      type="number"
                      min="1"
                      max="500"
                      value={diskThresholdGb}
                      onChange={(e) => setDiskThresholdGb(Number(e.target.value))}
                      placeholder="Free space threshold (GB)"
                      style={{ width: '100%' }}
                    />
                  ) : (
                    <select
                      value={scheduleFreq}
                      onChange={(e) => setScheduleFreq(e.target.value)}
                      style={{ width: '100%', padding: '6px' }}
                    >
                      <option value="hourly">Hourly</option>
                      <option value="daily">Daily</option>
                      <option value="weekly">Weekly</option>
                    </select>
                  )}
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
                    Target Folder Scope
                  </label>
                  <input
                    type="text"
                    value={scopePath}
                    onChange={(e) => setScopePath(e.target.value)}
                    placeholder="e.g. C:\Users\Developer\Projects"
                    style={{ width: '100%' }}
                  />
                </div>

              <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '10px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
                    Action to Execute
                  </label>
                  <select
                    value={actionType}
                    onChange={(e) => setActionType(e.target.value)}
                    style={{ width: '100%', padding: '6px' }}
                  >
                    <option value="create_plan">Generate Cleanup Plan</option>
                    <option value="notify">Notify User Only</option>
                    <option value="stage_quarantine">Stage to Quarantine Vault</option>
                  </select>
                </div>

                <div>
                  <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
                    Cooldown (s)
                  </label>
                  <input
                    type="number"
                    min="10"
                    max="86400"
                    value={cooldownSec}
                    onChange={(e) => setCooldownSec(Number(e.target.value))}
                    style={{ width: '100%' }}
                  />
                </div>
              </div>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', padding: '8px 10px', backgroundColor: 'var(--bg-surface-1)', borderRadius: 'var(--radius-sm)' }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer', fontSize: 'var(--text-caption)' }}>
                  <input
                    type="checkbox"
                    checked={dryRunMode}
                    onChange={(e) => setDryRunMode(e.target.checked)}
                  />
                  <span>
                    <strong>Dry Run Mode (Recommended):</strong> Only simulates candidates and reports projected space. Never modifies files directly.
                  </span>
                </label>

                <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer', fontSize: 'var(--text-caption)' }}>
                  <input
                    type="checkbox"
                    checked={requireApproval}
                    onChange={(e) => setRequireApproval(e.target.checked)}
                  />
                  <span>
                    <strong>Requires Explicit Approval:</strong> Always pauses destructive operations for user confirmation in Cleanup Planner.
                  </span>
                </label>
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px', marginTop: '6px' }}>
                <Button size="sm" variant="secondary" onClick={() => setIsModalOpen(false)}>
                  Cancel
                </Button>
                <Button size="sm" variant="primary" type="submit">
                  Save Automation Rule
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
