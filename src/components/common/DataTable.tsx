import React, { useState } from 'react';
import { FileItem } from '../../types';
import { formatBytes } from '../../utils/format';
import { Badge } from './Badge';
import { useApp } from '../../context/AppContext';
import { Folder, FileCode, FileArchive, FileVideo, FileText, FileCheck, ArrowUpDown, Lock } from 'lucide-react';

interface DataTableProps {
  files: FileItem[];
  selectedFileId?: string;
  onSelectFile?: (file: FileItem) => void;
  selectedIds?: Set<string>;
  onToggleSelectId?: (id: string) => void;
  showCheckbox?: boolean;
}

type SortField = 'name' | 'size' | 'mtime' | 'category' | 'risk';

export const DataTable: React.FC<DataTableProps> = ({
  files,
  selectedFileId,
  onSelectFile,
  selectedIds,
  onToggleSelectId,
  showCheckbox = false,
}) => {
  const { openContextMenu } = useApp();
  const [sortField, setSortField] = useState<SortField>('size');
  const [sortAsc, setSortAsc] = useState<boolean>(false);

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortAsc((prev) => !prev);
    } else {
      setSortField(field);
      setSortAsc(false); // default descending
    }
  };

  const sortedFiles = [...files].sort((a, b) => {
    let cmp = 0;
    if (sortField === 'name') cmp = a.name.localeCompare(b.name);
    else if (sortField === 'size') cmp = a.sizeBytes - b.sizeBytes;
    else if (sortField === 'mtime') cmp = a.mtime.localeCompare(b.mtime);
    else if (sortField === 'category') cmp = a.category.localeCompare(b.category);
    else if (sortField === 'risk') cmp = a.riskTier.localeCompare(b.riskTier);
    return sortAsc ? cmp : -cmp;
  });

  const getFileIcon = (file: FileItem) => {
    if (file.isDirectory) return <Folder size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />;
    switch (file.category) {
      case 'Code':
        return <FileCode size={14} style={{ color: 'var(--accent-base)', flexShrink: 0 }} />;
      case 'Archives':
        return <FileArchive size={14} style={{ color: 'var(--warning-base)', flexShrink: 0 }} />;
      case 'Media':
        return <FileVideo size={14} style={{ color: 'var(--secondary-base)', flexShrink: 0 }} />;
      case 'Caches':
        return <FileCheck size={14} style={{ color: 'var(--safe-base)', flexShrink: 0 }} />;
      default:
        return <FileText size={14} style={{ color: 'var(--text-secondary)', flexShrink: 0 }} />;
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        width: '100%',
        backgroundColor: 'var(--bg-surface-0)',
        overflow: 'hidden',
        border: '1px solid var(--border-subtle)',
      }}
    >
      {/* Table Header */}
      <div
        role="row"
        style={{
          display: 'grid',
          gridTemplateColumns: showCheckbox
            ? '32px minmax(220px, 2fr) minmax(100px, 1fr) 95px 120px 110px'
            : 'minmax(240px, 2fr) minmax(100px, 1fr) 95px 120px 110px',
          alignItems: 'center',
          height: '28px',
          padding: '0 8px',
          backgroundColor: 'var(--bg-surface-1)',
          borderBottom: '1px solid var(--border-strong)',
          fontSize: 'var(--text-micro)',
          fontWeight: 600,
          color: 'var(--text-secondary)',
          textTransform: 'uppercase',
          letterSpacing: '0.04em',
        }}
      >
        {showCheckbox && <div role="columnheader" />}
        <button
          type="button"
          role="columnheader"
          aria-sort={sortField === 'name' ? (sortAsc ? 'ascending' : 'descending') : 'none'}
          onClick={() => handleSort('name')}
          style={{ display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer', justifySelf: 'start' }}
        >
          <span>Name</span>
          <ArrowUpDown size={10} />
        </button>
        <button
          type="button"
          role="columnheader"
          aria-sort={sortField === 'category' ? (sortAsc ? 'ascending' : 'descending') : 'none'}
          onClick={() => handleSort('category')}
          style={{ display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer', justifySelf: 'start' }}
        >
          <span>Category</span>
          <ArrowUpDown size={10} />
        </button>
        <button
          type="button"
          role="columnheader"
          aria-sort={sortField === 'size' ? (sortAsc ? 'ascending' : 'descending') : 'none'}
          onClick={() => handleSort('size')}
          style={{ display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer', justifySelf: 'end' }}
        >
          <span>Size</span>
          <ArrowUpDown size={10} />
        </button>
        <button
          type="button"
          role="columnheader"
          aria-sort={sortField === 'mtime' ? (sortAsc ? 'ascending' : 'descending') : 'none'}
          onClick={() => handleSort('mtime')}
          style={{ display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer', justifySelf: 'end' }}
        >
          <span>Modified</span>
          <ArrowUpDown size={10} />
        </button>
        <button
          type="button"
          role="columnheader"
          aria-sort={sortField === 'risk' ? (sortAsc ? 'ascending' : 'descending') : 'none'}
          onClick={() => handleSort('risk')}
          style={{ display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer', justifySelf: 'end' }}
        >
          <span>Safety Tier</span>
          <ArrowUpDown size={10} />
        </button>
      </div>

      {/* Table Body (Virtualized Scroll Area with Keyboard Navigation) */}
      <div
        tabIndex={0}
        role="rowgroup"
        aria-label="Files table"
        onKeyDown={(e) => {
          if (sortedFiles.length === 0) return;
          if (e.key === 'ArrowDown') {
            e.preventDefault();
            const currentIndex = sortedFiles.findIndex((f) => f.id === selectedFileId);
            if (currentIndex < sortedFiles.length - 1) {
              onSelectFile && onSelectFile(sortedFiles[currentIndex + 1]);
            } else if (currentIndex === -1) {
              onSelectFile && onSelectFile(sortedFiles[0]);
            }
          } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            const currentIndex = sortedFiles.findIndex((f) => f.id === selectedFileId);
            if (currentIndex > 0) {
              onSelectFile && onSelectFile(sortedFiles[currentIndex - 1]);
            }
          } else if (e.key === ' ' && selectedFileId) {
            e.preventDefault();
            onToggleSelectId && onToggleSelectId(selectedFileId);
          }
        }}
        style={{
          flex: 1,
          overflowY: 'auto',
          outline: 'none',
        }}
      >
        {sortedFiles.map((file) => {
          const isSelected = selectedFileId === file.id;
          const isChecked = selectedIds?.has(file.id);

          return (
            <div
              key={file.id}
              role="row"
              aria-selected={isSelected}
              onClick={() => onSelectFile && onSelectFile(file)}
              onContextMenu={(e) => {
                e.preventDefault();
                openContextMenu(e.clientX, e.clientY, file);
              }}
              style={{
                display: 'grid',
                gridTemplateColumns: showCheckbox
                  ? '32px minmax(220px, 2fr) minmax(100px, 1fr) 95px 120px 110px'
                  : 'minmax(240px, 2fr) minmax(100px, 1fr) 95px 120px 110px',
                alignItems: 'center',
                height: '28px',
                padding: '0 8px',
                borderBottom: '1px solid var(--border-subtle)',
                backgroundColor: isSelected
                  ? 'var(--accent-subtle)'
                  : 'transparent',
                cursor: 'pointer',
                fontSize: 'var(--text-caption)',
                color: 'var(--text-primary)',
              }}
              onMouseEnter={(e) => {
                if (!isSelected) e.currentTarget.style.backgroundColor = 'var(--bg-surface-hover)';
              }}
              onMouseLeave={(e) => {
                if (!isSelected) e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              {showCheckbox && (
                <div style={{ display: 'flex', alignItems: 'center' }}>
                  <input
                    type="checkbox"
                    checked={isChecked || false}
                    onChange={(e) => {
                      e.stopPropagation();
                      onToggleSelectId && onToggleSelectId(file.id);
                    }}
                    style={{ cursor: 'pointer' }}
                  />
                </div>
              )}

              {/* Name & Icon */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', minWidth: 0, paddingRight: '8px' }}>
                {getFileIcon(file)}
                <span className="truncate" style={{ fontWeight: isSelected ? 600 : 400 }} title={file.path}>
                  {file.name}
                </span>
                {file.isLocked && (
                  <span title={file.lockingProcess || 'Locked file'}>
                    <Lock size={11} style={{ color: 'var(--danger-base)' }} />
                  </span>
                )}
              </div>

              {/* Category */}
              <div style={{ color: 'var(--text-secondary)', fontSize: 'var(--text-micro)' }}>
                {file.category}
              </div>

              {/* Size */}
              <div className="tabular-nums" style={{ textAlign: 'right', fontWeight: 500 }}>
                {formatBytes(file.sizeBytes)}
              </div>

              {/* Modified Date */}
              <div className="tabular-nums" style={{ textAlign: 'right', color: 'var(--text-secondary)', fontSize: 'var(--text-micro)' }}>
                {file.mtime.split(' ')[0]}
              </div>

              {/* Safety Tier Badge */}
              <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                <Badge tier={file.riskTier} />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
