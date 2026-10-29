import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { FileCategory, FileItem } from '@/types';
import { DataTable } from '@/components/common/DataTable';
import { Breadcrumbs } from '@/components/common/Breadcrumbs';
import { Button } from '@/components/common/Button';
import { RefreshCw, Play, XCircle, AlertTriangle, Folder, HardDrive, Copy, ExternalLink, Filter, Layers } from 'lucide-react';
import { formatBytes } from '@/utils/format';

export const StorageExplorerScreen: React.FC = () => {
  const {
    files,
    selectedFile,
    setSelectedFile,
    searchQuery,
    startScan,
    cancelScan,
    isScanning,
    scanProgress,
    scanSummary,
    scanErrors,
    folderRankings,
    openFileFolder,
    copyFilePath,
    customScanPath,
    setCustomScanPath,
    reconcileIndex,
    isBackendConnected,
  } = useApp();

  const [currentPath, setCurrentPath] = useState<string>(customScanPath || 'C:\\');
  const [selectedCategory, setSelectedCategory] = useState<FileCategory | 'All'>('All');
  const [sizeFilter, setSizeFilter] = useState<'all' | '10mb' | '100mb' | '1gb'>('all');
  const [showRankings, setShowRankings] = useState<boolean>(false);
  const [showErrorDetails, setShowErrorDetails] = useState<boolean>(false);

  const categories: (FileCategory | 'All')[] = [
    'All',
    'Caches',
    'Code',
    'Media',
    'Archives',
    'Documents',
    'Executables',
    'Other',
  ];

  const filteredFiles = files.filter((f: FileItem) => {
    const matchesCategory = selectedCategory === 'All' || f.category === selectedCategory;
    const matchesPath = currentPath === '' || f.path.toLowerCase().startsWith(currentPath.toLowerCase());
    const matchesSearch =
      searchQuery === '' ||
      f.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      f.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
      f.extension.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesSize =
      sizeFilter === 'all'
        ? true
        : sizeFilter === '10mb'
        ? f.sizeBytes >= 10 * 1024 * 1024
        : sizeFilter === '100mb'
        ? f.sizeBytes >= 100 * 1024 * 1024
        : f.sizeBytes >= 1024 * 1024 * 1024;

    return matchesCategory && matchesPath && matchesSearch && matchesSize;
  });

  const totalFilteredBytes = filteredFiles.reduce((acc: number, curr: FileItem) => acc + curr.sizeBytes, 0);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Root Selector & Live Scan Control Bar */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-1)',
          borderBottom: '1px solid var(--border-subtle)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: '8px',
          flexShrink: 0,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
          <HardDrive size={15} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />
          <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-secondary)' }}>
            Target Root:
          </span>
          <input
            type="text"
            value={customScanPath}
            onChange={(e) => {
              setCustomScanPath(e.target.value);
              setCurrentPath(e.target.value);
            }}
            placeholder="Enter directory path (e.g. C:\ or D:\Projects)"
            style={{
              padding: '4px 8px',
              borderRadius: 'var(--radius-sm)',
              border: '1px solid var(--border-base)',
              backgroundColor: 'var(--bg-surface-0)',
              color: 'var(--text-primary)',
              fontSize: 'var(--text-caption)',
              fontFamily: 'monospace',
              flex: 1,
              maxWidth: '380px',
            }}
          />
          {isScanning ? (
            <Button size="sm" variant="danger" icon={<XCircle size={12} />} onClick={() => cancelScan()}>
              Cancel Scan
            </Button>
          ) : (
            <Button size="sm" variant="primary" icon={<Play size={12} />} onClick={() => startScan(customScanPath)}>
              Scan Target
            </Button>
          )}

          <Button
            size="sm"
            variant="secondary"
            icon={<RefreshCw size={11} />}
            onClick={() => reconcileIndex()}
            title="Perform conservative incremental reconciliation without full rescanning"
          >
            Reconcile
          </Button>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <Button
            size="sm"
            variant={showRankings ? 'primary' : 'secondary'}
            icon={<Layers size={11} />}
            onClick={() => setShowRankings(!showRankings)}
          >
            Folder Rankings
          </Button>

          {isBackendConnected ? (
            <span
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '4px',
                fontSize: 'var(--text-micro)',
                color: 'var(--accent-base)',
                backgroundColor: 'rgba(74, 107, 91, 0.1)',
                padding: '2px 6px',
                borderRadius: 'var(--radius-sm)',
                fontWeight: 600,
              }}
            >
              ● Native Core Live
            </span>
          ) : (
            <span
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '4px',
                fontSize: 'var(--text-micro)',
                color: 'var(--risk-caution-text)',
                backgroundColor: 'var(--risk-caution-bg)',
                padding: '2px 6px',
                borderRadius: 'var(--radius-sm)',
                fontWeight: 600,
              }}
            >
              Standalone UI Mode
            </span>
          )}
        </div>
      </div>

      {/* Live Scan Progress Banner */}
      {isScanning && (
        <div
          style={{
            padding: '8px 12px',
            backgroundColor: 'var(--bg-surface-0)',
            borderBottom: '1px solid var(--accent-base)',
            display: 'flex',
            flexDirection: 'column',
            gap: '4px',
            flexShrink: 0,
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 'var(--text-caption)', fontWeight: 600, color: 'var(--text-primary)' }}>
              Scanning Active: {scanProgress.filesCount.toLocaleString()} files indexed ({formatBytes(scanProgress.bytesCount)})
            </span>
            <span style={{ fontSize: 'var(--text-caption)', fontWeight: 700, color: 'var(--accent-base)' }}>
              {scanProgress.percent.toFixed(1)}%
            </span>
          </div>

          {/* Progress track */}
          <div
            style={{
              width: '100%',
              height: '4px',
              backgroundColor: 'var(--border-subtle)',
              borderRadius: '2px',
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                width: `${scanProgress.percent}%`,
                height: '100%',
                backgroundColor: 'var(--accent-base)',
                transition: 'width 150ms ease-in-out',
              }}
            />
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
            <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '85%' }}>
              Path: {scanProgress.currentPath}
            </span>
            {scanProgress.errorsCount > 0 && (
              <span style={{ color: 'var(--risk-high-text)', fontWeight: 600 }}>
                {scanProgress.errorsCount} inaccessible paths
              </span>
            )}
          </div>
        </div>
      )}

      {/* Partial Scan / Inaccessible Paths Warning Banner */}
      {scanErrors.length > 0 && !isScanning && (
        <div
          style={{
            padding: '6px 12px',
            backgroundColor: 'var(--risk-caution-bg)',
            borderBottom: '1px solid var(--risk-caution-border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: '8px',
            fontSize: 'var(--text-caption)',
            color: 'var(--risk-caution-text)',
            flexShrink: 0,
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <AlertTriangle size={14} />
            <span>
              <strong>Partial Scan Notice:</strong> {scanErrors.length} path(s) could not be accessed due to system permissions or sharing locks.
            </span>
          </div>
          <button
            type="button"
            onClick={() => setShowErrorDetails(!showErrorDetails)}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--risk-caution-text)',
              textDecoration: 'underline',
              cursor: 'pointer',
              fontWeight: 600,
              fontSize: 'var(--text-caption)',
            }}
          >
            {showErrorDetails ? 'Hide Details' : 'View Errors'}
          </button>
        </div>
      )}

      {/* Inaccessible Path Details Drawer */}
      {showErrorDetails && scanErrors.length > 0 && (
        <div
          style={{
            maxHeight: '130px',
            overflowY: 'auto',
            padding: '8px 12px',
            backgroundColor: 'var(--bg-surface-0)',
            borderBottom: '1px solid var(--border-base)',
            fontSize: 'var(--text-micro)',
            fontFamily: 'monospace',
            flexShrink: 0,
          }}
        >
          <div style={{ fontWeight: 600, marginBottom: '4px', color: 'var(--text-secondary)' }}>
            Inaccessible Directory Log (Read-Only Scanner Policy Enforced):
          </div>
          {scanErrors.map((err) => (
            <div key={err.id} style={{ display: 'flex', gap: '8px', padding: '2px 0', borderBottom: '1px solid var(--border-subtle)' }}>
              <span style={{ color: 'var(--risk-caution-text)', fontWeight: 600 }}>[{err.errorKind}]</span>
              <span style={{ color: 'var(--text-primary)' }}>{err.path}</span>
              <span style={{ color: 'var(--text-muted)' }}>— {err.message}</span>
            </div>
          ))}
        </div>
      )}

      {/* Folder Breadcrumb Navigation */}
      <Breadcrumbs currentPath={currentPath} onNavigate={(path: string) => setCurrentPath(path)} />

      {/* Filter and Control Ribbon */}
      <div
        style={{
          padding: '6px 12px',
          backgroundColor: 'var(--bg-surface-0)',
          borderBottom: '1px solid var(--border-subtle)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: '8px',
          flexShrink: 0,
        }}
      >
        {/* Category Filter Chips */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px', overflowX: 'auto' }}>
          <span style={{ fontSize: 'var(--text-micro)', fontWeight: 600, color: 'var(--text-muted)', textTransform: 'uppercase', marginRight: '4px' }}>
            Category:
          </span>
          {categories.map((cat) => {
            const isSelected = selectedCategory === cat;
            return (
              <button
                key={cat}
                type="button"
                onClick={() => setSelectedCategory(cat)}
                style={{
                  padding: '2px 8px',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: 'var(--text-micro)',
                  fontWeight: isSelected ? 600 : 400,
                  backgroundColor: isSelected ? 'var(--accent-base)' : 'var(--bg-surface-1)',
                  color: isSelected ? 'var(--accent-contrast)' : 'var(--text-secondary)',
                  border: `1px solid ${isSelected ? 'var(--accent-base)' : 'var(--border-subtle)'}`,
                  cursor: 'pointer',
                  whiteSpace: 'nowrap',
                }}
              >
                {cat}
              </button>
            );
          })}

          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginLeft: '8px' }}>
            <Filter size={11} style={{ color: 'var(--text-muted)' }} />
            <select
              value={sizeFilter}
              onChange={(e) => setSizeFilter(e.target.value as any)}
              style={{
                fontSize: 'var(--text-micro)',
                padding: '2px 6px',
                borderRadius: 'var(--radius-sm)',
                border: '1px solid var(--border-subtle)',
                backgroundColor: 'var(--bg-surface-1)',
                color: 'var(--text-secondary)',
              }}
            >
              <option value="all">All Sizes</option>
              <option value="10mb">≥ 10 MB</option>
              <option value="100mb">≥ 100 MB</option>
              <option value="1gb">≥ 1 GB</option>
            </select>
          </div>
        </div>

        {/* Selected File Quick Actions & Summary Stats */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          {selectedFile && (
            <div style={{ display: 'flex', gap: '4px' }}>
              <Button
                size="sm"
                variant="ghost"
                icon={<Copy size={11} />}
                onClick={() => copyFilePath(selectedFile.path)}
                title="Copy full path to clipboard"
              >
                Copy Path
              </Button>
              <Button
                size="sm"
                variant="ghost"
                icon={<ExternalLink size={11} />}
                onClick={() => openFileFolder(selectedFile.path)}
                title="Open containing folder in Windows Explorer"
              >
                Open Folder
              </Button>
            </div>
          )}

          <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
            Showing {filteredFiles.length} items ({formatBytes(totalFilteredBytes)})
          </span>
        </div>
      </div>

      {/* Main Content Area: Split with Folder Rankings if open */}
      <div style={{ flex: 1, overflow: 'hidden', display: 'flex' }}>
        {/* Main Virtualized Data Table */}
        <div style={{ flex: 1, overflow: 'hidden', display: 'flex' }}>
          <DataTable
            files={filteredFiles}
            selectedFileId={selectedFile?.id}
            onSelectFile={(f: FileItem) => setSelectedFile(f)}
          />
        </div>

        {/* Folder Size Rankings Side Panel */}
        {showRankings && (
          <div
            style={{
              width: '280px',
              borderLeft: '1px solid var(--border-base)',
              backgroundColor: 'var(--bg-surface-1)',
              display: 'flex',
              flexDirection: 'column',
              overflow: 'hidden',
              flexShrink: 0,
            }}
          >
            <div
              style={{
                padding: '8px 12px',
                borderBottom: '1px solid var(--border-subtle)',
                fontWeight: 600,
                fontSize: 'var(--text-caption)',
                color: 'var(--text-primary)',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <span>Folder Size Rankings</span>
              <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>Top Space</span>
            </div>

            <div style={{ flex: 1, overflowY: 'auto', padding: '6px 8px' }}>
              {folderRankings.length === 0 ? (
                <div style={{ padding: '16px', textAlign: 'center', color: 'var(--text-muted)', fontSize: 'var(--text-caption)' }}>
                  No ranking data available yet. Run a target scan to populate.
                </div>
              ) : (
                folderRankings.map((fr, idx) => (
                  <div
                    key={fr.path}
                    onClick={() => setCurrentPath(fr.path)}
                    style={{
                      padding: '6px 8px',
                      marginBottom: '4px',
                      borderRadius: 'var(--radius-sm)',
                      backgroundColor: 'var(--bg-surface-0)',
                      border: '1px solid var(--border-subtle)',
                      cursor: 'pointer',
                      fontSize: 'var(--text-micro)',
                    }}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2px' }}>
                      <span style={{ fontWeight: 600, color: 'var(--text-primary)', display: 'flex', alignItems: 'center', gap: '4px' }}>
                        <Folder size={11} style={{ color: 'var(--accent-base)' }} />
                        #{idx + 1} {fr.path.split('\\').pop() || fr.path}
                      </span>
                      <span className="tabular-nums" style={{ fontWeight: 700, color: 'var(--text-secondary)' }}>
                        {formatBytes(fr.totalSizeBytes)}
                      </span>
                    </div>

                    <div style={{ width: '100%', height: '3px', backgroundColor: 'var(--border-subtle)', borderRadius: '2px', overflow: 'hidden' }}>
                      <div
                        style={{
                          width: `${Math.min(fr.percentageOfTotal, 100)}%`,
                          height: '100%',
                          backgroundColor: 'var(--accent-base)',
                        }}
                      />
                    </div>

                    <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text-muted)', marginTop: '2px' }}>
                      <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '170px' }}>
                        {fr.path}
                      </span>
                      <span className="tabular-nums">{fr.percentageOfTotal}%</span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}
      </div>

      {/* Storage Accounting Footer Bar */}
      {scanSummary?.storageAccounting && (
        <div
          style={{
            padding: '4px 12px',
            backgroundColor: 'var(--bg-surface-0)',
            borderTop: '1px solid var(--border-subtle)',
            fontSize: 'var(--text-micro)',
            color: 'var(--text-muted)',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            flexShrink: 0,
          }}
        >
          <span>
            {scanSummary.storageAccounting.exactFiguresNote}
          </span>
          <span style={{ fontStyle: 'italic' }}>
            Allocated: {formatBytes(scanSummary.storageAccounting.physicalSizeBytes)} ({scanSummary.storageAccounting.hardlinksDeduplicatedCount} hardlinks detected)
          </span>
        </div>
      )}
    </div>
  );
};
