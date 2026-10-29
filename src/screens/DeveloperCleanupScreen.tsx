import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import {
  Folder,
  Sparkles,
  RefreshCw,
  GitBranch,
  ShieldCheck,
  Box,
} from 'lucide-react';
import type { DeveloperProject, DeveloperArtifact } from '@/types';

export const DeveloperCleanupScreen: React.FC = () => {
  const {
    developerProjects,
    cleanDeveloperWorkspaces,
    fetchDeveloperWorkspaces,
    isScanningDeveloperProjects,
    showToast,
    customScanPath,
  } = useApp();

  const [scanPathInput, setScanPathInput] = useState<string>(customScanPath || 'C:\\');

  const totalReclaimable = developerProjects.reduce(
    (acc: number, p: DeveloperProject) => acc + (p.totalArtifactSizeBytes || 0),
    0
  );

  const handleRescan = async () => {
    showToast(`Scanning developer workspaces in ${scanPathInput}...`);
    await fetchDeveloperWorkspaces(scanPathInput);
  };

  const handleStageAll = async () => {
    const projectIds = developerProjects.map((p) => p.id);
    if (projectIds.length === 0) {
      showToast('No developer projects detected to clean.');
      return;
    }
    await cleanDeveloperWorkspaces(projectIds);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '14px' }}>
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
            Developer Workspace & Build Cache Intelligence
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Grounded manifest and lockfile analyzer identifying regenerable compilation outputs, dependency trees, and virtual environments.
          </span>
        </div>

        <div style={{ display: 'flex', gap: '8px' }}>
          <Button
            size="sm"
            variant="secondary"
            icon={<RefreshCw size={12} className={isScanningDeveloperProjects ? 'animate-spin' : ''} />}
            onClick={handleRescan}
            disabled={isScanningDeveloperProjects}
          >
            Rescan Workspaces
          </Button>

          <Button
            size="sm"
            variant="primary"
            icon={<Sparkles size={12} />}
            onClick={handleStageAll}
            disabled={developerProjects.length === 0}
          >
            Stage All Cleanable Artifacts
          </Button>
        </div>
      </div>

      {/* Scope Selector Ribbon */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        }}
      >
        <Folder size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
        <span style={{ fontSize: 'var(--text-caption)', fontWeight: 500 }}>Scan Root:</span>
        <input
          type="text"
          value={scanPathInput}
          onChange={(e) => setScanPathInput(e.target.value)}
          placeholder="Path to scan (e.g. C:\Projects\Workspace)"
          style={{ flex: 1, padding: '4px 8px', fontSize: 'var(--text-caption)' }}
        />
        <Button size="sm" variant="secondary" onClick={handleRescan} disabled={isScanningDeveloperProjects}>
          Scan
        </Button>
      </div>

      {/* Safety Invariant Banner */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          fontSize: 'var(--text-micro)',
          color: 'var(--text-secondary)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        }}
      >
        <ShieldCheck size={16} style={{ color: 'var(--status-safe-fg)', flexShrink: 0 }} />
        <span>
          <strong>Supply-Chain Safety Invariant:</strong> FileMind inspects actual package manifests and lockfiles. Source code files, credentials (.env, keys), and internal Git histories (.git) are strictly blocked from automated deletion. All purged items are first staged to the per-volume Quarantine Vault.
        </span>
      </div>

      {/* Summary Stats */}
      <div
        style={{
          padding: '10px 14px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: 'var(--text-caption)',
        }}
      >
        <span style={{ color: 'var(--text-secondary)' }}>
          Discovered <strong style={{ color: 'var(--text-primary)' }}>{developerProjects.length}</strong> active workspaces with build artifacts.
        </span>
        <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--accent-base)' }}>
          Total Cleanable Footprint: {formatBytes(totalReclaimable)}
        </span>
      </div>

      {/* Project Cards List */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {developerProjects.length === 0 ? (
          <div style={{ padding: '32px', textAlign: 'center', color: 'var(--text-muted)', backgroundColor: 'var(--bg-surface-0)', borderRadius: 'var(--radius-md)', border: '1px solid var(--border-subtle)' }}>
            <Box size={32} style={{ margin: '0 auto 8px', opacity: 0.5 }} />
            <div>No developer projects found in the selected folder.</div>
            <div style={{ fontSize: 'var(--text-micro)', marginTop: '4px' }}>
              Select a project folder containing package.json, Cargo.toml, pyproject.toml, or go.mod and click Scan.
            </div>
          </div>
        ) : (
          developerProjects.map((project: DeveloperProject) => {
            const isClean = project.gitStatus === 'clean';

            return (
              <div
                key={project.id}
                style={{
                  backgroundColor: 'var(--bg-surface-0)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-md)',
                  overflow: 'hidden',
                }}
              >
                {/* Project Card Header */}
                <div
                  style={{
                    padding: '10px 14px',
                    backgroundColor: 'var(--bg-surface-1)',
                    borderBottom: '1px solid var(--border-subtle)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    flexWrap: 'wrap',
                    gap: '8px',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <Folder size={16} style={{ color: 'var(--accent-base)' }} />
                    <div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap' }}>
                        <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                          {project.name}
                        </span>
                        <Badge variant="accent">{project.ecosystem}</Badge>
                        <Badge variant={isClean ? 'safe' : 'caution'}>
                          <GitBranch size={11} style={{ marginRight: '3px' }} />
                          {isClean ? 'Git Clean' : 'Uncommitted Work'}
                        </Badge>
                        {(project as any).manifestFile && (
                          <span className="font-mono" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                            {(project as any).manifestFile}
                            {(project as any).lockfileFile ? ` (${(project as any).lockfileFile})` : ' [No Lockfile]'}
                          </span>
                        )}
                      </div>
                      <div className="font-mono truncate" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }} title={project.path}>
                        {project.path}
                      </div>
                    </div>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <span className="tabular-nums" style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)' }}>
                      Artifacts: <strong style={{ color: 'var(--text-primary)' }}>{formatBytes(project.totalArtifactSizeBytes)}</strong>
                    </span>
                    <Button
                      size="sm"
                      variant={isClean ? 'primary' : 'secondary'}
                      onClick={() => cleanDeveloperWorkspaces([project.id])}
                    >
                      Stage for Cleanup
                    </Button>
                  </div>
                </div>

                {/* Artifact Item Breakdown */}
                <div style={{ padding: '8px 14px', display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  {project.artifacts.map((art: DeveloperArtifact, idx: number) => {
                    const isSafe = isClean && (art.isDisposable || (art as any).riskTier === 'safe');

                    return (
                      <div
                        key={idx}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          padding: '6px 0',
                          borderBottom: idx < project.artifacts.length - 1 ? '1px dashed var(--border-subtle)' : 'none',
                          fontSize: 'var(--text-caption)',
                          flexWrap: 'wrap',
                          gap: '6px',
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                          <code className="font-mono" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                            {art.name}
                          </code>
                          <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                            ({art.category})
                          </span>
                          <span className="font-mono" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                            {art.relativePath}
                          </span>
                        </div>

                        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                          <span className="tabular-nums" style={{ fontWeight: 500, color: 'var(--text-secondary)' }}>
                            {formatBytes(art.sizeBytes)}
                          </span>
                          <Badge tier={isSafe ? 'safe' : 'caution'}>
                            {isSafe ? 'Safe to Purge' : 'Review Required'}
                          </Badge>
                          {(art as any).cleanupProcedure && (
                            <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                              Reversible via Quarantine Vault
                            </span>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
