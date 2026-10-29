import React, { useState, useEffect } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import * as api from '@/services/api';
import {
  Search,
  FileText,
  FolderOpen,
  Copy,
  ShieldCheck,
  Trash2,
  RefreshCw,
  FileCheck2,
  AlertTriangle,
} from 'lucide-react';
import { FileItem, ContentSearchResultItem } from '@/types';

export const FileSearchScreen: React.FC = () => {
  const {
    openFileFolder,
    copyFilePath,
    showToast,
    contentStatus,
    refreshContentStatus,
    indexDocumentContent,
    clearContentIndex,
    setSelectedFile,
    setIsInspectorOpen,
  } = useApp();

  const [searchMode, setSearchMode] = useState<'metadata' | 'content'>('metadata');
  const [searchTerm, setSearchTerm] = useState<string>('');
  const [categoryFilter, setCategoryFilter] = useState<string>('All');
  const [extensionFilter, setExtensionFilter] = useState<string>('');
  const [minSizeKb, setMinSizeKb] = useState<string>('');
  const [maxSizeKb, setMaxSizeKb] = useState<string>('');
  const [sortBy, setSortBy] = useState<'size' | 'name' | 'mtime'>('size');
  const [sortOrder, setSortOrder] = useState<'desc' | 'asc'>('desc');

  // Results State
  const [metadataResults, setMetadataResults] = useState<FileItem[]>([]);
  const [totalMetadataCount, setTotalMetadataCount] = useState<number>(0);
  const [contentResults, setContentResults] = useState<ContentSearchResultItem[]>([]);
  const [totalContentMatches, setTotalContentMatches] = useState<number>(0);
  const [executionTimeMs, setExecutionTimeMs] = useState<number>(0);
  const [isSearching, setIsSearching] = useState<boolean>(false);
  const [isIndexing, setIsIndexing] = useState<boolean>(false);

  useEffect(() => {
    refreshContentStatus();
  }, []);

  const handleSearch = async () => {
    setIsSearching(true);
    const start = performance.now();

    try {
      if (searchMode === 'metadata') {
        const minBytes = minSizeKb ? Number(minSizeKb) * 1024 : undefined;
        const maxBytes = maxSizeKb ? Number(maxSizeKb) * 1024 : undefined;

        const res = await api.searchFiles({
          search: searchTerm.trim() || undefined,
          category: categoryFilter !== 'All' ? categoryFilter : undefined,
          extension: extensionFilter.trim() || undefined,
          minSize: minBytes,
          maxSize: maxBytes,
          sortBy,
          sortOrder,
          limit: 100,
        });

        setMetadataResults(res.files);
        setTotalMetadataCount(res.totalCount);
        setExecutionTimeMs(Math.round(performance.now() - start));
      } else {
        if (!searchTerm.trim()) {
          showToast('Please enter search keywords for full-text document search.');
          setIsSearching(false);
          return;
        }

        const res = await api.searchContent(searchTerm.trim(), 50);
        setContentResults(res.results);
        setTotalContentMatches(res.totalMatches);
        setExecutionTimeMs(res.executionTimeMs);
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Search error: ${msg}`);
    } finally {
      setIsSearching(false);
    }
  };

  const handleIndexClick = async () => {
    setIsIndexing(true);
    await indexDocumentContent(200);
    setIsIndexing(false);
  };

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
      {/* Top Header & Search Bar */}
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: '8px',
          backgroundColor: 'var(--bg-surface-0)',
          padding: '12px',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
                Search & Content Intelligence
              </h2>
              <Badge variant="accent">SQLite FTS5 + B-Tree Index</Badge>
            </div>
            <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
              Dual-mode search: fast indexed metadata queries or full-text document content search with highlighted snippets.
            </span>
          </div>

          {/* Mode Switcher Tabs */}
          <div
            style={{
              display: 'flex',
              backgroundColor: 'var(--bg-surface-1)',
              padding: '2px',
              borderRadius: 'var(--radius-sm)',
              border: '1px solid var(--border-subtle)',
            }}
          >
            <button
              type="button"
              onClick={() => setSearchMode('metadata')}
              style={{
                padding: '4px 12px',
                fontSize: 'var(--text-caption)',
                fontWeight: searchMode === 'metadata' ? 600 : 400,
                color: searchMode === 'metadata' ? 'var(--text-primary)' : 'var(--text-secondary)',
                backgroundColor: searchMode === 'metadata' ? 'var(--bg-surface-0)' : 'transparent',
                border: 'none',
                borderRadius: 'var(--radius-sm)',
                cursor: 'pointer',
              }}
            >
              Metadata Search
            </button>
            <button
              type="button"
              onClick={() => setSearchMode('content')}
              style={{
                padding: '4px 12px',
                fontSize: 'var(--text-caption)',
                fontWeight: searchMode === 'content' ? 600 : 400,
                color: searchMode === 'content' ? 'var(--text-primary)' : 'var(--text-secondary)',
                backgroundColor: searchMode === 'content' ? 'var(--bg-surface-0)' : 'transparent',
                border: 'none',
                borderRadius: 'var(--radius-sm)',
                cursor: 'pointer',
              }}
            >
              Document Content (FTS5)
            </button>
          </div>
        </div>

        {/* Input Bar */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <div style={{ position: 'relative', flex: 1 }}>
            <Search
              size={14}
              style={{ position: 'absolute', left: '10px', top: '9px', color: 'var(--text-muted)' }}
            />
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              placeholder={
                searchMode === 'metadata'
                  ? 'Search by file name or wildcard (e.g. report*, *.log)...'
                  : 'Search document text (Plain text, Markdown, Code, PDF)...'
              }
              style={{
                width: '100%',
                padding: '6px 10px 6px 32px',
                fontSize: 'var(--text-caption)',
                backgroundColor: 'var(--bg-surface-1)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
              }}
            />
          </div>

          <Button
            size="sm"
            variant="primary"
            disabled={isSearching}
            icon={<Search size={12} />}
            onClick={handleSearch}
          >
            {isSearching ? 'Searching...' : 'Search'}
          </Button>
        </div>

        {/* Filters Bar for Metadata Mode */}
        {searchMode === 'metadata' && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flexWrap: 'wrap', paddingTop: '4px' }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: 'var(--text-micro)' }}>
              <span style={{ color: 'var(--text-secondary)' }}>Category:</span>
              <select
                value={categoryFilter}
                onChange={(e) => setCategoryFilter(e.target.value)}
                style={{
                  padding: '2px 6px',
                  fontSize: 'var(--text-micro)',
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                }}
              >
                <option value="All">All Categories</option>
                <option value="Code">Code</option>
                <option value="Documents">Documents</option>
                <option value="Media">Media</option>
                <option value="Archives">Archives</option>
                <option value="Executables">Executables</option>
                <option value="Caches">Caches</option>
                <option value="Other">Other</option>
              </select>
            </label>

            <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: 'var(--text-micro)' }}>
              <span style={{ color: 'var(--text-secondary)' }}>Extension:</span>
              <input
                type="text"
                value={extensionFilter}
                onChange={(e) => setExtensionFilter(e.target.value)}
                placeholder=".pdf, .rs, .json"
                style={{
                  padding: '2px 6px',
                  width: '90px',
                  fontSize: 'var(--text-micro)',
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                }}
              />
            </label>

            <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: 'var(--text-micro)' }}>
              <span style={{ color: 'var(--text-secondary)' }}>Min KB:</span>
              <input
                type="number"
                value={minSizeKb}
                onChange={(e) => setMinSizeKb(e.target.value)}
                placeholder="0"
                style={{
                  padding: '2px 6px',
                  width: '70px',
                  fontSize: 'var(--text-micro)',
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                }}
              />
            </label>

            <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: 'var(--text-micro)' }}>
              <span style={{ color: 'var(--text-secondary)' }}>Max KB:</span>
              <input
                type="number"
                value={maxSizeKb}
                onChange={(e) => setMaxSizeKb(e.target.value)}
                placeholder="Any"
                style={{
                  padding: '2px 6px',
                  width: '70px',
                  fontSize: 'var(--text-micro)',
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                }}
              />
            </label>

            <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: 'var(--text-micro)' }}>
              <span style={{ color: 'var(--text-secondary)' }}>Sort:</span>
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as any)}
                style={{
                  padding: '2px 6px',
                  fontSize: 'var(--text-micro)',
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                }}
              >
                <option value="size">File Size</option>
                <option value="name">File Name</option>
                <option value="mtime">Modified Date</option>
              </select>
              <select
                value={sortOrder}
                onChange={(e) => setSortOrder(e.target.value as any)}
                style={{
                  padding: '2px 6px',
                  fontSize: 'var(--text-micro)',
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                }}
              >
                <option value="desc">Desc</option>
                <option value="asc">Asc</option>
              </select>
            </label>
          </div>
        )}

        {/* Content Mode Helper & Index Controls */}
        {searchMode === 'content' && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              flexWrap: 'wrap',
              gap: '8px',
              paddingTop: '6px',
              borderTop: '1px solid var(--border-subtle)',
              fontSize: 'var(--text-caption)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-secondary)' }}>
              <ShieldCheck size={14} style={{ color: 'var(--accent-base)' }} />
              <span>
                FTS5 Status: <strong>{contentStatus?.totalIndexedDocuments ?? 0}</strong> documents indexed (
                {(Number(contentStatus?.totalExtractedCharacters ?? 0) / 1000).toFixed(0)}k characters).
                Secrets/keys (<code>.env</code>, <code>.pem</code>, <code>.key</code>) excluded.
              </span>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <Button
                size="sm"
                variant="secondary"
                disabled={isIndexing}
                icon={<RefreshCw size={11} className={isIndexing ? 'animate-spin' : ''} />}
                onClick={handleIndexClick}
              >
                {isIndexing ? 'Indexing...' : 'Index Documents'}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                icon={<Trash2 size={11} />}
                onClick={clearContentIndex}
              >
                Clear Content Index
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Results Summary Bar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: 'var(--text-micro)',
          color: 'var(--text-secondary)',
          padding: '0 4px',
        }}
      >
        <span>
          {searchMode === 'metadata'
            ? `Found ${totalMetadataCount} indexed files (showing top ${metadataResults.length})`
            : `Found ${totalContentMatches} document matching occurrences`}
        </span>
        <span className="tabular-nums">Query Latency: {executionTimeMs} ms</span>
      </div>

      {/* Results View */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
        }}
      >
        {searchMode === 'metadata' ? (
          metadataResults.length === 0 ? (
            <div style={{ padding: '32px', textAlign: 'center', color: 'var(--text-muted)' }}>
              <FileCheck2 size={32} style={{ margin: '0 auto 8px', opacity: 0.6 }} />
              <p>No files match the specified query filters. Try a broader search term.</p>
            </div>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 'var(--text-caption)' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-1)' }}>
                  <th style={{ textAlign: 'left', padding: '6px 10px', fontWeight: 600 }}>File Name</th>
                  <th style={{ textAlign: 'left', padding: '6px 10px', fontWeight: 600 }}>Location</th>
                  <th style={{ textAlign: 'left', padding: '6px 10px', fontWeight: 600 }}>Category</th>
                  <th style={{ textAlign: 'right', padding: '6px 10px', fontWeight: 600 }}>Size</th>
                  <th style={{ textAlign: 'center', padding: '6px 10px', fontWeight: 600 }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {metadataResults.map((file) => (
                  <tr
                    key={file.id}
                    onClick={() => {
                      setSelectedFile(file);
                      setIsInspectorOpen(true);
                    }}
                    style={{
                      borderBottom: '1px solid var(--border-subtle)',
                      cursor: 'pointer',
                    }}
                    className="hover:bg-surface-1"
                  >
                    <td style={{ padding: '6px 10px', fontWeight: 500, color: 'var(--text-primary)' }}>
                      {file.name}
                    </td>
                    <td
                      style={{
                        padding: '6px 10px',
                        color: 'var(--text-secondary)',
                        maxWidth: '340px',
                      }}
                      className="truncate font-mono"
                      title={file.path}
                    >
                      {file.path}
                    </td>
                    <td style={{ padding: '6px 10px' }}>
                      <Badge variant="neutral">{file.category}</Badge>
                    </td>
                    <td style={{ padding: '6px 10px', textAlign: 'right' }} className="tabular-nums font-mono">
                      {formatBytes(file.sizeBytes)}
                    </td>
                    <td style={{ padding: '6px 10px', textAlign: 'center' }}>
                      <div style={{ display: 'inline-flex', gap: '4px' }}>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            openFileFolder(file.path);
                          }}
                          style={{
                            padding: '2px 5px',
                            backgroundColor: 'transparent',
                            border: '1px solid var(--border-subtle)',
                            borderRadius: 'var(--radius-sm)',
                            cursor: 'pointer',
                          }}
                          title="Reveal in Explorer"
                        >
                          <FolderOpen size={11} />
                        </button>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            copyFilePath(file.path);
                          }}
                          style={{
                            padding: '2px 5px',
                            backgroundColor: 'transparent',
                            border: '1px solid var(--border-subtle)',
                            borderRadius: 'var(--radius-sm)',
                            cursor: 'pointer',
                          }}
                          title="Copy Full Path"
                        >
                          <Copy size={11} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        ) : contentResults.length === 0 ? (
          <div style={{ padding: '32px', textAlign: 'center', color: 'var(--text-muted)' }}>
            <FileText size={32} style={{ margin: '0 auto 8px', opacity: 0.6 }} />
            <p>
              {searchTerm
                ? 'No document contents matched the search query. Try clicking "Index Documents" if you recently scanned new files.'
                : 'Enter keywords above to query full-text content inside plain text, markdown, source code, and PDFs.'}
            </p>
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            {contentResults.map((item) => (
              <div
                key={item.fileId}
                style={{
                  padding: '10px 14px',
                  borderBottom: '1px solid var(--border-subtle)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '6px',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <FileText size={14} style={{ color: 'var(--accent-base)' }} />
                    <span style={{ fontWeight: 600, color: 'var(--text-primary)', fontSize: 'var(--text-caption)' }}>
                      {item.filename}
                    </span>
                    <Badge variant="neutral">.{item.extension}</Badge>
                    {item.isStale && (
                      <Badge variant="warning" title="File was modified on disk after being indexed">
                        <AlertTriangle size={10} style={{ marginRight: '3px' }} /> Modified since Indexing
                      </Badge>
                    )}
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                      {formatBytes(item.sizeBytes)}
                    </span>
                    <button
                      type="button"
                      onClick={() => openFileFolder(item.path)}
                      style={{
                        padding: '2px 6px',
                        backgroundColor: 'transparent',
                        border: '1px solid var(--border-subtle)',
                        borderRadius: 'var(--radius-sm)',
                        cursor: 'pointer',
                        fontSize: 'var(--text-micro)',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '4px',
                      }}
                      title="Reveal in Explorer"
                    >
                      <FolderOpen size={11} /> Reveal
                    </button>
                    <button
                      type="button"
                      onClick={() => copyFilePath(item.path)}
                      style={{
                        padding: '2px 6px',
                        backgroundColor: 'transparent',
                        border: '1px solid var(--border-subtle)',
                        borderRadius: 'var(--radius-sm)',
                        cursor: 'pointer',
                        fontSize: 'var(--text-micro)',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '4px',
                      }}
                      title="Copy Full Path"
                    >
                      <Copy size={11} /> Copy
                    </button>
                  </div>
                </div>

                <div
                  className="font-mono truncate"
                  style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}
                  title={item.path}
                >
                  {item.path}
                </div>

                {/* Highlighted Match Snippet */}
                <div
                  style={{
                    backgroundColor: 'var(--bg-surface-1)',
                    padding: '6px 10px',
                    borderRadius: 'var(--radius-sm)',
                    fontSize: 'var(--text-micro)',
                    lineHeight: 1.45,
                    color: 'var(--text-secondary)',
                    fontFamily: 'var(--font-mono)',
                  }}
                  dangerouslySetInnerHTML={{ __html: item.snippet }}
                />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
