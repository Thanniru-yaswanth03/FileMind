import React, { useState, useRef, useEffect } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes, copyToClipboard } from '@/utils/format';
import { Badge } from '../common/Badge';
import { Button } from '../common/Button';
import {
  FileText,
  Copy,
  PlusCircle,
  ShieldAlert,
  Lock,
  X,
  Hash,
  HardDrive,
  FolderOpen,
  CheckCircle2,
  XCircle,
} from 'lucide-react';
import { evaluateSafety } from '@/services/api';
import { SafetyAssessment } from '@/types';

export const PropertyInspector: React.FC = () => {
  const {
    selectedFile,
    isInspectorOpen,
    setIsInspectorOpen,
    inspectorWidth,
    setInspectorWidth,
    showToast,
    setActiveScreen,
    openFileFolder,
    copyFilePath,
  } = useApp();
  const [isResizing, setIsResizing] = useState(false);
  const [liveAssessment, setLiveAssessment] = useState<SafetyAssessment | null>(null);
  const startXRef = useRef(0);
  const startWidthRef = useRef(inspectorWidth);

  useEffect(() => {
    if (selectedFile?.path) {
      evaluateSafety([selectedFile.path], 'evaluate')
        .then((resp) => {
          if (resp.assessments.length > 0) {
            setLiveAssessment(resp.assessments[0]);
          }
        })
        .catch(() => setLiveAssessment(null));
    } else {
      setLiveAssessment(null);
    }
  }, [selectedFile?.path]);

  if (!isInspectorOpen) return null;

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsResizing(true);
    startXRef.current = e.clientX;
    startWidthRef.current = inspectorWidth;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const delta = startXRef.current - moveEvent.clientX;
      const newWidth = Math.max(260, Math.min(500, startWidthRef.current + delta));
      setInspectorWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  return (
    <aside
      style={{
        width: `${inspectorWidth}px`,
        height: '100%',
        backgroundColor: 'var(--bg-surface-0)',
        borderLeft: '1px solid var(--border-subtle)',
        display: 'flex',
        flexDirection: 'column',
        flexShrink: 0,
        position: 'relative',
        userSelect: isResizing ? 'none' : 'auto',
      }}
    >
      {/* Drag Resize Handle */}
      <div
        onMouseDown={handleMouseDown}
        style={{
          position: 'absolute',
          left: '-3px',
          top: 0,
          bottom: 0,
          width: '6px',
          cursor: 'col-resize',
          zIndex: 10,
        }}
        title="Drag to resize inspector panel"
      />

      {/* Inspector Header */}
      <div
        style={{
          height: '40px',
          padding: '0 12px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          borderBottom: '1px solid var(--border-subtle)',
          backgroundColor: 'var(--bg-surface-1)',
        }}
      >
        <span style={{ fontSize: 'var(--text-ui)', fontWeight: 600, color: 'var(--text-primary)' }}>
          Property Inspector
        </span>
        <button
          type="button"
          onClick={() => setIsInspectorOpen(false)}
          title="Close Inspector"
          style={{ padding: '4px', color: 'var(--text-secondary)' }}
        >
          <X size={14} />
        </button>
      </div>

      {/* Content Body */}
      {selectedFile ? (
        <div style={{ flex: 1, overflowY: 'auto', padding: '14px', display: 'flex', flexDirection: 'column', gap: '14px' }}>
          {/* File Title & Category */}
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginBottom: '4px' }}>
              <FileText size={16} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
              <span style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }} className="truncate">
                {selectedFile.name}
              </span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <Badge variant="neutral">{selectedFile.category}</Badge>
              {selectedFile.extension && <Badge variant="neutral">.{selectedFile.extension}</Badge>}
              <Badge tier={selectedFile.riskTier} />
            </div>
          </div>

          {/* Full Path with Copy Button */}
          <div
            style={{
              padding: '8px 10px',
              backgroundColor: 'var(--bg-surface-1)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-sm)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '4px' }}>
              <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                Full Canonical Path
              </span>
              <button
                type="button"
                onClick={async () => {
                  await copyToClipboard(selectedFile.path);
                  showToast('Path copied to clipboard');
                }}
                style={{ color: 'var(--text-secondary)', padding: '2px' }}
                title="Copy Path"
              >
                <Copy size={11} />
              </button>
            </div>
            <div className="font-mono selectable" style={{ wordBreak: 'break-all', color: 'var(--text-primary)' }}>
              {selectedFile.path}
            </div>
          </div>

          {/* Storage & Allocation Metrics */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
              Storage Allocation
            </span>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: '1fr 1fr',
                gap: '8px',
                padding: '8px 10px',
                backgroundColor: 'var(--bg-surface-1)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
              }}
            >
              <div>
                <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Logical Size</div>
                <div className="tabular-nums" style={{ fontWeight: 600, fontSize: 'var(--text-body)', color: 'var(--text-primary)' }}>
                  {formatBytes(selectedFile.sizeBytes)}
                </div>
              </div>
              <div>
                <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Size on Disk</div>
                <div className="tabular-nums" style={{ fontWeight: 600, fontSize: 'var(--text-body)', color: 'var(--text-primary)' }}>
                  {formatBytes(selectedFile.physicalSizeBytes)}
                </div>
              </div>
            </div>
          </div>

          {/* Safety & Evidence Analysis */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                Safety & Dependency Evidence
              </span>
              {liveAssessment && (
                <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600 }}>
                  {liveAssessment.isPermitted ? (
                    <span style={{ color: 'var(--safe-base)', display: 'inline-flex', alignItems: 'center', gap: '3px' }}>
                      <CheckCircle2 size={11} /> Permitted
                    </span>
                  ) : (
                    <span style={{ color: 'var(--danger-base)', display: 'inline-flex', alignItems: 'center', gap: '3px' }}>
                      <XCircle size={11} /> Blocked
                    </span>
                  )}
                </span>
              )}
            </div>
            <div
              style={{
                padding: '10px',
                backgroundColor: 'var(--bg-surface-1)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                display: 'flex',
                flexDirection: 'column',
                gap: '6px',
              }}
            >
              <div style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', lineHeight: 1.4 }}>
                {liveAssessment?.blockReason || selectedFile.safetyRationale}
              </div>

              {liveAssessment && (
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginTop: '2px', flexWrap: 'wrap' }}>
                  <Badge variant="neutral">
                    {liveAssessment.classification.replace(/([A-Z])/g, ' $1').trim()}
                  </Badge>
                  <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                    Recovery: {liveAssessment.recoveryPossibility}
                  </span>
                </div>
              )}

              {selectedFile.isLocked && (
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    fontSize: 'var(--text-micro)',
                    color: 'var(--danger-base)',
                    fontWeight: 500,
                  }}
                >
                  <Lock size={12} />
                  <span>Locked by: {selectedFile.lockingProcess}</span>
                </div>
              )}
            </div>
          </div>

          {/* Cryptographic & NTFS Signatures */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
              Cryptographic Integrity
            </span>
            <div
              style={{
                padding: '8px 10px',
                backgroundColor: 'var(--bg-surface-1)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                display: 'flex',
                flexDirection: 'column',
                gap: '4px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>BLAKE3 Hash</span>
                {selectedFile.blake3Hash && (
                  <button
                    type="button"
                    onClick={async () => {
                      if (selectedFile.blake3Hash) {
                        await copyToClipboard(selectedFile.blake3Hash);
                        showToast('BLAKE3 hash copied');
                      }
                    }}
                    style={{ color: 'var(--text-secondary)', padding: '2px' }}
                    title="Copy Hash"
                  >
                    <Hash size={11} />
                  </button>
                )}
              </div>
              <div className="font-mono truncate selectable" style={{ color: 'var(--text-primary)' }}>
                {selectedFile.blake3Hash || 'Computed on demand'}
              </div>
            </div>
          </div>

          {/* Timestamps */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
              Timestamps (Local Time)
            </span>
            <div
              style={{
                fontSize: 'var(--text-micro)',
                color: 'var(--text-secondary)',
                display: 'flex',
                flexDirection: 'column',
                gap: '4px',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>Last Modified:</span>
                <span className="tabular-nums" style={{ color: 'var(--text-primary)' }}>{selectedFile.mtime}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>Created:</span>
                <span className="tabular-nums" style={{ color: 'var(--text-primary)' }}>{selectedFile.ctime}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>Last Accessed:</span>
                <span className="tabular-nums" style={{ color: 'var(--text-primary)' }}>{selectedFile.atime}</span>
              </div>
            </div>
          </div>

          {/* Action Ribbon */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', marginTop: 'auto', paddingTop: '12px' }}>
            <Button
              variant="primary"
              icon={<PlusCircle size={13} />}
              onClick={() => {
                showToast(`Staged "${selectedFile.name}" into Cleanup Plan`);
                setActiveScreen('cleanup-planner');
              }}
            >
              Stage for Cleanup
            </Button>
            <Button
              variant="secondary"
              icon={<ShieldAlert size={13} />}
              onClick={() => showToast(`Added exclusion rule for "${selectedFile.name}"`)}
            >
              Exclude from Scans
            </Button>
            <Button
              variant="secondary"
              icon={<FolderOpen size={13} />}
              onClick={() => openFileFolder(selectedFile.path)}
            >
              Reveal in File Explorer
            </Button>
            <Button
              variant="secondary"
              icon={<Copy size={13} />}
              onClick={() => copyFilePath(selectedFile.path)}
            >
              Copy Full Path
            </Button>
          </div>
        </div>
      ) : (
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '24px', textAlign: 'center', color: 'var(--text-muted)' }}>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '8px' }}>
            <HardDrive size={24} style={{ opacity: 0.5 }} />
            <div style={{ fontSize: 'var(--text-caption)' }}>Select any file row to inspect metadata and safety scoring.</div>
          </div>
        </div>
      )}
    </aside>
  );
};
