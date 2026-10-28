import React from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { Folder, Sparkles } from 'lucide-react';
import { DeveloperProject, DeveloperArtifact } from '@/types';

export const DeveloperCleanupScreen: React.FC = () => {
  const { developerProjects, cleanProjectArtifacts, showToast, setActiveScreen } = useApp();

  const totalReclaimable = developerProjects.reduce((acc: number, p: DeveloperProject) => acc + p.totalArtifactSizeBytes, 0);

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
            Developer Workspace & Build Cache Analyzer
          </h2>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Inspect dependency trees, compilation outputs, and virtual environments across detected repositories.
          </span>
        </div>

        <Button
          size="sm"
          variant="primary"
          icon={<Sparkles size={12} />}
          onClick={() => {
            showToast('Staged all safe inactive developer caches to Cleanup Plan');
            setActiveScreen('cleanup-planner');
          }}
        >
          Stage All Cleanable Artifacts
        </Button>
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
          Discovered <strong style={{ color: 'var(--text-primary)' }}>{developerProjects.length}</strong> active workspaces with build bloat.
        </span>
        <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--accent-base)' }}>
          Potential Reclamation: {formatBytes(totalReclaimable)}
        </span>
      </div>

      {/* Project Cards List */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {developerProjects.map((project: DeveloperProject) => {
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
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                  <Folder size={16} style={{ color: 'var(--accent-base)' }} />
                  <div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                        {project.name}
                      </span>
                      <Badge variant="neutral">{project.ecosystem}</Badge>
                      <Badge variant={isClean ? 'safe' : 'warning'}>
                        {isClean ? 'Git Clean' : 'Git Uncommitted Changes'}
                      </Badge>
                    </div>
                    <div className="font-mono truncate" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
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
                    onClick={() => cleanProjectArtifacts(project.id)}
                  >
                    Clean Artifacts
                  </Button>
                </div>
              </div>

              {/* Artifact Item Breakdown */}
              <div style={{ padding: '8px 14px', display: 'flex', flexDirection: 'column', gap: '6px' }}>
                {project.artifacts.map((art: DeveloperArtifact, idx: number) => (
                  <div
                    key={idx}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '4px 0',
                      borderBottom: idx < project.artifacts.length - 1 ? '1px dashed var(--border-subtle)' : 'none',
                      fontSize: 'var(--text-caption)',
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
                      <Badge tier={isClean ? 'safe' : 'caution'}>
                        {isClean ? 'Safe to Purge' : 'Review Required'}
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
