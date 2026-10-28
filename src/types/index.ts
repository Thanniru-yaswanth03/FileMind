// FileMind Domain Type Definitions

export type NavScreen =
  | 'overview'
  | 'storage-explorer'
  | 'large-files'
  | 'duplicates'
  | 'cleanup-planner'
  | 'safety-center'
  | 'developer-cleanup'
  | 'automations'
  | 'jarvis'
  | 'activity-recovery'
  | 'settings';

export type ThemeMode = 'light' | 'dark' | 'system';

export type RiskTier = 'safe' | 'caution' | 'high' | 'danger';

export interface ScanRoot {
  id: string;
  driveLetter: string;
  label: string;
  filesystem: 'NTFS' | 'ReFS' | 'exFAT';
  clusterSizeBytes: number;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
  isSystemDrive: boolean;
  lastScannedAt: string;
  scanFreshness: 'fresh' | 'stale' | 'unscanned';
}

export type FileCategory =
  | 'Code'
  | 'Media'
  | 'Archives'
  | 'Documents'
  | 'Executables'
  | 'Caches'
  | 'Other';

export interface FileItem {
  id: string;
  name: string;
  path: string;
  parentPath: string;
  extension: string;
  sizeBytes: number;
  physicalSizeBytes: number;
  category: FileCategory;
  mtime: string;
  ctime: string;
  atime: string;
  attributes: {
    isHidden: boolean;
    isSystem: boolean;
    isReadOnly: boolean;
    isReparsePoint: boolean;
  };
  ntfsFileId?: string;
  blake3Hash?: string;
  riskTier: RiskTier;
  safetyRationale: string;
  isLocked?: boolean;
  lockingProcess?: string;
  isDirectory?: boolean;
  childCount?: number;
}

export interface DuplicateGroup {
  groupId: string;
  blake3Hash: string;
  fileSizeBytes: number;
  wastedBytes: number;
  items: FileItem[];
  selectedCanonicalId: string;
}

export interface CleanupPlanItem {
  id: string;
  file: FileItem;
  riskTier: RiskTier;
  safetyRationale: string;
  potentialImpact: string;
  isSelected: boolean;
  isExcluded: boolean;
}

export interface CleanupPlan {
  planId: string;
  createdAt: string;
  status: 'draft' | 'approved' | 'in-progress' | 'completed' | 'cancelled';
  items: CleanupPlanItem[];
  totalItems: number;
  totalReclaimableBytes: number;
  riskCounts: Record<RiskTier, number>;
}

export interface SafetyRule {
  ruleId: string;
  name: string;
  type: 'PATH_GLOB' | 'REGEX' | 'PROCESS_LOCK' | 'MANIFEST_CHECK';
  pattern: string;
  riskTier: RiskTier;
  description: string;
  isBuiltin: boolean;
  isEnabled: boolean;
}

export interface DeveloperArtifact {
  name: string;
  relativePath: string;
  sizeBytes: number;
  isDisposable: boolean;
  category: string;
}

export interface DeveloperProject {
  id: string;
  name: string;
  path: string;
  ecosystem: 'Node.js' | 'Rust' | 'Python' | 'Go' | 'Java/Gradle' | 'Docker' | 'C/C++';
  artifacts: DeveloperArtifact[];
  totalArtifactSizeBytes: number;
  gitStatus: 'clean' | 'dirty' | 'untracked' | 'no-git';
  lastActiveDaysAgo: number;
}

export interface AutomationRule {
  id: string;
  name: string;
  trigger: 'schedule' | 'disk-pressure' | 'folder-watch';
  scheduleExpression: string;
  criteriaDescription: string;
  targetPath: string;
  targetTier: RiskTier;
  autoQuarantine: boolean;
  lastExecutedAt?: string;
  nextScheduledAt?: string;
  isEnabled: boolean;
}

export interface ActivityLog {
  id: string;
  timestamp: string;
  type: 'scan' | 'plan-generated' | 'quarantine' | 'restore' | 'purge' | 'automation';
  severity: 'info' | 'warn' | 'error' | 'security';
  title: string;
  description: string;
  affectedBytes?: number;
  itemCount?: number;
  manifestId?: string;
  canUndo?: boolean;
}

export interface QuarantineRecord {
  id: string;
  manifestId: string;
  originalPath: string;
  quarantineDate: string;
  sizeBytes: number;
  blake3Hash: string;
  status: 'quarantined' | 'restored' | 'purged';
  restorable: boolean;
}

export interface CopilotMessage {
  id: string;
  sender: 'user' | 'assistant';
  timestamp: string;
  content: string;
  suggestedAction?: {
    type: 'navigate' | 'create-plan' | 'inspect';
    label: string;
    screen?: NavScreen;
    targetPath?: string;
  };
}

export interface NotificationItem {
  id: string;
  timestamp: string;
  title: string;
  message: string;
  type: 'info' | 'warning' | 'success' | 'danger';
  read: boolean;
  actionScreen?: NavScreen;
}

export interface AppSettings {
  theme: ThemeMode;
  defaultScanRoot: string;
  lowPriorityIo: boolean;
  quarantineRetentionDays: number;
  ollamaEndpoint: string;
  ollamaModel: string;
  enableCopilot: boolean;
  excludedPaths: string[];
  autoCheckUpdates: boolean;
}

