import React, { useEffect, useState } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import {
  Copy,
  FolderOpen,
  ShieldCheck,
  ShieldAlert,
  CheckCircle2,
  RefreshCw,
  FileCheck2,
  Lock,
} from 'lucide-react';
import { RealDuplicateGroup, DuplicateFileItem } from '@/types';

export const DuplicateFinderScreen: React.FC = () => {
  const {
    realDuplicateGroups,
    isDuplicateScanning,
    duplicateScanResult,
    runDuplicateScan,
    verifyGroupByteByByte,
    selectRetainedCopy,
    openFileFolder,
    copyFilePath,
    scanSummary,
  } = useApp();

  const [minSizeFilter, setMinSizeFilter] = useState<number>(1024); // 1KB default
  const [verifyByteOption, setVerifyByteOption] = useState<boolean>(false);

  // Load duplicates on mount if not already populated
  useEffect(() => {
    if (realDuplicateGroups.length === 0 && !isDuplicateScanning) {
      runDuplicateScan(verifyByteOption);
    }
  }, []);

  const totalRecoverable =
    duplicateScanResult?.totalWastedBytes ??
    realDuplicateGroups.reduce((acc, g) => acc + g.wastedBytes, 0);

  const totalDupFiles =
    duplicateScanResult?.totalDuplicateFiles ??
    realDuplicateGroups.reduce((acc, g) => acc + g.files.length, 0);

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        overflow: 'hidden',
        padding: '14px',
        gap: '12px',
        backgroundColor: 'var(--bg-ground)',
      }}
    >
      {/* Header & Controls */}
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
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <h2
              style={{
                fontSize: 'var(--text-subhead)',
                fontWeight: 600,
                color: 'var(--text-primary)',
              }}
            >
              Duplicate Detection & Safety Review
            </h2>
            <Badge variant="accent">Phase 3 Native Pipeline</Badge>
          </div>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            3-Stage pipeline: size candidate grouping, streaming BLAKE3 cryptographic verification, and location-aware recommendations.
          </span>
        </div>

        {/* Action Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: 'var(--text-caption)' }}>
            <span style={{ color: 'var(--text-secondary)' }}>Min Size:</span>
            <select
              value={minSizeFilter}
              onChange={(e) => setMinSizeFilter(Number(e.target.value))}
              style={{
                padding: '4px 8px',
                fontSize: 'var(--text-caption)',
                backgroundColor: 'var(--bg-surface-1)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
              }}
            >
              <option value="1">All Sizes (&gt; 0 B)</option>
              <option value="1024">1 KB</option>
              <option value="1048576">1 MB</option>
              <option value="10485760">10 MB</option>
            </select>
          </label>

          <label
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              fontSize: 'var(--text-caption)',
              color: 'var(--text-secondary)',
              cursor: 'pointer',
            }}
          >
            <input
              type="checkbox"
              checked={verifyByteOption}
              onChange={(e) => setVerifyByteOption(e.target.checked)}
            />
            <span>Byte-for-Byte Verify</span>
          </label>

          <Button
            size="sm"
            variant="primary"
            disabled={isDuplicateScanning}
            icon={<RefreshCw size={12} className={isDuplicateScanning ? 'animate-spin' : ''} />}
            onClick={() => runDuplicateScan(verifyByteOption)}
          >
            {isDuplicateScanning ? 'Analyzing Hashes...' : 'Re-Run Detection'}
          </Button>
        </div>
      </div>

      {/* Safety Policy Notice Bar */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-1)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: 'var(--text-caption)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-secondary)' }}>
          <ShieldCheck size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
          <span>
            <strong>Safety Invariant:</strong> Duplicate review is strictly non-destructive. No files are moved, deleted, or altered.
          </span>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <span style={{ color: 'var(--text-muted)' }}>
            Verified Groups: <strong>{realDuplicateGroups.length}</strong> ({totalDupFiles} total copies)
          </span>
          <span className="tabular-nums" style={{ fontWeight: 600, color: 'var(--accent-base)' }}>
            Potential Recoverable Space: {formatBytes(totalRecoverable)}
          </span>
        </div>
      </div>

      {/* Main Content Area */}
      {isDuplicateScanning ? (
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '12px',
            backgroundColor: 'var(--bg-surface-0)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
          }}
        >
          <RefreshCw size={24} className="animate-spin" style={{ color: 'var(--accent-base)' }} />
          <div style={{ textAlign: 'center' }}>
            <h4 style={{ fontSize: 'var(--text-body)', fontWeight: 600, color: 'var(--text-primary)' }}>
              Streaming Cryptographic Verification
            </h4>
            <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)', maxWidth: '420px', marginTop: '4px' }}>
              Reading candidate blocks with bounded 64KB buffers and computing BLAKE3 hashes without exhausting system RAM.
            </p>
          </div>
        </div>
      ) : realDuplicateGroups.length === 0 ? (
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '12px',
            backgroundColor: 'var(--bg-surface-0)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
            padding: '24px',
          }}
        >
          <FileCheck2 size={36} style={{ color: 'var(--text-muted)' }} />
          <div style={{ textAlign: 'center' }}>
            <h4 style={{ fontSize: 'var(--text-body)', fontWeight: 600, color: 'var(--text-primary)' }}>
              No Duplicate Files Identified
            </h4>
            <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)', maxWidth: '460px', marginTop: '4px' }}>
              {scanSummary?.session?.sessionId
                ? 'All indexed files in the active session have distinct contents. Try adjusting the size filter or scanning additional roots.'
                : 'No files are currently indexed. Please run a filesystem scan from the Storage Explorer to index candidate folders.'}
            </p>
          </div>
        </div>
      ) : (
        <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {realDuplicateGroups.map((group: RealDuplicateGroup, gIdx: number) => {
            const hasProtectedFile = group.files.some((f) => f.isProtected);
            const isByteVerified = group.confidence === 'byte_for_byte_verified';

            return (
              <div
                key={group.groupId}
                style={{
                  backgroundColor: 'var(--bg-surface-0)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-md)',
                  overflow: 'hidden',
                }}
              >
                {/* Group Header */}
                <div
                  style={{
                    padding: '8px 12px',
                    backgroundColor: 'var(--bg-surface-1)',
                    borderBottom: '1px solid var(--border-subtle)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    flexWrap: 'wrap',
                    gap: '8px',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                    <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                      Duplicate Group #{gIdx + 1}
                    </span>
                    <Badge variant={isByteVerified ? 'accent' : 'warning'}>
                      {isByteVerified ? 'Byte-for-Byte Verified' : 'BLAKE3 Hash Match'}
                    </Badge>
                    {hasProtectedFile && (
                      <Badge variant="danger" title="Contains files inside protected Windows system or core app directories">
                        <ShieldAlert size={10} style={{ marginRight: '3px' }} /> Protected Directory
                      </Badge>
                    )}
                    {group.hash && (
                      <span
                        className="font-mono tabular-nums"
                        style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}
                        title={`Full cryptographic hash: ${group.hash}`}
                      >
                        Hash: {group.hash.slice(0, 16)}...
                      </span>
                    )}
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <span className="tabular-nums" style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)' }}>
                      Each: <strong>{formatBytes(group.sizeBytes)}</strong>
                    </span>
                    <span
                      className="tabular-nums"
                      style={{ fontSize: 'var(--text-caption)', color: 'var(--accent-base)', fontWeight: 600 }}
                    >
                      Recoverable: {formatBytes(group.wastedBytes)}
                    </span>
                    {!isByteVerified && (
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() =>
                          verifyGroupByteByByte(
                            group.groupId,
                            group.files.map((f) => f.path)
                          )
                        }
                      >
                        Verify Bitstream
                      </Button>
                    )}
                  </div>
                </div>

                {/* Group Rationale Explanation */}
                <div
                  style={{
                    padding: '6px 12px',
                    backgroundColor: 'var(--bg-surface-2)',
                    borderBottom: '1px solid var(--border-subtle)',
                    fontSize: 'var(--text-micro)',
                    color: 'var(--text-secondary)',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                  }}
                >
                  <CheckCircle2 size={12} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
                  <span>{group.groupRationale}</span>
                </div>

                {/* Copies Comparison Table */}
                <div style={{ display: 'flex', flexDirection: 'column' }}>
                  {group.files.map((file: DuplicateFileItem) => {
                    const isSelected = file.isSelectedForRetention;

                    return (
                      <div
                        key={file.id}
                        style={{
                          padding: '8px 12px',
                          borderBottom: '1px solid var(--border-subtle)',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          gap: '12px',
                          backgroundColor: isSelected ? 'var(--accent-subtle)' : 'transparent',
                        }}
                      >
                        {/* Selector & Path with Location Context */}
                        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flex: 1, minWidth: 0 }}>
                          <label
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: '6px',
                              cursor: 'pointer',
                              flexShrink: 0,
                            }}
                          >
                            <input
                              type="radio"
                              name={`retained-${group.groupId}`}
                              checked={isSelected}
                              onChange={() => selectRetainedCopy(group.groupId, file.id)}
                              style={{ cursor: 'pointer' }}
                            />
                            <span
                              style={{
                                fontSize: 'var(--text-micro)',
                                fontWeight: 600,
                                color: isSelected ? 'var(--accent-base)' : 'var(--text-secondary)',
                              }}
                            >
                              {isSelected ? 'KEEP COPY' : 'REDUNDANT'}
                            </span>
                          </label>

                          {/* Context Badges */}
                          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', flexShrink: 0 }}>
                            {file.isRecommendedKeep && (
                              <Badge variant="accent" title="Intelligently ranked as safest copy to preserve">
                                Recommended
                              </Badge>
                            )}
                            {file.isProtected && (
                              <Badge variant="danger" title="Protected operating system or application path">
                                <Lock size={9} style={{ marginRight: '2px' }} /> System Path
                              </Badge>
                            )}
                            {file.locationTags.includes('project_member') && (
                              <Badge variant="neutral">Project Repo</Badge>
                            )}
                            {file.locationTags.includes('backup_location') && (
                              <Badge variant="neutral">Backup Folder</Badge>
                            )}
                          </div>

                          {/* File Path & Rationale */}
                          <div style={{ minWidth: 0, flex: 1 }}>
                            <div
                              className="font-mono truncate"
                              style={{
                                color: 'var(--text-primary)',
                                fontSize: 'var(--text-caption)',
                                fontWeight: isSelected ? 500 : 400,
                              }}
                              title={file.path}
                            >
                              {file.path}
                            </div>
                            <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', marginTop: '2px' }}>
                              {file.recommendationRationale}
                            </div>
                          </div>
                        </div>

                        {/* Metadata & Actions */}
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flexShrink: 0 }}>
                          <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                            Modified: {file.formattedMtime}
                          </span>

                          <button
                            type="button"
                            onClick={() => openFileFolder(file.path)}
                            style={{
                              color: 'var(--text-secondary)',
                              padding: '3px 6px',
                              backgroundColor: 'transparent',
                              border: '1px solid var(--border-subtle)',
                              borderRadius: 'var(--radius-sm)',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '4px',
                              fontSize: 'var(--text-micro)',
                              cursor: 'pointer',
                            }}
                            title="Reveal containing folder in Windows Explorer"
                          >
                            <FolderOpen size={11} />
                            <span>Reveal</span>
                          </button>

                          <button
                            type="button"
                            onClick={() => copyFilePath(file.path)}
                            style={{
                              color: 'var(--text-secondary)',
                              padding: '3px 6px',
                              backgroundColor: 'transparent',
                              border: '1px solid var(--border-subtle)',
                              borderRadius: 'var(--radius-sm)',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '4px',
                              fontSize: 'var(--text-micro)',
                              cursor: 'pointer',
                            }}
                            title="Copy exact canonical path to clipboard"
                          >
                            <Copy size={11} />
                            <span>Copy Path</span>
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
