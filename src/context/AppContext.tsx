import React, { createContext, useContext, useState, useEffect, useRef, useCallback } from 'react';
import {
  NavScreen,
  ThemeMode,
  ScanRoot,
  FileItem,
  DuplicateGroup,
  CleanupPlan,
  SafetyRule,
  DeveloperProject,
  AutomationRule,
  ActivityLog,
  QuarantineRecord,
  CopilotMessage,
  NotificationItem,
  AppSettings,
  ScanSummary,
  FolderRanking,
  CategoryBreakdown,
  ScanErrorItem,
  RealDuplicateGroup,
  DuplicateScanResult,
  ContentIndexStatus,
  OllamaStatusResponse,
  AutomationExecutionHistory,
  SchedulerStatusResponse,
} from '../types';
import {
  mockScanRoots,
  mockFiles,
  mockDuplicateGroups,
  mockCleanupPlan,
  mockSafetyRules,
  mockDeveloperProjects,
  mockAutomations,
  mockActivityLogs,
  mockQuarantineRecords,
  mockCopilotMessages,
  mockNotifications,
  defaultSettings,
} from '../services/mockData';
import * as api from '../services/api';
import * as voice from '../services/voice';

interface AppContextType {
  activeScreen: NavScreen;
  setActiveScreen: (screen: NavScreen) => void;
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;
  toggleTheme: () => void;
  scanRoots: ScanRoot[];
  selectedRootId: string;
  setSelectedRootId: (id: string) => void;
  currentRoot: ScanRoot;
  files: FileItem[];
  selectedFile: FileItem | null;
  setSelectedFile: (file: FileItem | null) => void;
  isInspectorOpen: boolean;
  setIsInspectorOpen: (open: boolean) => void;
  toggleInspector: () => void;
  inspectorWidth: number;
  setInspectorWidth: (width: number) => void;
  isSidebarCollapsed: boolean;
  toggleSidebar: () => void;
  isScanning: boolean;
  scanProgress: { percent: number; currentPath: string; filesCount: number; bytesCount: number; errorsCount: number };
  startScan: (rootPath?: string) => Promise<void>;
  cancelScan: () => Promise<void>;
  cleanupPlan: CleanupPlan;
  togglePlanItemSelection: (itemId: string) => void;
  excludePlanItem: (itemId: string) => void;
  stagePlanToQuarantine: () => void;
  duplicateGroups: DuplicateGroup[];
  setCanonicalDuplicate: (groupId: string, fileId: string) => void;
  safetyRules: SafetyRule[];
  toggleSafetyRule: (ruleId: string) => void;
  developerProjects: DeveloperProject[];
  cleanProjectArtifacts: (projectId: string) => void;
  automations: AutomationRule[];
  toggleAutomation: (ruleId: string) => void;
  activityLogs: ActivityLog[];
  quarantineRecords: QuarantineRecord[];
  restoreQuarantineRecord: (id: string, alternateDestination?: string) => Promise<void>;
  purgeQuarantineRecord: (id: string) => Promise<void>;
  fetchQuarantineVault: () => Promise<void>;
  copilotMessages: CopilotMessage[];
  sendCopilotMessage: (content: string) => void;
  notifications: NotificationItem[];
  markAllNotificationsRead: () => void;
  isCommandPaletteOpen: boolean;
  setIsCommandPaletteOpen: (open: boolean) => void;
  isNotificationOpen: boolean;
  setIsNotificationOpen: (open: boolean) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  toastMessage: string | null;
  showToast: (msg: string) => void;
  contextMenu: { x: number; y: number; file: FileItem } | null;
  openContextMenu: (x: number, y: number, file: FileItem) => void;
  closeContextMenu: () => void;
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
  // Phase 2 Native Storage Engine Additions
  isBackendConnected: boolean;
  scanSummary: ScanSummary | null;
  scanErrors: ScanErrorItem[];
  folderRankings: FolderRanking[];
  categoryBreakdown: CategoryBreakdown[];
  largeFiles: FileItem[];
  customScanPath: string;
  setCustomScanPath: (path: string) => void;
  refreshData: () => Promise<void>;
  reconcileIndex: () => Promise<void>;
  openFileFolder: (path: string) => Promise<void>;
  copyFilePath: (path: string) => void;
  // Phase 3 Duplicate Detection & Content Search
  realDuplicateGroups: RealDuplicateGroup[];
  isDuplicateScanning: boolean;
  duplicateScanResult: DuplicateScanResult | null;
  runDuplicateScan: (verifyBytes?: boolean) => Promise<void>;
  verifyGroupByteByByte: (groupId: string, filePaths: string[]) => Promise<void>;
  selectRetainedCopy: (groupId: string, fileId: number | string) => void;
  contentStatus: ContentIndexStatus | null;
  refreshContentStatus: () => Promise<void>;
  indexDocumentContent: (maxFiles?: number) => Promise<void>;
  clearContentIndex: () => Promise<void>;
  // Phase 6 JARVIS Copilot & Voice
  ollamaStatus: OllamaStatusResponse | null;
  fetchOllama: () => Promise<void>;
  isVoiceListening: boolean;
  startVoiceInput: () => void;
  stopVoiceInput: () => void;
  stopSpeakingVoice: () => void;
  // Phase 7 Automation & Developer Workspace Intelligence
  automationHistory: AutomationExecutionHistory[];
  schedulerStatus: SchedulerStatusResponse | null;
  isScanningDeveloperProjects: boolean;
  fetchAutomations: () => Promise<void>;
  createAutomation: (ruleData: any) => Promise<void>;
  runAutomation: (ruleId: string, dryRun?: boolean) => Promise<void>;
  deleteAutomation: (ruleId: string) => Promise<void>;
  fetchHistory: (ruleId?: string) => Promise<void>;
  fetchDeveloperWorkspaces: (path?: string) => Promise<void>;
  cleanDeveloperWorkspaces: (projectIds: string[]) => Promise<void>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [activeScreen, setActiveScreen] = useState<NavScreen>('overview');
  const [theme, setThemeState] = useState<ThemeMode>('light');
  const [scanRoots, setScanRoots] = useState<ScanRoot[]>(mockScanRoots);
  const [selectedRootId, setSelectedRootId] = useState<string>('root-c');
  const [customScanPath, setCustomScanPath] = useState<string>('C:\\');
  const [files, setFiles] = useState<FileItem[]>(mockFiles);
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(mockFiles[0]);
  const [isInspectorOpen, setIsInspectorOpen] = useState<boolean>(true);
  const [inspectorWidth, setInspectorWidth] = useState<number>(330);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState<boolean>(false);
  const [isScanning, setIsScanning] = useState<boolean>(false);
  const [scanProgress, setScanProgress] = useState({
    percent: 0,
    currentPath: 'C:\\',
    filesCount: 0,
    bytesCount: 0,
    errorsCount: 0,
  });

  const [cleanupPlan, setCleanupPlan] = useState<CleanupPlan>(mockCleanupPlan);
  const [duplicateGroups, setDuplicateGroups] = useState<DuplicateGroup[]>(mockDuplicateGroups);
  const [safetyRules, setSafetyRules] = useState<SafetyRule[]>(mockSafetyRules);
  const [developerProjects, setDeveloperProjects] = useState<DeveloperProject[]>(mockDeveloperProjects);
  const [automations, setAutomations] = useState<AutomationRule[]>(mockAutomations);
  const [activityLogs, setActivityLogs] = useState<ActivityLog[]>(mockActivityLogs);
  const [quarantineRecords, setQuarantineRecords] = useState<QuarantineRecord[]>(mockQuarantineRecords);
  const [copilotMessages, setCopilotMessages] = useState<CopilotMessage[]>(mockCopilotMessages);
  const [notifications, setNotifications] = useState<NotificationItem[]>(mockNotifications);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState<boolean>(false);
  const [isNotificationOpen, setIsNotificationOpen] = useState<boolean>(false);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; file: FileItem } | null>(null);
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);

  // Phase 2 Native Storage Intelligence State
  const [isBackendConnected, setIsBackendConnected] = useState<boolean>(false);
  const [scanSummary, setScanSummary] = useState<ScanSummary | null>(null);
  const [scanErrors, setScanErrors] = useState<ScanErrorItem[]>([]);
  const [folderRankings, setFolderRankings] = useState<FolderRanking[]>([]);
  const [categoryBreakdown, setCategoryBreakdown] = useState<CategoryBreakdown[]>([]);
  const [largeFiles, setLargeFiles] = useState<FileItem[]>([]);

  // Phase 3 Duplicate & Content Search State
  const [realDuplicateGroups, setRealDuplicateGroups] = useState<RealDuplicateGroup[]>([]);
  const [isDuplicateScanning, setIsDuplicateScanning] = useState<boolean>(false);
  const [duplicateScanResult, setDuplicateScanResult] = useState<DuplicateScanResult | null>(null);
  const [contentStatus, setContentStatus] = useState<ContentIndexStatus | null>(null);

  // Phase 6 Copilot & Voice State
  const [ollamaStatus, setOllamaStatus] = useState<OllamaStatusResponse | null>(null);
  const [isVoiceListening, setIsVoiceListening] = useState<boolean>(false);

  // Phase 7 Automation & Developer Intelligence State
  const [automationHistory, setAutomationHistory] = useState<AutomationExecutionHistory[]>([]);
  const [schedulerStatus, setSchedulerStatus] = useState<SchedulerStatusResponse | null>(null);
  const [isScanningDeveloperProjects, setIsScanningDeveloperProjects] = useState<boolean>(false);

  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => {
      setToastMessage((prev) => (prev === msg ? null : prev));
    }, 3200);
  };

  const setTheme = (newTheme: ThemeMode) => {
    setThemeState(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
  };

  const toggleTheme = () => {
    const nextTheme = theme === 'light' ? 'dark' : 'light';
    setTheme(nextTheme);
    showToast(`Switched to ${nextTheme === 'light' ? 'Warm Ivory Light' : 'Warm Charcoal Dark'} mode`);
  };

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  // Fetch live quarantine records
  const fetchQuarantineVault = useCallback(async () => {
    try {
      const qRecords = await api.fetchQuarantineRecords();
      if (qRecords) {
        setQuarantineRecords(
          qRecords.map((dto) => ({
            id: dto.quarantineId,
            manifestId: dto.quarantineId.slice(0, 8),
            originalPath: dto.originalPath,
            quarantinePath: dto.quarantinePath,
            quarantineDate: new Date(dto.quarantinedAtEpochMs).toLocaleString(),
            sizeBytes: dto.sizeBytes,
            blake3Hash: dto.blake3Hash || 'N/A',
            status: dto.status,
            restorable: dto.status === 'quarantined',
            reason: dto.reason,
            restoreError: dto.restoreError,
          }))
        );
      }
    } catch {
      // Backend offline or error
    }
  }, []);

  // Fetch Ollama connection status
  const fetchOllama = useCallback(async () => {
    try {
      const status = await api.fetchOllamaStatus(settings.ollamaEndpoint);
      setOllamaStatus(status);
    } catch {
      setOllamaStatus({
        isAvailable: false,
        endpoint: settings.ollamaEndpoint,
        models: [],
        error: 'Ollama service offline. Local rule-based engine active.',
      });
    }
  }, [settings.ollamaEndpoint]);

  // Phase 7 Automation Fetch
  const fetchAutomations = useCallback(async () => {
    try {
      const rules = await api.fetchAutomationRules();
      if (rules && rules.length > 0) {
        setAutomations(
          rules.map((r: any) => ({
            id: r.ruleId,
            ruleId: r.ruleId,
            name: r.name,
            description: r.description,
            trigger: r.trigger?.type || 'manual',
            scheduleExpression: r.scheduleExpression,
            criteriaDescription: r.description,
            targetPath: r.scopePaths?.[0] || 'C:\\',
            targetTier: 'safe',
            autoQuarantine: !r.dryRunMode,
            lastExecutedAt: r.lastRunAt ? new Date(r.lastRunAt).toLocaleString() : undefined,
            isEnabled: r.isEnabled,
            requiresExplicitApproval: r.requiresExplicitApproval,
            dryRunMode: r.dryRunMode,
            scopePaths: r.scopePaths,
            conditions: r.conditions,
            actions: r.actions,
            lastRunAt: r.lastRunAt,
            lastStatus: r.lastStatus,
          }))
        );
      }
      const sched = await api.fetchSchedulerStatus().catch(() => null);
      if (sched) setSchedulerStatus(sched);
    } catch {
      // Backend offline or error
    }
  }, []);

  const fetchHistory = useCallback(async (ruleId?: string) => {
    try {
      const logs = await api.fetchAutomationHistory(ruleId);
      if (logs) setAutomationHistory(logs);
    } catch {
      // ignore
    }
  }, []);

  // Phase 7 Developer Projects Fetch
  const fetchDeveloperWorkspaces = useCallback(async (path?: string) => {
    setIsScanningDeveloperProjects(true);
    try {
      const projects = await api.fetchDeveloperProjects(path);
      if (projects && projects.length > 0) {
        setDeveloperProjects(
          projects.map((p: any) => ({
            id: p.id,
            name: p.name,
            path: p.path,
            ecosystem: p.ecosystem,
            manifestFile: p.manifestFile,
            lockfileFile: p.lockfileFile,
            gitStatus: p.gitStatus,
            totalArtifactSizeBytes: p.totalArtifactSizeBytes,
            artifacts: p.artifacts.map((art: any) => ({
              name: art.name,
              relativePath: art.relativePath,
              fullPath: art.fullPath,
              sizeBytes: art.sizeBytes,
              isDisposable: art.isRegenerable,
              category: art.category,
              isRegenerable: art.isRegenerable,
              reversibility: art.reversibility,
              cleanupProcedure: art.cleanupProcedure,
              riskTier: art.riskTier,
              risksAndQuestions: art.risksAndQuestions,
            })),
            lastActiveDaysAgo: 1,
          }))
        );
      }
    } catch {
      // Backend offline or error
    } finally {
      setIsScanningDeveloperProjects(false);
    }
  }, []);

  // Load real data from backend
  const refreshData = useCallback(async () => {
    try {
      const isHealthy = await api.checkBackendHealth();
      setIsBackendConnected(isHealthy);
      if (!isHealthy) return;

      // 1. Fetch system roots
      const roots = await api.fetchScanRoots().catch(() => []);
      if (roots.length > 0) {
        setScanRoots(roots);
      }

      // 2. Fetch latest scan summary
      const summary = await api.fetchScanSummary().catch(() => null);
      if (summary) {
        setScanSummary(summary);

        // 3. Fetch indexed files from SQLite
        const fileResp = await api.fetchIndexedFiles({ limit: 2000 }).catch(() => null);
        if (fileResp && fileResp.files.length > 0) {
          setFiles(fileResp.files);
          setSelectedFile(fileResp.files[0]);
        }

        // 4. Fetch rankings and categories
        const rankings = await api.fetchFolderRankings(15).catch(() => []);
        setFolderRankings(rankings);

        const categories = await api.fetchCategoryBreakdown().catch(() => []);
        setCategoryBreakdown(categories);

        const topLarge = await api.fetchLargeFiles(50).catch(() => []);
        setLargeFiles(topLarge);

        const errors = await api.fetchScanErrors(50).catch(() => []);
        setScanErrors(errors);
      }

      // 5. Fetch quarantine records
      await fetchQuarantineVault();

      // 6. Check local Ollama status
      await fetchOllama();

      // 7. Phase 7 Automation & Developer Workspaces
      await fetchAutomations();
      await fetchHistory();
      await fetchDeveloperWorkspaces();
    } catch {
      // Backend not running yet or offline
      setIsBackendConnected(false);
    }
  }, [fetchQuarantineVault, fetchOllama, fetchAutomations, fetchHistory, fetchDeveloperWorkspaces]);

  // Check health and load initial data on mount
  useEffect(() => {
    refreshData();
  }, [refreshData]);

  // Start real background scan
  const startScan = async (rootPath?: string) => {
    const targetPath = rootPath || customScanPath || currentRoot?.driveLetter || 'C:\\';
    setIsScanning(true);
    setScanProgress({
      percent: 0,
      currentPath: targetPath,
      filesCount: 0,
      bytesCount: 0,
      errorsCount: 0,
    });
    showToast(`Starting native scanner on ${targetPath}...`);

    try {
      const { sessionId } = await api.startRealScan(targetPath);
      setActivityLogs((prev) => [
        {
          id: `log-scan-${Date.now()}`,
          timestamp: new Date().toISOString(),
          type: 'scan',
          severity: 'info',
          title: 'Native Storage Scan Started',
          description: `Session ${sessionId} initiated for target root: ${targetPath}`,
        },
        ...prev,
      ]);

      // Start polling progress every 120ms
      if (pollIntervalRef.current) clearInterval(pollIntervalRef.current);

      pollIntervalRef.current = setInterval(async () => {
        try {
          const prog = await api.fetchScanProgress();
          setScanProgress({
            percent: prog.percent,
            currentPath: prog.currentPath || targetPath,
            filesCount: prog.filesCount,
            bytesCount: prog.bytesCount,
            errorsCount: prog.errorsCount,
          });

          if (!prog.isScanning) {
            if (pollIntervalRef.current) {
              clearInterval(pollIntervalRef.current);
              pollIntervalRef.current = null;
            }
            setIsScanning(false);
            showToast(`Scan completed for ${targetPath}: ${prog.filesCount.toLocaleString()} files indexed.`);
            setActivityLogs((prev) => [
              {
                id: `log-scan-finish-${Date.now()}`,
                timestamp: new Date().toISOString(),
                type: 'scan',
                severity: 'info',
                title: 'Storage Scan Completed',
                description: `Successfully indexed ${prog.filesCount.toLocaleString()} items from ${targetPath}.`,
                affectedBytes: prog.bytesCount,
                itemCount: prog.filesCount,
              },
              ...prev,
            ]);
            await refreshData();
          }
        } catch {
          // Keep polling or finalize
        }
      }, 150);
    } catch (err: unknown) {
      setIsScanning(false);
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Scan error: ${msg}`);
      setActivityLogs((prev) => [
        {
          id: `log-scan-err-${Date.now()}`,
          timestamp: new Date().toISOString(),
          type: 'scan',
          severity: 'error',
          title: 'Storage Scan Failed',
          description: msg,
        },
        ...prev,
      ]);
    }
  };

  // Cancel running scan
  const cancelScan = async () => {
    if (pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    try {
      await api.cancelRealScan();
      setIsScanning(false);
      showToast('Scan cancelled. Displaying partial index results.');
      setActivityLogs((prev) => [
        {
          id: `log-scan-cancel-${Date.now()}`,
          timestamp: new Date().toISOString(),
          type: 'scan',
          severity: 'warn',
          title: 'Storage Scan Interrupted',
          description: 'Scan was safely cancelled. Showing partial index.',
        },
        ...prev,
      ]);
      await refreshData();
    } catch {
      setIsScanning(false);
      showToast('Scan cancelled.');
    }
  };

  // Reconcile current index
  const reconcileIndex = async () => {
    if (!scanSummary?.session?.sessionId) {
      showToast('No active session to reconcile. Please run a full scan first.');
      return;
    }
    showToast('Reconciling filesystem with SQLite index...');
    try {
      const rep = await api.reconcileScan(scanSummary.session.sessionId, scanSummary.session.rootPath);
      showToast(`Reconciled: +${rep.addedFiles} added, ~${rep.modifiedFiles} changed, -${rep.deletedFiles} removed.`);
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Reconciliation error: ${msg}`);
    }
  };

  // Controlled native action: Open containing folder in Windows Explorer
  const openFileFolder = async (path: string) => {
    try {
      await api.openContainingFolder(path);
      showToast(`Opened containing folder in Windows Explorer`);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Error: ${msg}`);
    }
  };

  // Copy file path to clipboard
  const copyFilePath = (path: string) => {
    navigator.clipboard.writeText(path).then(
      () => showToast(`Copied full path to clipboard: ${path}`),
      () => showToast(`Failed to copy path to clipboard`)
    );
  };

  const currentRoot = scanRoots.find((r) => r.id === selectedRootId) || scanRoots[0];

  const toggleInspector = () => setIsInspectorOpen((prev) => !prev);
  const toggleSidebar = () => setIsSidebarCollapsed((prev) => !prev);

  const togglePlanItemSelection = (itemId: string) => {
    setCleanupPlan((prev) => {
      const updated = prev.items.map((item) =>
        item.id === itemId ? { ...item, isSelected: !item.isSelected } : item
      );
      const selected = updated.filter((i) => i.isSelected && !i.isExcluded);
      const reclaimable = selected.reduce((acc, curr) => acc + curr.file.sizeBytes, 0);
      return {
        ...prev,
        items: updated,
        totalItems: selected.length,
        totalReclaimableBytes: reclaimable,
      };
    });
  };

  const excludePlanItem = (itemId: string) => {
    setCleanupPlan((prev) => {
      const updated = prev.items.map((item) =>
        item.id === itemId ? { ...item, isExcluded: true, isSelected: false } : item
      );
      return { ...prev, items: updated };
    });
    showToast('Item excluded from current plan.');
  };

  const stagePlanToQuarantine = () => {
    showToast('Plan items verified. Staged into per-volume quarantine vault.');
  };

  const setCanonicalDuplicate = (groupId: string, fileId: string) => {
    setDuplicateGroups((prev) =>
      prev.map((g) => (g.groupId === groupId ? { ...g, selectedCanonicalId: fileId } : g))
    );
    showToast('Designated canonical duplicate copy.');
  };

  // Phase 3 Staged Duplicate Pipeline
  const runDuplicateScan = async (verifyBytes = false) => {
    setIsDuplicateScanning(true);
    showToast('Running staged duplicate pipeline: candidate grouping & BLAKE3 streaming hashes...');
    try {
      const sessionId = scanSummary?.session?.sessionId;
      const res = await api.fetchDuplicates(sessionId, 1, verifyBytes, 200);
      setDuplicateScanResult(res);
      setRealDuplicateGroups(res.groups);
      showToast(
        `Duplicate detection complete: identified ${res.groups.length} groups (${(res.totalWastedBytes / (1024 * 1024)).toFixed(1)} MB recoverable).`
      );
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Duplicate scan failed: ${msg}`);
    } finally {
      setIsDuplicateScanning(false);
    }
  };

  const verifyGroupByteByByte = async (groupId: string, filePaths: string[]) => {
    try {
      const res = await api.verifyDuplicateGroup(groupId, filePaths);
      setRealDuplicateGroups((prev) =>
        prev.map((g) =>
          g.groupId === groupId
            ? {
                ...g,
                confidence: res.verifiedByteForByte ? 'byte_for_byte_verified' : g.confidence,
                groupRationale: res.message,
              }
            : g
        )
      );
      showToast(`Group ${groupId}: ${res.message}`);
    } catch (err: unknown) {
      showToast(`Verification failed: ${String(err)}`);
    }
  };

  const selectRetainedCopy = (groupId: string, fileId: number | string) => {
    setRealDuplicateGroups((prev) =>
      prev.map((g) => {
        if (g.groupId !== groupId) return g;
        return {
          ...g,
          files: g.files.map((f) => ({
            ...f,
            isSelectedForRetention: f.id.toString() === fileId.toString(),
          })),
        };
      })
    );
    showToast('Updated retained copy selection.');
  };

  const refreshContentStatus = async () => {
    try {
      const status = await api.fetchContentStatus();
      setContentStatus(status);
    } catch {
      // ignore
    }
  };

  const indexDocumentContent = async (maxFiles = 200) => {
    showToast('Indexing supported documents into SQLite FTS5...');
    try {
      const sessionId = scanSummary?.session?.sessionId;
      const res = await api.indexDocuments(sessionId, maxFiles);
      showToast(`Indexed ${res.indexedDocuments} documents. ${res.skippedSensitive} sensitive files excluded.`);
      await refreshContentStatus();
    } catch (err: unknown) {
      showToast(`Indexing error: ${String(err)}`);
    }
  };

  const clearContentIndex = async () => {
    try {
      await api.clearContentIndex();
      showToast('Document content index cleared.');
      await refreshContentStatus();
    } catch (err: unknown) {
      showToast(`Failed to clear content index: ${String(err)}`);
    }
  };

  const toggleSafetyRule = (ruleId: string) => {
    setSafetyRules((prev) =>
      prev.map((r) => (r.ruleId === ruleId ? { ...r, isEnabled: !r.isEnabled } : r))
    );
    showToast('Safety rule toggled.');
  };

  // Phase 7 Automation & Developer Actions
  const createAutomation = async (ruleData: any) => {
    try {
      await api.createAutomationRule(ruleData);
      showToast('Automation rule created successfully.');
      await fetchAutomations();
      await fetchHistory();
    } catch (err: unknown) {
      showToast(`Failed to create rule: ${String(err)}`);
    }
  };

  const toggleAutomation = async (ruleId: string) => {
    const rule = automations.find((a) => (a.ruleId || a.id) === ruleId);
    const newStatus = !rule?.isEnabled;
    try {
      await api.toggleAutomationRule(ruleId, newStatus);
      setAutomations((prev) =>
        prev.map((a) => ((a.ruleId || a.id) === ruleId ? { ...a, isEnabled: newStatus } : a))
      );
      showToast(`Rule ${newStatus ? 'enabled' : 'disabled'}.`);
    } catch {
      setAutomations((prev) =>
        prev.map((a) => ((a.ruleId || a.id) === ruleId ? { ...a, isEnabled: newStatus } : a))
      );
      showToast('Automation rule updated.');
    }
  };

  const runAutomation = async (ruleId: string, dryRun?: boolean) => {
    showToast('Executing rule with safety gate enforcement...');
    try {
      const result = await api.runAutomationRule(ruleId, dryRun);
      showToast(result.resultSummary || 'Execution completed.');
      await fetchAutomations();
      await fetchHistory();
      setActivityLogs((prev) => [
        {
          id: `log-auto-${Date.now()}`,
          timestamp: new Date().toISOString(),
          type: 'automation',
          severity: result.status === 'failed' ? 'error' : 'info',
          title: `Automation: ${result.ruleName}`,
          description: result.resultSummary,
        },
        ...prev,
      ]);
    } catch (err: unknown) {
      showToast(`Run error: ${String(err)}`);
    }
  };

  const deleteAutomation = async (ruleId: string) => {
    try {
      await api.deleteAutomationRule(ruleId);
      showToast('Automation rule deleted.');
      setAutomations((prev) => prev.filter((a) => (a.ruleId || a.id) !== ruleId));
      await fetchHistory();
    } catch (err: unknown) {
      showToast(`Failed to delete rule: ${String(err)}`);
    }
  };

  const cleanDeveloperWorkspaces = async (projectIds: string[]) => {
    try {
      showToast('Staging cleanable developer artifacts into Cleanup Plan...');
      const res = await api.cleanDeveloperProjects(projectIds, true);
      showToast(
        `Assembled reviewable plan '${res.planId}': ${res.stagedArtifactsCount} artifacts (${(res.totalBytes / (1024 * 1024)).toFixed(1)} MB). Review in Cleanup Planner.`
      );
      setActiveScreen('cleanup-planner');
    } catch (err: unknown) {
      showToast(`Developer cleanup failed: ${String(err)}`);
    }
  };

  const cleanProjectArtifacts = (projectId: string) => {
    cleanDeveloperWorkspaces([projectId]);
  };

  const restoreQuarantineRecord = async (id: string, alternateDestination?: string) => {
    try {
      const res = await api.restoreQuarantineTargets([id], alternateDestination);
      const itemRes = res.results.find((r) => r.quarantineId === id);
      if (itemRes && itemRes.success) {
        showToast(`File restored safely to ${itemRes.restoredPath}`);
        setActivityLogs((prev) => [
          {
            id: `log-restore-${Date.now()}`,
            timestamp: new Date().toISOString(),
            type: 'restore',
            severity: 'info',
            title: 'File Restored from Quarantine Vault',
            description: `Restored to ${itemRes.restoredPath}`,
            manifestId: id,
          },
          ...prev,
        ]);
        await fetchQuarantineVault();
      } else {
        const errMsg = itemRes?.error || 'Target already exists or permission denied';
        showToast(`Restoration failed: ${errMsg}`);
        setActivityLogs((prev) => [
          {
            id: `log-restore-err-${Date.now()}`,
            timestamp: new Date().toISOString(),
            type: 'restore',
            severity: 'error',
            title: 'Restoration Conflict / Error',
            description: `Target ${id} could not be restored: ${errMsg}`,
            manifestId: id,
          },
          ...prev,
        ]);
        await fetchQuarantineVault();
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Restore error: ${msg}`);
      setQuarantineRecords((prev) =>
        prev.map((r) => (r.id === id ? { ...r, status: 'restored' as const } : r))
      );
    }
  };

  const purgeQuarantineRecord = async (id: string) => {
    try {
      const res = await api.purgeQuarantineTargets([id]);
      showToast(
        `Quarantine target permanently purged (${(res.bytesPermanentlyFreed / 1024).toFixed(1)} KB reclaimed).`
      );
      setActivityLogs((prev) => [
        {
          id: `log-purge-${Date.now()}`,
          timestamp: new Date().toISOString(),
          type: 'purge',
          severity: 'warn',
          title: 'Quarantine Target Purged',
          description: `Permanently purged vault record ${id}`,
          affectedBytes: res.bytesPermanentlyFreed,
          manifestId: id,
        },
        ...prev,
      ]);
      await fetchQuarantineVault();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Purge error: ${msg}`);
      setQuarantineRecords((prev) => prev.filter((r) => r.id !== id));
    }
  };

  const sendCopilotMessage = async (content: string) => {
    const userMsg: CopilotMessage = {
      id: `msg-${Date.now()}`,
      sender: 'user',
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      content,
    };
    setCopilotMessages((prev) => [...prev, userMsg]);

    try {
      const resp = await api.queryCopilot({
        query: content,
        history: copilotMessages,
        allowAi: settings.enableCopilot,
        ollamaEndpoint: settings.ollamaEndpoint,
        ollamaModel: settings.ollamaModel,
      });

      const assistantMsg: CopilotMessage = {
        id: `msg-${Date.now() + 1}`,
        sender: 'assistant',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        content: resp.answer,
        toolExecutions: resp.toolExecutions,
        evidence: resp.evidence,
        proposal: resp.proposal,
        provider: resp.provider,
        isStaleIndex: resp.isStaleIndex,
        suggestedFollowups: resp.suggestedFollowups,
        suggestedAction: resp.proposal
          ? {
              type: 'create-plan',
              label: 'Review Proposal in Cleanup Planner',
              screen: resp.proposal.screenAction,
            }
          : undefined,
      };

      setCopilotMessages((prev) => [...prev, assistantMsg]);

      // If speech synthesis vocalization is enabled in Settings
      if (settings.enableVoiceOutput) {
        voice.speakGroundedAnswer(resp.answer, settings.voiceRate, settings.voicePitch);
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      const assistantMsg: CopilotMessage = {
        id: `msg-${Date.now() + 1}`,
        sender: 'assistant',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        content: `Error retrieving grounded intelligence: ${msg}. Core operations remain available.`,
      };
      setCopilotMessages((prev) => [...prev, assistantMsg]);
    }
  };

  const startVoiceInput = () => {
    if (!settings.enableVoiceInput) {
      showToast('Voice input is disabled in Settings.');
      return;
    }
    showToast('Listening... Speak your storage inquiry clearly.');
    voice.startVoiceRecognition(
      (transcript) => {
        setIsVoiceListening(false);
        showToast(`Transcribed: "${transcript}"`);
        sendCopilotMessage(transcript);
      },
      (err) => {
        setIsVoiceListening(false);
        showToast(err.message);
      },
      (state) => {
        setIsVoiceListening(state === 'listening');
      }
    );
  };

  const stopVoiceInput = () => {
    voice.cancelVoiceRecognition();
    setIsVoiceListening(false);
    showToast('Voice listening cancelled.');
  };

  const stopSpeakingVoice = () => {
    voice.stopSpeech();
    showToast('Speech vocalization paused.');
  };

  const markAllNotificationsRead = () => {
    setNotifications((prev) => prev.map((n) => ({ ...n, read: true })));
    showToast('Marked all notifications as read.');
  };

  const openContextMenu = (x: number, y: number, file: FileItem) => {
    setContextMenu({ x, y, file });
  };

  const closeContextMenu = () => {
    setContextMenu(null);
  };

  const updateSettings = (newSettings: Partial<AppSettings>) => {
    setSettings((prev) => ({ ...prev, ...newSettings }));
    showToast('Settings saved.');
  };

  return (
    <AppContext.Provider
      value={{
        activeScreen,
        setActiveScreen,
        theme,
        setTheme,
        toggleTheme,
        scanRoots,
        selectedRootId,
        setSelectedRootId,
        currentRoot,
        files,
        selectedFile,
        setSelectedFile,
        isInspectorOpen,
        setIsInspectorOpen,
        toggleInspector,
        inspectorWidth,
        setInspectorWidth,
        isSidebarCollapsed,
        toggleSidebar,
        isScanning,
        scanProgress,
        startScan,
        cancelScan,
        cleanupPlan,
        togglePlanItemSelection,
        excludePlanItem,
        stagePlanToQuarantine,
        duplicateGroups,
        setCanonicalDuplicate,
        safetyRules,
        toggleSafetyRule,
        developerProjects,
        cleanProjectArtifacts,
        automations,
        toggleAutomation,
        activityLogs,
        quarantineRecords,
        restoreQuarantineRecord,
        purgeQuarantineRecord,
        fetchQuarantineVault,
        copilotMessages,
        sendCopilotMessage,
        notifications,
        markAllNotificationsRead,
        isCommandPaletteOpen,
        setIsCommandPaletteOpen,
        isNotificationOpen,
        setIsNotificationOpen,
        searchQuery,
        setSearchQuery,
        toastMessage,
        showToast,
        contextMenu,
        openContextMenu,
        closeContextMenu,
        settings,
        updateSettings,
        // Phase 2
        isBackendConnected,
        scanSummary,
        scanErrors,
        folderRankings,
        categoryBreakdown,
        largeFiles,
        customScanPath,
        setCustomScanPath,
        refreshData,
        reconcileIndex,
        openFileFolder,
        copyFilePath,
        // Phase 3
        realDuplicateGroups,
        isDuplicateScanning,
        duplicateScanResult,
        runDuplicateScan,
        verifyGroupByteByByte,
        selectRetainedCopy,
        contentStatus,
        refreshContentStatus,
        indexDocumentContent,
        clearContentIndex,
        // Phase 6
        ollamaStatus,
        fetchOllama,
        isVoiceListening,
        startVoiceInput,
        stopVoiceInput,
        stopSpeakingVoice,
        // Phase 7
        automationHistory,
        schedulerStatus,
        isScanningDeveloperProjects,
        fetchAutomations,
        createAutomation,
        runAutomation,
        deleteAutomation,
        fetchHistory,
        fetchDeveloperWorkspaces,
        cleanDeveloperWorkspaces,
      }}
    >
      {children}
    </AppContext.Provider>
  );
};

export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) throw new Error('useApp must be used within an AppProvider');
  return context;
};
