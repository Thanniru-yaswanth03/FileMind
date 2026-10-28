import React from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { StatCard } from '@/components/common/StatCard';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import {
  HardDrive,
  Sparkles,
  ArrowRight,
  ShieldCheck,
  FileCode,
  FileArchive,
  Layers,
  Copy,
} from 'lucide-react';
import { DATA_DISCLAIMER } from '@/services/mockData';
import { ScanRoot, DuplicateGroup } from '@/types';

export const OverviewScreen: React.FC = () => {
  const { scanRoots, currentRoot, setActiveScreen, startScan, cleanupPlan, duplicateGroups } = useApp();

  return (
    <div style={{ padding: '16px', display: 'flex', flexDirection: 'column', gap: '16px', height: '100%', overflowY: 'auto' }}>
      {/* Simulation Disclaimer Banner */}
      <div
        style={{
          padding: '6px 12px',
          backgroundColor: 'var(--bg-surface-1)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: 'var(--text-micro)',
          color: 'var(--text-secondary)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <span style={{ fontWeight: 600, color: 'var(--accent-base)' }}>[DEVELOPMENT MODE]</span>
          <span>{DATA_DISCLAIMER}</span>
        </div>
        <Badge variant="accent">Phase 1 Design Preview</Badge>
      </div>

      {/* Primary Drive Metrics Overview */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '12px' }}>
        {scanRoots.map((root: ScanRoot) => {
          const usedPct = Math.round((root.usedBytes / root.totalBytes) * 100);
          const isSelected = root.id === currentRoot.id;

          return (
            <div
              key={root.id}
              style={{
                backgroundColor: 'var(--bg-surface-0)',
                border: isSelected ? '1.5px solid var(--accent-base)' : '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-md)',
                padding: '12px 14px',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <HardDrive size={15} style={{ color: isSelected ? 'var(--accent-base)' : 'var(--text-secondary)' }} />
                  <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                    {root.driveLetter} {root.label}
                  </span>
                </div>
                <Badge variant={root.scanFreshness === 'fresh' ? 'safe' : 'warning'}>
                  {root.scanFreshness === 'fresh' ? 'Fresh Scan' : 'Stale (>5d)'}
                </Badge>
              </div>

              {/* Visual Meter Bar */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                <div
                  style={{
                    height: '8px',
                    width: '100%',
                    backgroundColor: 'var(--bg-surface-2)',
                    borderRadius: 'var(--radius-xs)',
                    overflow: 'hidden',
                  }}
                >
                  <div
                    style={{
                      height: '100%',
                      width: `${usedPct}%`,
                      backgroundColor: usedPct > 85 ? 'var(--danger-base)' : 'var(--accent-base)',
                    }}
                  />
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                  <span className="tabular-nums">Used: {formatBytes(root.usedBytes)} ({usedPct}%)</span>
                  <span className="tabular-nums">Free: {formatBytes(root.freeBytes)}</span>
                </div>
              </div>

              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '4px' }}>
                <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                  Scanned: {root.lastScannedAt}
                </span>
                <Button size="sm" variant="secondary" onClick={() => startScan(root.id)}>
                  Scan Now
                </Button>
              </div>
            </div>
          );
        })}
      </div>

      {/* Key Finding Metric Row */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px' }}>
        <StatCard
          label="Cleanup Potential"
          value={formatBytes(cleanupPlan.totalReclaimableBytes)}
          subValue="Across 3 evaluated candidate items"
          badge={<Badge variant="accent">Actionable</Badge>}
          icon={<Sparkles size={14} style={{ color: 'var(--accent-base)' }} />}
        />
        <StatCard
          label="Identified Duplicates"
          value={formatBytes(duplicateGroups.reduce((acc: number, g: DuplicateGroup) => acc + g.wastedBytes, 0))}
          subValue={`${duplicateGroups.length} redundant duplicate groups`}
          badge={<Badge variant="warning">Review</Badge>}
          icon={<Copy size={14} style={{ color: 'var(--warning-base)' }} />}
        />
        <StatCard
          label="Developer Caches"
          value="20.9 GB"
          subValue="node_modules, target/, .venv"
          badge={<Badge variant="safe">Safe Tier</Badge>}
          icon={<FileCode size={14} style={{ color: 'var(--safe-base)' }} />}
        />
        <StatCard
          label="Safety Evaluation"
          value="100% Guarded"
          subValue="Zero unknown deletions permitted"
          badge={<Badge variant="safe">Enforced</Badge>}
          icon={<ShieldCheck size={14} style={{ color: 'var(--safe-base)' }} />}
        />
      </div>

      {/* Bottom Grid: Top Storage Categories & Actionable Recommendations */}
      <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 1fr', gap: '14px' }}>
        {/* Category Breakdown */}
        <div
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
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 'var(--text-ui)', fontWeight: 600, color: 'var(--text-primary)' }}>
              Top Storage Consumers (Allocated Footprint)
            </span>
            <Button size="sm" variant="ghost" onClick={() => setActiveScreen('storage-explorer')}>
              Explore All <ArrowRight size={12} />
            </Button>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {[
              { name: 'Developer Caches (target/, node_modules)', size: '20.9 GB', pct: 38, icon: <FileCode size={13} /> },
              { name: 'Virtual Disks & WSL Images', size: '48.2 GB', pct: 65, icon: <Layers size={13} /> },
              { name: 'Media Deliverables & 4K Video', size: '16.8 GB', pct: 30, icon: <Layers size={13} /> },
              { name: 'OS Core Binaries (System32 - Protected)', size: '24.5 GB', pct: 45, icon: <HardDrive size={13} /> },
              { name: 'Archives & Downloaded ISOs', size: '11.5 GB', pct: 20, icon: <FileArchive size={13} /> },
            ].map((cat, idx) => (
              <div key={idx} style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 'var(--text-caption)' }}>
                  <span style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--text-primary)' }}>
                    {cat.icon}
                    {cat.name}
                  </span>
                  <span className="tabular-nums" style={{ fontWeight: 500, color: 'var(--text-secondary)' }}>
                    {cat.size}
                  </span>
                </div>
                <div style={{ height: '4px', backgroundColor: 'var(--bg-surface-2)', borderRadius: 'var(--radius-xs)', overflow: 'hidden' }}>
                  <div style={{ height: '100%', width: `${cat.pct}%`, backgroundColor: 'var(--accent-base)' }} />
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Actionable Recommendations */}
        <div
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
          <span style={{ fontSize: 'var(--text-ui)', fontWeight: 600, color: 'var(--text-primary)' }}>
            Actionable Recommendations
          </span>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            <div
              style={{
                padding: '10px',
                backgroundColor: 'var(--accent-subtle)',
                border: '1px solid var(--accent-border)',
                borderRadius: 'var(--radius-sm)',
                display: 'flex',
                flexDirection: 'column',
                gap: '4px',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--accent-base)' }}>
                  Clean Inactive Build Caches
                </span>
                <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--accent-base)' }}>+15.6 GB</span>
              </div>
              <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)', lineHeight: 1.3 }}>
                Rust target/ and WebPlatform node_modules are verified in clean Git repositories and have been inactive for over 14 days.
              </p>
              <div style={{ marginTop: '4px' }}>
                <Button size="sm" variant="primary" onClick={() => setActiveScreen('cleanup-planner')}>
                  Review Plan #001
                </Button>
              </div>
            </div>

            <div
              style={{
                padding: '10px',
                backgroundColor: 'var(--warning-subtle)',
                border: '1px solid var(--warning-border)',
                borderRadius: 'var(--radius-sm)',
                display: 'flex',
                flexDirection: 'column',
                gap: '4px',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--warning-text)' }}>
                  Deduplicate Ubuntu ISO Images
                </span>
                <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--warning-text)' }}>+5.75 GB</span>
              </div>
              <p style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)', lineHeight: 1.3 }}>
                Identified identical BLAKE3 hash matches between Downloads and D:\ISO_Images.
              </p>
              <div style={{ marginTop: '4px' }}>
                <Button size="sm" variant="secondary" onClick={() => setActiveScreen('duplicates')}>
                  Inspect Duplicates
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
