import {
  FileItem,
  ScanRoot,
  ScanSummary,
  FolderRanking,
  CategoryBreakdown,
  ScanErrorItem,
  ScanSessionItem,
  DuplicateScanResult,
  ContentSearchResponse,
  ContentIndexStatus,
  SafetyEvaluationResponse,
  SafetyAuditRecord,
  ExecuteCleanupResponse,
  SafetyRule,
  CleanupPlanDto,
  DryRunResponse,
  ExecutePlanResponse,
  QuarantineRecordDto,
  RestoreResponse,
  PurgeResponse,
} from '../types';

const API_BASE = '/api';

let cachedSessionToken: string = '';
let sessionTokenPromise: Promise<string> | null = null;

export async function getSessionToken(): Promise<string> {
  if (cachedSessionToken) return cachedSessionToken;

  if (typeof document !== 'undefined') {
    const meta = document.querySelector('meta[name="filemind-session-token"]');
    const content = meta?.getAttribute('content');
    if (content) {
      cachedSessionToken = content;
      return cachedSessionToken;
    }
  }

  if (sessionTokenPromise) return sessionTokenPromise;

  sessionTokenPromise = (async () => {
    try {
      const res = await fetch(`${API_BASE}/auth/handshake`, { signal: AbortSignal.timeout(1500) });
      if (res.ok) {
        const data = await res.json();
        if (data.session_token) {
          cachedSessionToken = data.session_token;
          return cachedSessionToken;
        }
      }
    } catch {}
    return '';
  })();

  return sessionTokenPromise;
}

export async function authFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  const token = await getSessionToken();
  const headers = new Headers(init?.headers);
  if (token && !headers.has('X-FileMind-Session-Token')) {
    headers.set('X-FileMind-Session-Token', token);
  }
  return fetch(input, {
    ...init,
    headers,
  });
}


export interface FileQueryParams {
  sessionId?: string;
  prefix?: string;
  category?: string;
  extension?: string;
  search?: string;
  minSize?: number;
  maxSize?: number;
  sortBy?: 'size' | 'name' | 'mtime' | 'category';
  sortOrder?: 'asc' | 'desc';
  limit?: number;
  offset?: number;
}

export interface FileQueryResponse {
  totalCount: number;
  totalBytes: number;
  files: FileItem[];
}

export interface ScanProgressResponse {
  percent: number;
  currentPath: string;
  filesCount: number;
  bytesCount: number;
  directoriesCount: number;
  errorsCount: number;
  elapsedMs: number;
  isScanning: boolean;
  sessionId?: string;
}

export async function checkBackendHealth(): Promise<boolean> {
  try {
    const res = await authFetch(`${API_BASE}/health`, { signal: AbortSignal.timeout(1500) });
    return res.ok;
  } catch {
    return false;
  }
}

export async function fetchScanRoots(): Promise<ScanRoot[]> {
  const res = await authFetch(`${API_BASE}/roots`);
  if (!res.ok) throw new Error(`Failed to fetch roots: ${res.statusText}`);
  return res.json();
}

export async function startRealScan(path: string, excludedPatterns?: string[]): Promise<{ sessionId: string }> {
  const res = await authFetch(`${API_BASE}/scan/start`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, excludedPatterns }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to start scan');
  }
  return res.json();
}

export async function cancelRealScan(): Promise<{ status: string }> {
  const res = await authFetch(`${API_BASE}/scan/cancel`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  });
  if (!res.ok) throw new Error(`Failed to cancel scan: ${res.statusText}`);
  return res.json();
}

export async function fetchScanProgress(): Promise<ScanProgressResponse> {
  const res = await authFetch(`${API_BASE}/scan/progress`);
  if (!res.ok) throw new Error(`Failed to fetch progress: ${res.statusText}`);
  return res.json();
}

export async function fetchScanSummary(root?: string): Promise<ScanSummary> {
  const url = root ? `${API_BASE}/scan/summary?root=${encodeURIComponent(root)}` : `${API_BASE}/scan/summary`;
  const res = await authFetch(url);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch scan summary');
  }
  return res.json();
}

export async function fetchScanSessions(): Promise<ScanSessionItem[]> {
  const res = await authFetch(`${API_BASE}/scan/sessions`);
  if (!res.ok) throw new Error(`Failed to fetch sessions: ${res.statusText}`);
  return res.json();
}

export async function fetchIndexedFiles(params: FileQueryParams = {}): Promise<FileQueryResponse> {
  const qs = new URLSearchParams();
  if (params.sessionId) qs.set('sessionId', params.sessionId);
  if (params.prefix) qs.set('prefix', params.prefix);
  if (params.category && params.category !== 'All') qs.set('category', params.category);
  if (params.extension) qs.set('extension', params.extension);
  if (params.search) qs.set('search', params.search);
  if (params.minSize !== undefined) qs.set('minSize', params.minSize.toString());
  if (params.maxSize !== undefined) qs.set('maxSize', params.maxSize.toString());
  if (params.sortBy) qs.set('sortBy', params.sortBy);
  if (params.sortOrder) qs.set('sortOrder', params.sortOrder);
  if (params.limit !== undefined) qs.set('limit', params.limit.toString());
  if (params.offset !== undefined) qs.set('offset', params.offset.toString());

  const res = await authFetch(`${API_BASE}/files?${qs.toString()}`);
  if (!res.ok) throw new Error(`Failed to query files: ${res.statusText}`);
  return res.json();
}

export async function fetchFolderRankings(limit = 20, sessionId?: string): Promise<FolderRanking[]> {
  const qs = new URLSearchParams({ limit: limit.toString() });
  if (sessionId) qs.set('sessionId', sessionId);
  const res = await authFetch(`${API_BASE}/folders/ranking?${qs.toString()}`);
  if (!res.ok) throw new Error(`Failed to fetch folder rankings: ${res.statusText}`);
  return res.json();
}

export async function fetchCategoryBreakdown(sessionId?: string): Promise<CategoryBreakdown[]> {
  const qs = new URLSearchParams();
  if (sessionId) qs.set('sessionId', sessionId);
  const res = await authFetch(`${API_BASE}/categories?${qs.toString()}`);
  if (!res.ok) throw new Error(`Failed to fetch category breakdown: ${res.statusText}`);
  return res.json();
}

export async function fetchLargeFiles(limit = 100, sessionId?: string): Promise<FileItem[]> {
  const qs = new URLSearchParams({ limit: limit.toString() });
  if (sessionId) qs.set('sessionId', sessionId);
  const res = await authFetch(`${API_BASE}/large-files?${qs.toString()}`);
  if (!res.ok) throw new Error(`Failed to fetch large files: ${res.statusText}`);
  return res.json();
}

export async function fetchScanErrors(limit = 100, sessionId?: string): Promise<ScanErrorItem[]> {
  const qs = new URLSearchParams({ limit: limit.toString() });
  if (sessionId) qs.set('sessionId', sessionId);
  const res = await authFetch(`${API_BASE}/errors?${qs.toString()}`);
  if (!res.ok) throw new Error(`Failed to fetch errors: ${res.statusText}`);
  return res.json();
}

export async function openContainingFolder(path: string): Promise<void> {
  const res = await authFetch(`${API_BASE}/actions/open-folder`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to open containing folder');
  }
}

export async function reconcileScan(sessionId: string, rootPath: string) {
  const res = await authFetch(`${API_BASE}/reconcile`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sessionId, rootPath }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to reconcile scan');
  }
  return res.json();
}

// ---------------------------------------------------------------------------
// Phase 3: Duplicate Detection API
// ---------------------------------------------------------------------------

export async function fetchDuplicates(
  sessionId?: string,
  minSizeBytes = 1,
  verifyBytes = false,
  maxGroups = 200
): Promise<DuplicateScanResult> {
  const qs = new URLSearchParams({
    minSizeBytes: minSizeBytes.toString(),
    verifyBytes: verifyBytes ? 'true' : 'false',
    maxGroups: maxGroups.toString(),
  });
  if (sessionId) qs.set('sessionId', sessionId);

  const res = await authFetch(`${API_BASE}/duplicates?${qs.toString()}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch duplicates');
  }
  return res.json();
}

export async function verifyDuplicateGroup(
  groupId: string,
  filePaths: string[]
): Promise<{ groupId: string; confidence: string; verifiedByteForByte: boolean; message: string }> {
  const res = await authFetch(`${API_BASE}/duplicates/verify-group`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ groupId, filePaths }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to verify duplicate group');
  }
  return res.json();
}

// ---------------------------------------------------------------------------
// Phase 3: Search & Content Indexing API
// ---------------------------------------------------------------------------

export async function searchFiles(params: FileQueryParams): Promise<FileQueryResponse> {
  const qs = new URLSearchParams();
  if (params.sessionId) qs.set('sessionId', params.sessionId);
  if (params.search) qs.set('search', params.search);
  if (params.category) qs.set('category', params.category);
  if (params.extension) qs.set('extension', params.extension);
  if (params.prefix) qs.set('prefix', params.prefix);
  if (params.minSize !== undefined) qs.set('minSize', params.minSize.toString());
  if (params.maxSize !== undefined) qs.set('maxSize', params.maxSize.toString());
  if (params.sortBy) qs.set('sortBy', params.sortBy);
  if (params.sortOrder) qs.set('sortOrder', params.sortOrder);
  if (params.limit) qs.set('limit', params.limit.toString());
  if (params.offset) qs.set('offset', params.offset.toString());

  const res = await authFetch(`${API_BASE}/search?${qs.toString()}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Search query failed');
  }
  return res.json();
}

export async function searchContent(
  q: string,
  limit = 50,
  offset = 0
): Promise<ContentSearchResponse> {
  const qs = new URLSearchParams({
    q,
    limit: limit.toString(),
    offset: offset.toString(),
  });

  const res = await authFetch(`${API_BASE}/search/content?${qs.toString()}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Content search failed');
  }
  return res.json();
}

export async function indexDocuments(
  sessionId?: string,
  maxFiles = 200
): Promise<{ indexedDocuments: number; totalDocuments: number; skippedSensitive: number }> {
  const res = await authFetch(`${API_BASE}/content/index`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sessionId, maxFiles }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to index documents');
  }
  return res.json();
}

export async function fetchContentStatus(): Promise<ContentIndexStatus> {
  const res = await authFetch(`${API_BASE}/content/status`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch content status');
  }
  return res.json();
}

export async function clearContentIndex(): Promise<void> {
  const res = await authFetch(`${API_BASE}/content/clear`, {
    method: 'POST',
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to clear content index');
  }
}

// ---------------------------------------------------------------------------
// Phase 4: Smart Delete Guardian & Safety Intelligence
// ---------------------------------------------------------------------------

export async function evaluateSafety(
  paths: string[],
  operation: string = 'evaluate'
): Promise<SafetyEvaluationResponse> {
  const res = await authFetch(`${API_BASE}/safety/evaluate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ paths, operation }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to evaluate target safety');
  }
  return res.json();
}

export async function fetchSafetyAudit(limit: number = 50): Promise<SafetyAuditRecord[]> {
  const res = await authFetch(`${API_BASE}/safety/audit?limit=${limit}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch safety audit log');
  }
  return res.json();
}

export async function executeGuardedCleanup(
  paths: string[],
  operation: 'recycle' | 'permanent' = 'recycle'
): Promise<ExecuteCleanupResponse> {
  const res = await authFetch(`${API_BASE}/safety/execute`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ paths, operation }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Cleanup execution failed');
  }
  return res.json();
}

export async function fetchLiveSafetyRules(): Promise<SafetyRule[]> {
  const res = await authFetch(`${API_BASE}/safety/rules`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch safety rules');
  }
  return res.json();
}

// ---------------------------------------------------------------------------
// Phase 5: Intelligent Cleanup Planning & Quarantine API
// ---------------------------------------------------------------------------

export async function createCleanupPlan(params: {
  goal: string;
  targetBytes?: number;
  folderPath?: string;
  selectedPaths?: string[];
  sessionId?: string;
}): Promise<CleanupPlanDto> {
  const res = await authFetch(`${API_BASE}/cleanup/plan`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to generate cleanup plan');
  }
  return res.json();
}

export async function executeDryRun(planId: string): Promise<DryRunResponse> {
  const res = await authFetch(`${API_BASE}/cleanup/dry-run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ planId }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Dry run execution failed');
  }
  return res.json();
}

export async function executeCleanupPlan(
  planId: string,
  selectedItemIds?: string[]
): Promise<ExecutePlanResponse> {
  const res = await authFetch(`${API_BASE}/cleanup/execute`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ planId, selectedItemIds }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Plan execution failed');
  }
  return res.json();
}

export async function fetchQuarantineRecords(): Promise<QuarantineRecordDto[]> {
  const res = await authFetch(`${API_BASE}/quarantine/records`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch quarantine records');
  }
  return res.json();
}

export async function restoreQuarantineTargets(
  quarantineIds: string[],
  alternateDestination?: string
): Promise<RestoreResponse> {
  const res = await authFetch(`${API_BASE}/quarantine/restore`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ quarantineIds, alternateDestination }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to restore targets');
  }
  return res.json();
}

export async function purgeQuarantineTargets(quarantineIds: string[]): Promise<PurgeResponse> {
  const res = await authFetch(`${API_BASE}/quarantine/purge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ quarantineIds }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to purge quarantine records');
  }
  return res.json();
}

// ---------------------------------------------------------------------------
// Phase 6: JARVIS Copilot & Local AI API
// ---------------------------------------------------------------------------

export async function queryCopilot(params: {
  query: string;
  history?: any[];
  allowAi?: boolean;
  ollamaEndpoint?: string;
  ollamaModel?: string;
}): Promise<any> {
  const res = await authFetch(`${API_BASE}/copilot/query`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Copilot query failed');
  }
  return res.json();
}

export async function fetchOllamaStatus(endpoint?: string): Promise<any> {
  const qs = endpoint ? `?endpoint=${encodeURIComponent(endpoint)}` : '';
  const res = await authFetch(`${API_BASE}/copilot/models${qs}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch Ollama status');
  }
  return res.json();
}

export async function executeCopilotTool(tool: string, args: any = {}): Promise<any> {
  const res = await authFetch(`${API_BASE}/copilot/tool`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tool, arguments: args }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Tool execution failed');
  }
  return res.json();
}

// ---------------------------------------------------------------------------
// Phase 7: Local Automation Engine & Developer Workspace API
// ---------------------------------------------------------------------------

export async function fetchAutomationRules(): Promise<any[]> {
  const res = await authFetch(`${API_BASE}/automation/rules`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch automation rules');
  }
  return res.json();
}

export async function createAutomationRule(ruleData: any): Promise<any> {
  const res = await authFetch(`${API_BASE}/automation/rules`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ruleData),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to create automation rule');
  }
  return res.json();
}

export async function toggleAutomationRule(ruleId: string, enabled: boolean): Promise<any> {
  const res = await authFetch(`${API_BASE}/automation/rules/${encodeURIComponent(ruleId)}/toggle`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ enabled }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to toggle rule');
  }
  return res.json();
}

export async function runAutomationRule(ruleId: string, dryRunOverride?: boolean): Promise<any> {
  const res = await authFetch(`${API_BASE}/automation/rules/${encodeURIComponent(ruleId)}/run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ dryRunOverride }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to run automation rule');
  }
  return res.json();
}

export async function deleteAutomationRule(ruleId: string): Promise<any> {
  const res = await authFetch(`${API_BASE}/automation/rules/${encodeURIComponent(ruleId)}`, {
    method: 'DELETE',
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to delete automation rule');
  }
  return res.json();
}

export async function fetchAutomationHistory(ruleId?: string, limit = 50): Promise<any[]> {
  const qs = new URLSearchParams();
  if (ruleId) qs.set('ruleId', ruleId);
  qs.set('limit', limit.toString());

  const res = await authFetch(`${API_BASE}/automation/history?${qs.toString()}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch automation history');
  }
  return res.json();
}

export async function fetchSchedulerStatus(): Promise<any> {
  const res = await authFetch(`${API_BASE}/automation/scheduler/status`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to fetch scheduler status');
  }
  return res.json();
}

export async function fetchDeveloperProjects(path?: string): Promise<any[]> {
  const qs = path ? `?path=${encodeURIComponent(path)}` : '';
  const res = await authFetch(`${API_BASE}/developer/projects${qs}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to scan developer projects');
  }
  return res.json();
}

export async function cleanDeveloperProjects(projectIds: string[], dryRun = true): Promise<any> {
  const res = await authFetch(`${API_BASE}/developer/clean`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ projectIds, dryRun }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Failed to stage developer cleanup');
  }
  return res.json();
}





