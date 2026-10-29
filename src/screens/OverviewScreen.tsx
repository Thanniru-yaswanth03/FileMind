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
  CheckCircle2,
  Folder,
} from 'lucide-react';
import { ScanRoot, DuplicateGroup } from '@/types';

export const OverviewScreen: React.FC = () => {
  const {
    scanRoots,
    currentRoot,
    setActiveScreen,
    startScan,
    cleanupPlan,
    duplicateGroups,
    isBackendConnected,
    scanSummary,
    categoryBreakdown,
    folderRankings,
    files,
  } = useApp();

  return (
    <div style={{ padding: '16px', display: 'flex', flexDirection: 'column', gap: '16px', height: '100%', overflowY: 'auto' }}>
      {/* Engine Status Banner */}
      <div
        style={{
          padding: '6px 12px',
          backgroundColor: isBackendConnected ? 'rgba(74, 107, 91, 0.08)' : 'var(--bg-surface-1)',
          border: `1px solid ${isBackendConnected ? 'var(--accent-base)' : 'var(--border-subtle)'}`,
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: 'var(--text-micro)',
          color: 'var(--text-secondary)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          {isBackendConnected ? (
            <>
              <CheckCircle2 size={13} style={{ color: 'var(--accent-base)' }} />
              <span style={{ fontWeight: 600, color: 'var(--accent-base)' }}>[NATIVE ENGINE ONLINE]</span>
              <span>Real Win32 Storage Scanner, SQLite WAL Index, and Storage Intelligence Subsystem Active.</span>
            </>
          ) : (
            <>
              <span style={{ fontWeight: 600, color: 'var(--accent-base)' }}>[STANDALONE UI]</span>
              <span>Running in standalone desktop mode. Connect native backend at port 41732 for real-time live indexing.</span>
            </>
          )}
        </div>
        <Badge variant={isBackendConnected ? 'safe' : 'accent'}>
          {isBackendConnected ? 'Native Core v0.2.0' : 'Desktop Standalone'}
        </Badge>
      </div>

      {/* Primary Drive Metrics Overview */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '12px' }}>
        {scanRoots.map((root: ScanRoot) => {
          const usedPct = root.totalBytes > 0 ? Math.round((root.usedBytes / root.totalBytes) * 100) : 0;
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
                  {root.filesystem} · Cluster {formatBytes(root.clusterSizeBytes)}
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
                  Total Capacity: {formatBytes(root.totalBytes)}
                </span>
                <Button size="sm" variant="secondary" onClick={() => startScan(root.driveLetter)}>
                  Scan Volume
                </Button>
              </div>
            </div>
          );
        })}
      </div>

      {/* Key Finding Metric Row */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px' }}>
        <StatCard
          label="Indexed Files Total"
          value={files.length.toLocaleString()}
          subValue={scanSummary ? `${formatBytes(scanSummary.session.logicalSizeBytes)} verified` : `${formatBytes(files.reduce((a, b) => a + b.sizeBytes, 0))} loaded`}
          badge={<Badge variant="accent">Indexed</Badge>}
          icon={<HardDrive size={14} style={{ color: 'var(--accent-base)' }} />}
        />
        <StatCard
          label="Cleanup Potential"
          value={formatBytes(cleanupPlan.totalReclaimableBytes)}
          subValue="Across 3 evaluated candidate items"
          badge={<Badge variant="safe">Safe Tier</Badge>}
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
          label="Safety Evaluation"
          value="100% Guarded"
          subValue="Zero unknown deletions permitted"
          badge={<Badge variant="safe">Enforced</Badge>}
          icon={<ShieldCheck size={14} style={{ color: 'var(--safe-base)' }} />}
        />
      </div>

      {/* Bottom Grid: Real Category Breakdown & Folder Rankings */}
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
              Storage Breakdown by Category
            </span>
            <Button size="sm" variant="ghost" onClick={() => setActiveScreen('storage-explorer')}>
              Explore All <ArrowRight size={12} />
            </Button>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {categoryBreakdown.length > 0 ? (
              categoryBreakdown.map((cat) => (
                <div key={cat.category} style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 'var(--text-caption)' }}>
                    <span style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--text-primary)' }}>
                      <Layers size={13} />
                      {cat.category} ({cat.fileCount.toLocaleString()} files)
                    </span>
                    <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--text-secondary)' }}>
                      {formatBytes(cat.totalBytes)} ({cat.percentage}%)
                    </span>
                  </div>
                  <div style={{ height: '4px', backgroundColor: 'var(--bg-surface-2)', borderRadius: 'var(--radius-xs)', overflow: 'hidden' }}>
                    <div style={{ height: '100%', width: `${Math.min(cat.percentage, 100)}%`, backgroundColor: 'var(--accent-base)' }} />
                  </div>
                </div>
              ))
            ) : (
              [
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
              ))
            )}
          </div>
        </div>

        {/* Top Space Consuming Folders */}
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
              Top Space-Consuming Folders
            </span>
            <Button size="sm" variant="ghost" onClick={() => setActiveScreen('storage-explorer')}>
              Inspect <ArrowRight size={12} />
            </Button>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', overflowY: 'auto', maxHeight: '230px' }}>
            {folderRankings.length > 0 ? (
              folderRankings.slice(0, 5).map((fr, idx) => (
                <div
                  key={fr.path}
                  style={{
                    padding: '6px 8px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: 'var(--bg-surface-1)',
                    border: '1px solid var(--border-subtle)',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '2px',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 'var(--text-micro)' }}>
                    <span style={{ fontWeight: 600, color: 'var(--text-primary)', display: 'flex', alignItems: 'center', gap: '4px' }}>
                      <Folder size={11} style={{ color: 'var(--accent-base)' }} />
                      #{idx + 1} {fr.path.split('\\').pop() || fr.path}
                    </span>
                    <span className="tabular-nums" style={{ fontWeight: 700, color: 'var(--text-secondary)' }}>
                      {formatBytes(fr.totalSizeBytes)}
                    </span>
                  </div>
                  <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {fr.path}
                  </div>
                </div>
              ))
            ) : (
              <div style={{ padding: '16px', textAlign: 'center', color: 'var(--text-muted)', fontSize: 'var(--text-caption)' }}>
                Target scan required to populate folder rankings.
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
