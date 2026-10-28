import React, { createContext, useContext, useState, useEffect } from 'react';
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
  scanProgress: { percent: number; currentPath: string; filesCount: number; bytesCount: number };
  startScan: (rootId?: string) => void;
  cancelScan: () => void;
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
  restoreQuarantineRecord: (id: string) => void;
  purgeQuarantineRecord: (id: string) => void;
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
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [activeScreen, setActiveScreen] = useState<NavScreen>('overview');
  const [theme, setThemeState] = useState<ThemeMode>('light');
  const [scanRoots] = useState<ScanRoot[]>(mockScanRoots);
  const [selectedRootId, setSelectedRootId] = useState<string>('root-c');
  const [files] = useState<FileItem[]>(mockFiles);
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(mockFiles[0]);
  const [isInspectorOpen, setIsInspectorOpen] = useState<boolean>(true);
  const [inspectorWidth, setInspectorWidth] = useState<number>(330);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState<boolean>(false);
  const [isScanning, setIsScanning] = useState<boolean>(false);
  const [scanProgress, setScanProgress] = useState({
    percent: 0,
    currentPath: 'C:\\Users\\Developer\\AppData',
    filesCount: 0,
    bytesCount: 0,
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

  // Apply theme to document data-theme attribute
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

  const currentRoot = scanRoots.find((r) => r.id === selectedRootId) || scanRoots[0];

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => {
      setToastMessage((prev) => (prev === msg ? null : prev));
    }, 2800);
  };

  const toggleInspector = () => setIsInspectorOpen((prev) => !prev);
  const toggleSidebar = () => setIsSidebarCollapsed((prev) => !prev);

  // Simulated scan progress runner
  const startScan = (rootId?: string) => {
    const target = rootId ? scanRoots.find((r) => r.id === rootId)?.driveLetter || 'C:' : currentRoot.driveLetter;
    setIsScanning(true);
    setScanProgress({ percent: 0, currentPath: `${target}\\Windows\\System32`, filesCount: 0, bytesCount: 0 });
    showToast(`Started background scan on ${target}...`);

    let progress = 0;
    const interval = setInterval(() => {
      progress += 12;
      if (progress >= 100) {
        clearInterval(interval);
        setIsScanning(false);
        setScanProgress({ percent: 100, currentPath: 'Complete', filesCount: 421890, bytesCount: 1024 * 1024 * 1024 * 418.5 });
        showToast(`Scan complete for ${target}: 421,890 files verified.`);
      } else {
        setScanProgress({
          percent: progress,
          currentPath: `${target}\\Users\\Developer\\AppData\\Local\\File_${progress * 14}`,
          filesCount: Math.round(progress * 4200),
          bytesCount: Math.round(progress * 4.2 * 1024 * 1024 * 1024),
        });
      }
    }, 250);
  };

  const cancelScan = () => {
    setIsScanning(false);
    showToast('Scan cancelled by user.');
  };

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
      const selected = updated.filter((i) => i.isSelected && !i.isExcluded);
      return {
        ...prev,
        items: updated,
        totalItems: selected.length,
        totalReclaimableBytes: selected.reduce((acc, curr) => acc + curr.file.sizeBytes, 0),
      };
    });
    showToast('Item excluded from current cleanup plan.');
  };

  const stagePlanToQuarantine = () => {
    const selected = cleanupPlan.items.filter((i) => i.isSelected && !i.isExcluded);
    if (selected.length === 0) {
      showToast('No items selected for quarantine.');
      return;
    }

    // Add activity log
    const newLog: ActivityLog = {
      id: `act-${Date.now()}`,
      timestamp: new Date().toLocaleTimeString(),
      type: 'quarantine',
      severity: 'info',
      title: `Staged ${selected.length} items to .filemind_quarantine`,
      description: `Staged ${selected.map((s) => s.file.name).join(', ')}`,
      affectedBytes: cleanupPlan.totalReclaimableBytes,
      itemCount: selected.length,
      manifestId: `man-${Date.now()}`,
      canUndo: true,
    };

    setActivityLogs((prev) => [newLog, ...prev]);
    showToast(`Successfully staged ${selected.length} items into Quarantine Vault.`);
  };

  const setCanonicalDuplicate = (groupId: string, fileId: string) => {
    setDuplicateGroups((prev) =>
      prev.map((g) => (g.groupId === groupId ? { ...g, selectedCanonicalId: fileId } : g))
    );
    showToast('Designated canonical copy to keep.');
  };

  const toggleSafetyRule = (ruleId: string) => {
    setSafetyRules((prev) =>
      prev.map((r) => (r.ruleId === ruleId ? { ...r, isEnabled: !r.isEnabled } : r))
    );
    showToast('Updated safety rule status.');
  };

  const cleanProjectArtifacts = (projectId: string) => {
    const project = developerProjects.find((p) => p.id === projectId);
    if (!project) return;
    setDeveloperProjects((prev) =>
      prev.map((p) =>
        p.id === projectId
          ? {
              ...p,
              artifacts: p.artifacts.map((a) => ({ ...a, isDisposable: false })),
              totalArtifactSizeBytes: 0,
            }
          : p
      )
    );
    showToast(`Added build artifacts from ${project.name} to Cleanup Plan.`);
    setActiveScreen('cleanup-planner');
  };

  const toggleAutomation = (ruleId: string) => {
    setAutomations((prev) =>
      prev.map((a) => (a.id === ruleId ? { ...a, isEnabled: !a.isEnabled } : a))
    );
    showToast('Automation schedule toggled.');
  };

  const restoreQuarantineRecord = (id: string) => {
    setQuarantineRecords((prev) =>
      prev.map((q) => (q.id === id ? { ...q, status: 'restored' } : q))
    );
    showToast('Restoration completed with verified BLAKE3 hash parity.');
  };

  const purgeQuarantineRecord = (id: string) => {
    setQuarantineRecords((prev) =>
      prev.map((q) => (q.id === id ? { ...q, status: 'purged', restorable: false } : q))
    );
    showToast('Record permanently purged from quarantine.');
  };

  const sendCopilotMessage = (content: string) => {
    const userMsg: CopilotMessage = {
      id: `cop-${Date.now()}`,
      sender: 'user',
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      content,
    };

    // Advisory assistant response
    const assistantMsg: CopilotMessage = {
      id: `cop-${Date.now() + 1}`,
      sender: 'assistant',
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      content: `I received your inquiry regarding "${content}".\n\nAdvisory analysis: Reviewing verified metadata index for root ${currentRoot.driveLetter}. There are 3 recommended cleanup candidates categorized as Safe (Tier 1). No direct deletions can be made without your explicit plan confirmation.`,
      suggestedAction: {
        type: 'navigate',
        label: 'Review Cleanup Plan',
        screen: 'cleanup-planner',
      },
    };

    setCopilotMessages((prev) => [...prev, userMsg, assistantMsg]);
  };

  const markAllNotificationsRead = () => {
    setNotifications((prev) => prev.map((n) => ({ ...n, read: true })));
    showToast('All notifications marked as read.');
  };

  const openContextMenu = (x: number, y: number, file: FileItem) => {
    setContextMenu({ x, y, file });
  };

  const closeContextMenu = () => setContextMenu(null);

  const updateSettings = (newSettings: Partial<AppSettings>) => {
    setSettings((prev) => ({ ...prev, ...newSettings }));
    showToast('Preferences updated.');
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
