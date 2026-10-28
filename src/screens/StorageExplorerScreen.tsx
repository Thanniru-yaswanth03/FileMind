import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { FileCategory, FileItem } from '@/types';
import { DataTable } from '@/components/common/DataTable';
import { Breadcrumbs } from '@/components/common/Breadcrumbs';
import { Button } from '@/components/common/Button';
import { RefreshCw } from 'lucide-react';
import { formatBytes } from '@/utils/format';

export const StorageExplorerScreen: React.FC = () => {
  const { files, selectedFile, setSelectedFile, searchQuery, startScan } = useApp();
  const [currentPath, setCurrentPath] = useState<string>('D:\\Projects\\FileMind');
  const [selectedCategory, setSelectedCategory] = useState<FileCategory | 'All'>('All');

  const categories: (FileCategory | 'All')[] = ['All', 'Caches', 'Code', 'Media', 'Archives', 'Documents', 'Executables'];

  const filteredFiles = files.filter((f: FileItem) => {
    const matchesCategory = selectedCategory === 'All' || f.category === selectedCategory;
    const matchesSearch =
      searchQuery === '' ||
      f.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      f.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
      f.extension.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  const totalFilteredBytes = filteredFiles.reduce((acc: number, curr: FileItem) => acc + curr.sizeBytes, 0);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
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
            Filter:
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
        </div>

        {/* Refresh & Summary Stats */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
            Showing {filteredFiles.length} items ({formatBytes(totalFilteredBytes)})
          </span>
          <Button size="sm" variant="secondary" icon={<RefreshCw size={11} />} onClick={() => startScan()}>
            Re-scan Folder
          </Button>
        </div>
      </div>

      {/* Main Virtualized Data Table */}
      <div style={{ flex: 1, overflow: 'hidden', display: 'flex' }}>
        <DataTable
          files={filteredFiles}
          selectedFileId={selectedFile?.id}
          onSelectFile={(f: FileItem) => setSelectedFile(f)}
        />
      </div>
    </div>
  );
};
