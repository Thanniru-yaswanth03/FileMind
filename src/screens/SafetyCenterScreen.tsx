import React from 'react';
import { useApp } from '@/context/AppContext';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import {
  ShieldCheck,
  Lock,
  GitBranch,
  FileCheck,
  AlertOctagon,
  Plus,
} from 'lucide-react';
import { SafetyRule } from '@/types';

export const SafetyCenterScreen: React.FC = () => {
  const { safetyRules, toggleSafetyRule, showToast } = useApp();

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '16px' }}>
      {/* Header */}
      <div>
        <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
          Safety Center & Deterministic Evidence Engine
        </h2>
        <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
          Review the deterministic rules and evidence criteria that guard user files against unintended deletion.
        </span>
      </div>

      {/* The Fundamental Axiom Card */}
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
        <ShieldCheck size={22} style={{ color: 'var(--accent-base)', flexShrink: 0, marginTop: '2px' }} />
        <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
          <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
            Core Invariant: Unknown is Unsafe
          </span>
          <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', lineHeight: 1.5 }}>
            FileMind operates on an evidence-based security model. If a file or directory cannot be verified against a positive safety rule, it is strictly classified as <strong>High Risk</strong> or <strong>Blocked</strong>. Missing manifests, unrecognized extensions, and ambiguous folders are never treated as safe.
          </p>
        </div>
      </div>

      {/* The 4 Pillars of Evidence Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '12px' }}>
        <div
          style={{
            padding: '12px',
            backgroundColor: 'var(--bg-surface-0)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            gap: '6px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
            <FileCheck size={14} style={{ color: 'var(--safe-base)' }} />
            <span>Project Manifests</span>
          </div>
          <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4 }}>
            Confirms directories like <code className="font-mono">target/</code> or <code className="font-mono">node_modules</code> are accompanied by valid manifests (<code className="font-mono">Cargo.toml</code>, <code className="font-mono">package.json</code>).
          </p>
        </div>

        <div
          style={{
            padding: '12px',
            backgroundColor: 'var(--bg-surface-0)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            gap: '6px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
            <GitBranch size={14} style={{ color: 'var(--warning-base)' }} />
            <span>Git Repository Guard</span>
          </div>
          <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4 }}>
            Demotes build targets to High Risk if the parent Git repository has uncommitted changes, untracked files, or dirty stashes.
          </p>
        </div>

        <div
          style={{
            padding: '12px',
            backgroundColor: 'var(--bg-surface-0)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            gap: '6px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
            <Lock size={14} style={{ color: 'var(--danger-base)' }} />
            <span>Win32 Process Locks</span>
          </div>
          <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4 }}>
            Queries the Windows Restart Manager API to detect files held open by active background applications before operations begin.
          </p>
        </div>

        <div
          style={{
            padding: '12px',
            backgroundColor: 'var(--bg-surface-0)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            gap: '6px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600, fontSize: 'var(--text-caption)' }}>
            <AlertOctagon size={14} style={{ color: 'var(--danger-base)' }} />
            <span>Hardcoded OS Denylists</span>
          </div>
          <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4 }}>
            Hardcoded denylists prevent inclusion of <code className="font-mono">C:\Windows</code>, <code className="font-mono">Recovery</code>, <code className="font-mono">EFI</code>, or <code className="font-mono">System Volume Information</code>.
          </p>
        </div>
      </div>

      {/* Active Rules Registry */}
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
            Active Safety Rules ({safetyRules.length})
          </span>
          <Button
            size="sm"
            variant="secondary"
            icon={<Plus size={12} />}
            onClick={() => showToast('Custom rule editor is available in Phase 6')}
          >
            Add Custom Rule
          </Button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column' }}>
          {safetyRules.map((rule: SafetyRule) => (
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

              {/* Status & Toggle */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer', fontSize: 'var(--text-micro)' }}>
                  <input
                    type="checkbox"
                    checked={rule.isEnabled}
                    onChange={() => toggleSafetyRule(rule.ruleId)}
                    style={{ cursor: 'pointer' }}
                  />
                  <span>{rule.isEnabled ? 'Enabled' : 'Disabled'}</span>
                </label>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
