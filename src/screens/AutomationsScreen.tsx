import React from 'react';
import { useApp } from '@/context/AppContext';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { Clock, Plus, CheckCircle2, Play } from 'lucide-react';
import type { AutomationRule } from '@/types';

export const AutomationsScreen: React.FC = () => {
  const { automations, toggleAutomation, showToast } = useApp();

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '16px' }}>
      {/* Header */}
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
            Configurable File Automations
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Declarative scheduled cleanup tasks executing with low I/O priority and bounded safety gates.
          </span>
        </div>

        <Button
          size="sm"
          variant="primary"
          icon={<Plus size={12} />}
          onClick={() => showToast('New automation workflow designer available in Phase 6')}
        >
          New Automation Rule
        </Button>
      </div>

      {/* Safety Constraint Note */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          fontSize: 'var(--text-caption)',
          color: 'var(--text-secondary)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        }}
      >
        <CheckCircle2 size={15} style={{ color: 'var(--safe-base)', flexShrink: 0 }} />
        <span>
          <strong>Automated Safety Guarantee:</strong> Background tasks only operate on <strong>Tier 1 (Safe)</strong> files. Any ambiguous or caution item automatically pauses execution and posts a desktop notification for manual review.
        </span>
      </div>

      {/* Workflows List */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {automations.map((auto: AutomationRule) => (
          <div
            key={auto.id}
            style={{
              backgroundColor: 'var(--bg-surface-0)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              padding: '14px',
              display: 'flex',
              flexDirection: 'column',
              gap: '8px',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Clock size={16} style={{ color: 'var(--accent-base)' }} />
                <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                  {auto.name}
                </span>
                <Badge variant="accent">{auto.trigger.toUpperCase()}</Badge>
                <Badge tier={auto.targetTier} />
              </div>

              {/* Toggle switch */}
              <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer', fontSize: 'var(--text-caption)' }}>
                <input
                  type="checkbox"
                  checked={auto.isEnabled}
                  onChange={() => toggleAutomation(auto.id)}
                  style={{ cursor: 'pointer' }}
                />
                <span style={{ fontWeight: 500, color: auto.isEnabled ? 'var(--text-primary)' : 'var(--text-muted)' }}>
                  {auto.isEnabled ? 'Active' : 'Disabled'}
                </span>
              </label>
            </div>

            <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)' }}>
              <strong>Criteria: </strong> {auto.criteriaDescription}
            </p>

            <div
              style={{
                display: 'flex',
                flexWrap: 'wrap',
                alignItems: 'center',
                justifyContent: 'space-between',
                paddingTop: '6px',
                borderTop: '1px solid var(--border-subtle)',
                fontSize: 'var(--text-micro)',
                color: 'var(--text-muted)',
              }}
            >
              <div style={{ display: 'flex', gap: '14px' }}>
                <span className="tabular-nums">Schedule: {auto.scheduleExpression}</span>
                <span className="tabular-nums">Last Run: {auto.lastExecutedAt || 'Never'}</span>
                <span className="tabular-nums">Next: {auto.nextScheduledAt || 'Pending'}</span>
              </div>

              <div style={{ display: 'flex', gap: '6px' }}>
                <Button size="sm" variant="secondary" icon={<Play size={10} />} onClick={() => showToast(`Manually triggered: ${auto.name}`)}>
                  Run Now
                </Button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
