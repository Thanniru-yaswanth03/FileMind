import React, { useState, useEffect } from 'react';
import { useApp } from '@/context/AppContext';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import {
  ShieldCheck,
  AlertOctagon,
  Lock,
  FileCheck,
  RefreshCw,
  Search,
  CheckCircle2,
  XCircle,
  HardDrive,
  History,
  Shield,
  Layers,
} from 'lucide-react';
import {
  SafetyAssessment,
  SafetyAuditRecord,
  SafetyRule,
} from '@/types';
import {
  evaluateSafety,
  fetchSafetyAudit,
  fetchLiveSafetyRules,
  executeGuardedCleanup,
} from '@/services/api';
import { formatBytes } from '@/utils/format';

export const SafetyCenterScreen: React.FC = () => {
  const { showToast } = useApp();

  const [activeTab, setActiveTab] = useState<'evaluator' | 'rules' | 'audit'>('evaluator');
  const [evalPathInput, setEvalPathInput] = useState<string>('C:\\Windows\\System32\\ntoskrnl.exe');
  const [isEvaluating, setIsEvaluating] = useState<boolean>(false);
  const [assessment, setAssessment] = useState<SafetyAssessment | null>(null);

  const [auditLog, setAuditLog] = useState<SafetyAuditRecord[]>([]);
  const [isLoadingAudit, setIsLoadingAudit] = useState<boolean>(false);

  const [liveRules, setLiveRules] = useState<SafetyRule[]>([]);
  const [isExecuting, setIsExecuting] = useState<boolean>(false);

  // Load rules and audit log on mount
  useEffect(() => {
    loadRules();
    loadAuditLog();
    // Run initial evaluation for default path
    handleEvaluate('C:\\Windows\\System32\\ntoskrnl.exe');
  }, []);

  const loadRules = async () => {
    try {
      const rules = await fetchLiveSafetyRules();
      setLiveRules(rules);
    } catch {
      // Fallback if backend not yet running
    }
  };

  const loadAuditLog = async () => {
    setIsLoadingAudit(true);
    try {
      const records = await fetchSafetyAudit(50);
      setAuditLog(records);
    } catch {
      // Ignore initial failure if server booting
    } finally {
      setIsLoadingAudit(false);
    }
  };

  const handleEvaluate = async (targetPath?: string) => {
    const pathToTest = targetPath || evalPathInput.trim();
    if (!pathToTest) {
      showToast('Please specify a target path to evaluate');
      return;
    }

    setIsEvaluating(true);
    try {
      const resp = await evaluateSafety([pathToTest], 'evaluate');
      if (resp.assessments.length > 0) {
        setAssessment(resp.assessments[0]);
      }
      // Refresh audit log to reflect evaluation record
      loadAuditLog();
    } catch (e: any) {
      showToast(e.message || 'Safety evaluation request failed');
    } finally {
      setIsEvaluating(false);
    }
  };

  const handleExecute = async (operation: 'recycle' | 'permanent') => {
    if (!assessment) return;

    if (!assessment.isPermitted && assessment.allowedOperation === 'blocked') {
      showToast('Operation blocked: Target is strictly protected by safety engine');
      return;
    }

    if (!confirm(`Are you sure you want to execute '${operation}' on ${assessment.targetPath}?`)) {
      return;
    }

    setIsExecuting(true);
    try {
      const resp = await executeGuardedCleanup([assessment.canonicalPath], operation);
      if (resp.results.length > 0) {
        const item = resp.results[0];
        if (item.success) {
          showToast(`Cleanup executed successfully via ${item.methodUsed}`);
        } else {
          showToast(`Cleanup rejected: ${item.error || 'Protected'}`);
        }
      }
      loadAuditLog();
      // Re-evaluate target state
      handleEvaluate(assessment.targetPath);
    } catch (e: any) {
      showToast(e.message || 'Operation failed');
    } finally {
      setIsExecuting(false);
    }
  };

  const presetCandidates = [
    { label: 'Windows Kernel', path: 'C:\\Windows\\System32\\ntoskrnl.exe', note: 'Critical OS' },
    { label: 'WSL Ext4 Image', path: 'C:\\Users\\User\\AppData\\Local\\Packages\\Canonical\\LocalState\\ext4.vhdx', note: 'Linux Subsystem' },
    { label: 'Project .env', path: 'C:\\Projects\\app\\.env', note: 'Credential Secret' },
    { label: 'Rust Build Target', path: 'C:\\Projects\\app\\target', note: 'Verified Regenerable' },
    { label: 'Source File', path: 'C:\\Projects\\app\\src\\main.rs', note: 'Source Code' },
    { label: 'Relative Traversal', path: 'C:\\Users\\Public\\..\\Windows\\System32', note: 'Jailbreak Attempt' },
  ];

  const getClassificationBadge = (classification: string) => {
    switch (classification) {
      case 'LowRiskCandidate':
        return <Badge variant="safe">Low-Risk Candidate</Badge>;
      case 'ReviewRequired':
        return <Badge variant="warning">Review Required</Badge>;
      case 'HighRiskOrUnknown':
        return <Badge variant="danger">High Risk or Unknown</Badge>;
      case 'ProtectedAndBlocked':
        return <Badge variant="danger">Protected & Blocked</Badge>;
      default:
        return <Badge variant="neutral">{classification}</Badge>;
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '16px' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div>
          <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
            Smart Delete Guardian & Safety Intelligence
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Deterministic, evidence-based file safety engine in Rust protecting Windows, WSL, and developer ecosystems.
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Badge variant="accent">Deterministic Policy Engine</Badge>
          <Badge variant="neutral">TOCTOU Guard Active</Badge>
        </div>
      </div>

      {/* Core Security Invariant Banner */}
      <div
        style={{
          padding: '12px 16px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1.5px solid var(--accent-base)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          alignItems: 'flex-start',
          gap: '12px',
        }}
      >
        <ShieldCheck size={24} style={{ color: 'var(--accent-base)', flexShrink: 0, marginTop: '2px' }} />
        <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
          <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
            Core Invariant: Fail Closed & Non-Bypassable
          </span>
          <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', lineHeight: 1.5 }}>
            FileMind determines what can be definitively established, what remains unknown, and whether the proposed action is permitted.
            <strong> The AI copilot, frontend, and schedulers cannot bypass this engine.</strong> Ambiguous dependencies, missing lockfiles, or unverified build folders automatically fail closed to safer classifications.
          </p>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div style={{ display: 'flex', gap: '8px', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '8px' }}>
        <button
          type="button"
          onClick={() => setActiveTab('evaluator')}
          style={{
            padding: '6px 12px',
            borderRadius: 'var(--radius-sm)',
            fontSize: 'var(--text-caption)',
            fontWeight: 600,
            cursor: 'pointer',
            backgroundColor: activeTab === 'evaluator' ? 'var(--bg-surface-2)' : 'transparent',
            color: activeTab === 'evaluator' ? 'var(--text-primary)' : 'var(--text-muted)',
            border: 'none',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <Search size={14} />
          <span>Interactive Target Evaluator</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('audit')}
          style={{
            padding: '6px 12px',
            borderRadius: 'var(--radius-sm)',
            fontSize: 'var(--text-caption)',
            fontWeight: 600,
            cursor: 'pointer',
            backgroundColor: activeTab === 'audit' ? 'var(--bg-surface-2)' : 'transparent',
            color: activeTab === 'audit' ? 'var(--text-primary)' : 'var(--text-muted)',
            border: 'none',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <History size={14} />
          <span>Safety Audit Log ({auditLog.length})</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('rules')}
          style={{
            padding: '6px 12px',
            borderRadius: 'var(--radius-sm)',
            fontSize: 'var(--text-caption)',
            fontWeight: 600,
            cursor: 'pointer',
            backgroundColor: activeTab === 'rules' ? 'var(--bg-surface-2)' : 'transparent',
            color: activeTab === 'rules' ? 'var(--text-primary)' : 'var(--text-muted)',
            border: 'none',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <Shield size={14} />
          <span>Active Rules & Policies</span>
        </button>
      </div>

      {/* TAB 1: INTERACTIVE TARGET EVALUATOR */}
      {activeTab === 'evaluator' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          {/* Path Input Box & Presets */}
          <div
            style={{
              padding: '14px',
              backgroundColor: 'var(--bg-surface-0)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              flexDirection: 'column',
              gap: '12px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
                Inspect File or Directory Path
              </span>
              <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Evaluates NTFS handle identity, manifests, lockfiles, and Windows/WSL policies
              </span>
            </div>

            <div style={{ display: 'flex', gap: '8px' }}>
              <input
                type="text"
                value={evalPathInput}
                onChange={(e) => setEvalPathInput(e.target.value)}
                placeholder="Enter absolute file or folder path (e.g. C:\Windows\System32, target/, .env)"
                className="font-mono"
                style={{
                  flex: 1,
                  padding: '8px 12px',
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                  color: 'var(--text-primary)',
                  fontSize: 'var(--text-caption)',
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleEvaluate();
                }}
              />
              <Button
                variant="primary"
                onClick={() => handleEvaluate()}
                disabled={isEvaluating}
                icon={isEvaluating ? <RefreshCw size={14} className="animate-spin" /> : <Search size={14} />}
              >
                {isEvaluating ? 'Evaluating...' : 'Evaluate Target'}
              </Button>
            </div>

            {/* Quick Test Presets */}
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap' }}>
              <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', fontWeight: 600 }}>
                Test Scenarios:
              </span>
              {presetCandidates.map((preset) => (
                <button
                  key={preset.label}
                  type="button"
                  onClick={() => {
                    setEvalPathInput(preset.path);
                    handleEvaluate(preset.path);
                  }}
                  style={{
                    padding: '3px 8px',
                    backgroundColor: 'var(--bg-surface-1)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: 'var(--radius-sm)',
                    fontSize: 'var(--text-micro)',
                    color: 'var(--text-secondary)',
                    cursor: 'pointer',
                  }}
                  title={preset.path}
                >
                  {preset.label} ({preset.note})
                </button>
              ))}
            </div>
          </div>

          {/* Assessment Breakdown Panel */}
          {assessment && (
            <div
              style={{
                padding: '16px',
                backgroundColor: 'var(--bg-surface-0)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-md)',
                display: 'flex',
                flexDirection: 'column',
                gap: '14px',
              }}
            >
              {/* Header Status & Classification */}
              <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: '12px' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
                      Safety Assessment Result
                    </span>
                    {getClassificationBadge(assessment.classification)}
                    {assessment.isPermitted ? (
                      <span style={{ display: 'inline-flex', alignItems: 'center', gap: '4px', color: 'var(--safe-base)', fontSize: 'var(--text-micro)', fontWeight: 600 }}>
                        <CheckCircle2 size={13} /> Operation Permitted
                      </span>
                    ) : (
                      <span style={{ display: 'inline-flex', alignItems: 'center', gap: '4px', color: 'var(--danger-base)', fontSize: 'var(--text-micro)', fontWeight: 600 }}>
                        <XCircle size={13} /> Operation Prohibited
                      </span>
                    )}
                  </div>
                  <div className="font-mono" style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', wordBreak: 'break-all' }}>
                    {assessment.canonicalPath}
                  </div>
                </div>

                {/* Permitted Action CTA */}
                <div style={{ display: 'flex', gap: '8px' }}>
                  {assessment.isPermitted && (
                    <Button
                      variant="primary"
                      size="sm"
                      onClick={() => handleExecute('recycle')}
                      disabled={isExecuting}
                    >
                      Safe Recycle Bin
                    </Button>
                  )}
                  {assessment.allowedOperation === 'requires_explicit_user_confirmation' && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => handleExecute('recycle')}
                      disabled={isExecuting}
                    >
                      Confirm User Recycle
                    </Button>
                  )}
                </div>
              </div>

              {/* Block Reason Callout */}
              {assessment.blockReason && (
                <div
                  style={{
                    padding: '10px 12px',
                    backgroundColor: 'rgba(239, 68, 68, 0.08)',
                    border: '1px solid var(--danger-base)',
                    borderRadius: 'var(--radius-sm)',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '10px',
                  }}
                >
                  <AlertOctagon size={18} style={{ color: 'var(--danger-base)', flexShrink: 0 }} />
                  <div style={{ display: 'flex', flexDirection: 'column' }}>
                    <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--danger-base)' }}>
                      Strict Safety Block Triggered
                    </span>
                    <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)' }}>
                      {assessment.blockReason}
                    </span>
                  </div>
                </div>
              )}

              {/* Metrics Grid */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '10px' }}>
                <div style={{ padding: '8px 12px', backgroundColor: 'var(--bg-surface-1)', borderRadius: 'var(--radius-sm)' }}>
                  <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Target Size</div>
                  <div className="tabular-nums" style={{ fontSize: 'var(--text-body)', fontWeight: 600, color: 'var(--text-primary)' }}>
                    {assessment.sizeBytes > 0 ? formatBytes(assessment.sizeBytes) : (assessment.isDirectory ? 'Directory' : '0 B')}
                  </div>
                </div>

                <div style={{ padding: '8px 12px', backgroundColor: 'var(--bg-surface-1)', borderRadius: 'var(--radius-sm)' }}>
                  <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>NTFS File ID</div>
                  <div className="font-mono truncate" style={{ fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                    {assessment.fileIdentity || 'Handle unavailable'}
                  </div>
                </div>

                <div style={{ padding: '8px 12px', backgroundColor: 'var(--bg-surface-1)', borderRadius: 'var(--radius-sm)' }}>
                  <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Recovery Mode</div>
                  <div style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
                    {assessment.recoveryPossibility}
                  </div>
                </div>

                <div style={{ padding: '8px 12px', backgroundColor: 'var(--bg-surface-1)', borderRadius: 'var(--radius-sm)' }}>
                  <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Operation Permission</div>
                  <div style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: assessment.isPermitted ? 'var(--safe-base)' : 'var(--danger-base)' }}>
                    {assessment.allowedOperation.replace(/_/g, ' ').toUpperCase()}
                  </div>
                </div>
              </div>

              {/* Typed Evidence Records */}
              <div>
                <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)', marginBottom: '8px', display: 'block' }}>
                  Typed Evidence Records ({assessment.evidence.length})
                </span>
                <div
                  style={{
                    border: '1px solid var(--border-subtle)',
                    borderRadius: 'var(--radius-sm)',
                    overflow: 'hidden',
                  }}
                >
                  <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 'var(--text-caption)' }}>
                    <thead>
                      <tr style={{ backgroundColor: 'var(--bg-surface-1)', borderBottom: '1px solid var(--border-subtle)', textAlign: 'left' }}>
                        <th style={{ padding: '6px 10px', color: 'var(--text-secondary)', fontWeight: 600 }}>Evidence Pillar</th>
                        <th style={{ padding: '6px 10px', color: 'var(--text-secondary)', fontWeight: 600 }}>Claim</th>
                        <th style={{ padding: '6px 10px', color: 'var(--text-secondary)', fontWeight: 600 }}>Status</th>
                        <th style={{ padding: '6px 10px', color: 'var(--text-secondary)', fontWeight: 600 }}>Details</th>
                      </tr>
                    </thead>
                    <tbody>
                      {assessment.evidence.map((item, idx) => (
                        <tr key={idx} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                          <td style={{ padding: '6px 10px', whiteSpace: 'nowrap' }}>
                            <Badge variant="neutral">{item.source.replace(/_/g, ' ')}</Badge>
                          </td>
                          <td style={{ padding: '6px 10px', color: 'var(--text-primary)', fontWeight: 500 }}>
                            {item.claim}
                          </td>
                          <td style={{ padding: '6px 10px', whiteSpace: 'nowrap' }}>
                            <span style={{ color: item.supportsSafety ? 'var(--safe-base)' : 'var(--danger-base)', fontWeight: 600 }}>
                              {item.verificationStatus}
                            </span>
                          </td>
                          <td style={{ padding: '6px 10px', color: 'var(--text-muted)' }}>
                            {item.details}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>

              {/* Consequences and Warnings */}
              {assessment.consequences.length > 0 && (
                <div>
                  <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)', marginBottom: '6px', display: 'block' }}>
                    Consequences Analysis
                  </span>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {assessment.consequences.map((c, i) => (
                      <div key={i} style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: 'var(--text-micro)', color: 'var(--text-secondary)' }}>
                        <AlertOctagon size={12} style={{ color: 'var(--warning-base)', flexShrink: 0 }} />
                        <span>{c}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* TAB 2: AUDIT LOG */}
      {activeTab === 'audit' && (
        <div
          style={{
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
            <div>
              <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
                Persistent SQLite Safety Audit Log ({auditLog.length} Records)
              </span>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Immutable historical record of every evaluated target, TOCTOU check, and executed destructive operation
              </div>
            </div>
            <Button
              size="sm"
              variant="secondary"
              onClick={loadAuditLog}
              disabled={isLoadingAudit}
              icon={<RefreshCw size={12} className={isLoadingAudit ? 'animate-spin' : ''} />}
            >
              Refresh Log
            </Button>
          </div>

          <div style={{ maxHeight: '500px', overflowY: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 'var(--text-caption)' }}>
              <thead>
                <tr style={{ backgroundColor: 'var(--bg-surface-1)', borderBottom: '1px solid var(--border-subtle)', textAlign: 'left' }}>
                  <th style={{ padding: '8px 12px', color: 'var(--text-secondary)', fontWeight: 600 }}>ID</th>
                  <th style={{ padding: '8px 12px', color: 'var(--text-secondary)', fontWeight: 600 }}>Target Path</th>
                  <th style={{ padding: '8px 12px', color: 'var(--text-secondary)', fontWeight: 600 }}>Classification</th>
                  <th style={{ padding: '8px 12px', color: 'var(--text-secondary)', fontWeight: 600 }}>Permitted</th>
                  <th style={{ padding: '8px 12px', color: 'var(--text-secondary)', fontWeight: 600 }}>TOCTOU</th>
                  <th style={{ padding: '8px 12px', color: 'var(--text-secondary)', fontWeight: 600 }}>Executed</th>
                  <th style={{ padding: '8px 12px', color: 'var(--text-secondary)', fontWeight: 600 }}>Timestamp</th>
                </tr>
              </thead>
              <tbody>
                {auditLog.map((rec) => (
                  <tr key={rec.auditId} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                    <td style={{ padding: '8px 12px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
                      #{rec.auditId}
                    </td>
                    <td style={{ padding: '8px 12px', color: 'var(--text-primary)', fontFamily: 'var(--font-mono)' }} className="truncate">
                      {rec.targetPath}
                    </td>
                    <td style={{ padding: '8px 12px' }}>
                      {getClassificationBadge(rec.classification)}
                    </td>
                    <td style={{ padding: '8px 12px' }}>
                      {rec.isPermitted ? (
                        <span style={{ color: 'var(--safe-base)', fontWeight: 600 }}>YES</span>
                      ) : (
                        <span style={{ color: 'var(--danger-base)', fontWeight: 600 }}>BLOCKED</span>
                      )}
                    </td>
                    <td style={{ padding: '8px 12px' }}>
                      {rec.toctouVerified ? (
                        <Badge variant="safe">Verified</Badge>
                      ) : (
                        <span style={{ color: 'var(--text-muted)' }}>N/A</span>
                      )}
                    </td>
                    <td style={{ padding: '8px 12px' }}>
                      {rec.executed ? (
                        <Badge variant="accent">Executed</Badge>
                      ) : (
                        <span style={{ color: 'var(--text-muted)' }}>Evaluated Only</span>
                      )}
                    </td>
                    <td style={{ padding: '8px 12px', color: 'var(--text-muted)', fontSize: 'var(--text-micro)' }}>
                      {new Date(rec.timestampEpochMs).toLocaleTimeString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* TAB 3: ACTIVE RULES REGISTRY */}
      {activeTab === 'rules' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {/* Four Pillars Info */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '12px' }}>
            <div style={{ padding: '12px', backgroundColor: 'var(--bg-surface-0)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
                <HardDrive size={14} style={{ color: 'var(--danger-base)' }} />
                <span>Windows Root & Metadata</span>
              </div>
              <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4, marginTop: '4px' }}>
                Strict hardcoded denylists block access to C:\Windows, System32, WinSxS, bootmgr, and registry hives.
              </p>
            </div>

            <div style={{ padding: '12px', backgroundColor: 'var(--bg-surface-0)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
                <Layers size={14} style={{ color: 'var(--warning-base)' }} />
                <span>WSL & Linux Containment</span>
              </div>
              <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4, marginTop: '4px' }}>
                Protects ext4.vhdx virtual hard disks, UWP distribution packages, and \\wsl$\ network mounts from accidental deletion.
              </p>
            </div>

            <div style={{ padding: '12px', backgroundColor: 'var(--bg-surface-0)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
                <FileCheck size={14} style={{ color: 'var(--safe-base)' }} />
                <span>Developer Manifest & Lockfile</span>
              </div>
              <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4, marginTop: '4px' }}>
                Validates parent Cargo.toml/package.json manifests and strict lockfiles before approving target/ or node_modules.
              </p>
            </div>

            <div style={{ padding: '12px', backgroundColor: 'var(--bg-surface-0)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
                <Lock size={14} style={{ color: 'var(--accent-base)' }} />
                <span>TOCTOU & Handle Inspection</span>
              </div>
              <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4, marginTop: '4px' }}>
                Re-opens handles using non-following flags to verify NTFS file ID and size before execution, defeating symlink races.
              </p>
            </div>
          </div>

          {/* Rules List */}
          <div
            style={{
              backgroundColor: 'var(--bg-surface-0)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              overflow: 'hidden',
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
                Active Rust Engine Protection Rules ({liveRules.length})
              </span>
              <Badge variant="accent">Enforced in Native Core</Badge>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column' }}>
              {liveRules.map((rule) => (
                <div
                  key={rule.ruleId}
                  style={{
                    padding: '10px 14px',
                    borderBottom: '1px solid var(--border-subtle)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    gap: '12px',
                  }}
                >
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '3px', flex: 1 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                        {rule.name}
                      </span>
                      <Badge tier={rule.riskTier} />
                      <span className="font-mono" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                        {rule.pattern}
                      </span>
                    </div>
                    <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)' }}>
                      {rule.description}
                    </p>
                  </div>

                  <Badge variant="safe">Active</Badge>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
