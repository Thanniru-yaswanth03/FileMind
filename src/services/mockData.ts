import {
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
  AppSettings,
  NotificationItem,
} from '../types';

export const IS_SAMPLE_DATA_SET = true;
export const DATA_DISCLAIMER =
  'Simulated Development Dataset — Demonstrating interaction patterns before Phase 4 native engine integration.';

export const mockScanRoots: ScanRoot[] = [
  {
    id: 'root-c',
    driveLetter: 'C:',
    label: 'Local Disk (OS NVMe)',
    filesystem: 'NTFS',
    clusterSizeBytes: 4096,
    totalBytes: 1024 * 1024 * 1024 * 512, // 512 GB
    usedBytes: 1024 * 1024 * 1024 * 418.5, // 418.5 GB (81.7%)
    freeBytes: 1024 * 1024 * 1024 * 93.5, // 93.5 GB
    isSystemDrive: true,
    lastScannedAt: '2026-10-28 14:15:22',
    scanFreshness: 'fresh',
  },
  {
    id: 'root-d',
    driveLetter: 'D:',
    label: 'Projects & Workspaces (SATA SSD)',
    filesystem: 'NTFS',
    clusterSizeBytes: 4096,
    totalBytes: 1024 * 1024 * 1024 * 1024, // 1 TB
    usedBytes: 1024 * 1024 * 1024 * 712.4, // 712.4 GB (69.5%)
    freeBytes: 1024 * 1024 * 1024 * 311.6, // 311.6 GB
    isSystemDrive: false,
    lastScannedAt: '2026-10-27 19:40:10',
    scanFreshness: 'fresh',
  },
  {
    id: 'root-e',
    driveLetter: 'E:',
    label: 'Media Scratch & Archives (HDD)',
    filesystem: 'exFAT',
    clusterSizeBytes: 32768,
    totalBytes: 1024 * 1024 * 1024 * 2048, // 2 TB
    usedBytes: 1024 * 1024 * 1024 * 1680.0, // 1.68 TB (82%)
    freeBytes: 1024 * 1024 * 1024 * 368.0,
    isSystemDrive: false,
    lastScannedAt: '2026-10-22 09:12:00',
    scanFreshness: 'stale',
  },
];

export const mockFiles: FileItem[] = [
  {
    id: 'f-1',
    name: 'target',
    path: 'D:\\Projects\\FileMind\\src-tauri\\target',
    parentPath: 'D:\\Projects\\FileMind\\src-tauri',
    extension: '',
    sizeBytes: 1024 * 1024 * 1024 * 14.8, // 14.8 GB
    physicalSizeBytes: 1024 * 1024 * 1024 * 15.1,
    category: 'Caches',
    mtime: '2026-10-28 11:20:14',
    ctime: '2026-09-12 10:04:00',
    atime: '2026-10-28 11:20:14',
    attributes: { isHidden: false, isSystem: false, isReadOnly: false, isReparsePoint: false },
    riskTier: 'safe',
    safetyRationale: 'Disposable Rust compilation artifact verified by clean parent Git repository.',
    isDirectory: true,
    childCount: 4281,
  },
  {
    id: 'f-2',
    name: 'node_modules',
    path: 'D:\\Projects\\WebPlatform\\frontend\\node_modules',
    parentPath: 'D:\\Projects\\WebPlatform\\frontend',
    extension: '',
    sizeBytes: 1024 * 1024 * 840, // 840 MB
    physicalSizeBytes: 1024 * 1024 * 892,
    category: 'Caches',
    mtime: '2026-10-15 08:33:10',
    ctime: '2026-07-01 14:12:00',
    atime: '2026-10-15 08:33:10',
    attributes: { isHidden: false, isSystem: false, isReadOnly: false, isReparsePoint: false },
    riskTier: 'safe',
    safetyRationale: 'Node dependency tree regenerable via package-lock.json.',
    isDirectory: true,
    childCount: 18450,
  },
  {
    id: 'f-3',
    name: 'ubuntu-24.04-desktop-amd64.iso',
    path: 'C:\\Users\\Developer\\Downloads\\ubuntu-24.04-desktop-amd64.iso',
    parentPath: 'C:\\Users\\Developer\\Downloads',
    extension: 'iso',
    sizeBytes: 1024 * 1024 * 1024 * 5.75, // 5.75 GB
    physicalSizeBytes: 1024 * 1024 * 1024 * 5.75,
    category: 'Archives',
    mtime: '2026-05-18 16:40:02',
    ctime: '2026-05-18 16:40:02',
    atime: '2026-06-01 10:00:00',
    attributes: { isHidden: false, isSystem: false, isReadOnly: false, isReparsePoint: false },
    blake3Hash: 'b3_a891f4c78104be9d837612f08a9c8e11a2d4f5e61234',
    riskTier: 'caution',
    safetyRationale: 'Download archive untouched for over 140 days; review before staging.',
    isDirectory: false,
  },
  {
    id: 'f-4',
    name: 'ubuntu-24.04-desktop-amd64 (1).iso',
    path: 'D:\\ISO_Images\\ubuntu-24.04-desktop-amd64 (1).iso',
    parentPath: 'D:\\ISO_Images',
    extension: 'iso',
    sizeBytes: 1024 * 1024 * 1024 * 5.75, // Exact same size
    physicalSizeBytes: 1024 * 1024 * 1024 * 5.75,
    category: 'Archives',
    mtime: '2026-05-20 12:15:30',
    ctime: '2026-05-20 12:15:30',
    atime: '2026-05-20 12:15:30',
    attributes: { isHidden: false, isSystem: false, isReadOnly: false, isReparsePoint: false },
    blake3Hash: 'b3_a891f4c78104be9d837612f08a9c8e11a2d4f5e61234',
    riskTier: 'caution',
    safetyRationale: 'Verified duplicate copy of download ISO; candidate for quarantine.',
    isDirectory: false,
  },
  {
    id: 'f-5',
    name: 'docker_desktop_data.vhdx',
    path: 'C:\\Users\\Developer\\AppData\\Local\\Docker\\wsl\\data\\ext4.vhdx',
    parentPath: 'C:\\Users\\Developer\\AppData\\Local\\Docker\\wsl\\data',
    extension: 'vhdx',
    sizeBytes: 1024 * 1024 * 1024 * 48.2, // 48.2 GB
    physicalSizeBytes: 1024 * 1024 * 1024 * 48.2,
    category: 'Caches',
    mtime: '2026-10-28 14:02:11',
    ctime: '2025-11-04 09:20:00',
    atime: '2026-10-28 14:02:11',
    attributes: { isHidden: false, isSystem: false, isReadOnly: false, isReparsePoint: false },
    riskTier: 'danger',
    safetyRationale: 'Active Docker WSL virtual disk; locked by Docker Desktop service.',
    isLocked: true,
    lockingProcess: 'Docker Desktop.exe (PID 8412)',
    isDirectory: false,
  },
  {
    id: 'f-6',
    name: 'ClientPresentation_Final_v3.mp4',
    path: 'E:\\VideoProjects\\2026\\ClientPresentation_Final_v3.mp4',
    parentPath: 'E:\\VideoProjects\\2026',
    extension: 'mp4',
    sizeBytes: 1024 * 1024 * 1024 * 8.4, // 8.4 GB
    physicalSizeBytes: 1024 * 1024 * 1024 * 8.4,
    category: 'Media',
    mtime: '2026-09-02 18:22:10',
    ctime: '2026-09-02 18:22:10',
    atime: '2026-09-05 11:00:00',
    attributes: { isHidden: false, isSystem: false, isReadOnly: false, isReparsePoint: false },
    blake3Hash: 'b3_4490fba8c91024de817726cb310a08e49221cd31459a',
    riskTier: 'high',
    safetyRationale: 'User-created video deliverable. Never safe for automated deletion.',
    isDirectory: false,
  },
  {
    id: 'f-7',
    name: 'ClientPresentation_Final_v3_copy.mp4',
    path: 'E:\\VideoProjects\\Backups\\ClientPresentation_Final_v3_copy.mp4',
    parentPath: 'E:\\VideoProjects\\Backups',
    extension: 'mp4',
    sizeBytes: 1024 * 1024 * 1024 * 8.4,
    physicalSizeBytes: 1024 * 1024 * 1024 * 8.4,
    category: 'Media',
    mtime: '2026-09-03 09:10:00',
    ctime: '2026-09-03 09:10:00',
    atime: '2026-09-03 09:10:00',
    attributes: { isHidden: false, isSystem: false, isReadOnly: false, isReparsePoint: false },
    blake3Hash: 'b3_4490fba8c91024de817726cb310a08e49221cd31459a',
    riskTier: 'high',
    safetyRationale: 'Exact duplicate video file in backup directory.',
    isDirectory: false,
  },
  {
    id: 'f-8',
    name: 'System32',
    path: 'C:\\Windows\\System32',
    parentPath: 'C:\\Windows',
    extension: '',
    sizeBytes: 1024 * 1024 * 1024 * 24.1,
    physicalSizeBytes: 1024 * 1024 * 1024 * 24.5,
    category: 'Executables',
    mtime: '2026-10-24 03:00:00',
    ctime: '2025-01-10 00:00:00',
    atime: '2026-10-28 14:15:22',
    attributes: { isHidden: false, isSystem: true, isReadOnly: true, isReparsePoint: false },
    riskTier: 'danger',
    safetyRationale: 'Core Windows operating system root directory. Denylisted by security policy.',
    isDirectory: true,
    childCount: 4890,
  },
];

export const mockDuplicateGroups: DuplicateGroup[] = [
  {
    groupId: 'dup-1',
    blake3Hash: 'b3_a891f4c78104be9d837612f08a9c8e11a2d4f5e61234',
    fileSizeBytes: 1024 * 1024 * 1024 * 5.75,
    wastedBytes: 1024 * 1024 * 1024 * 5.75,
    selectedCanonicalId: 'f-3',
    items: [mockFiles[2], mockFiles[3]],
  },
  {
    groupId: 'dup-2',
    blake3Hash: 'b3_4490fba8c91024de817726cb310a08e49221cd31459a',
    fileSizeBytes: 1024 * 1024 * 1024 * 8.4,
    wastedBytes: 1024 * 1024 * 1024 * 8.4,
    selectedCanonicalId: 'f-6',
    items: [mockFiles[5], mockFiles[6]],
  },
];

export const mockCleanupPlan: CleanupPlan = {
  planId: 'plan-2026-10-28-001',
  createdAt: '2026-10-28 14:30:00',
  status: 'draft',
  totalItems: 4,
  totalReclaimableBytes: 1024 * 1024 * 1024 * 21.39, // ~21.4 GB
  riskCounts: {
    safe: 2,
    caution: 2,
    high: 0,
    danger: 0,
  },
  items: [
    {
      id: 'pi-1',
      file: mockFiles[0], // Rust target/ (14.8 GB)
      riskTier: 'safe',
      safetyRationale: 'Verified Rust compilation target in inactive branch.',
      potentialImpact: 'Next cargo build will recompile dependencies.',
      isSelected: true,
      isExcluded: false,
    },
    {
      id: 'pi-2',
      file: mockFiles[1], // WebPlatform node_modules (840 MB)
      riskTier: 'safe',
      safetyRationale: 'Disposable dependency directory; clean Git tree.',
      potentialImpact: 'Next build requires pnpm install.',
      isSelected: true,
      isExcluded: false,
    },
    {
      id: 'pi-3',
      file: mockFiles[3], // Duplicate ISO (5.75 GB)
      riskTier: 'caution',
      safetyRationale: 'Redundant copy of Ubuntu ISO already present in Downloads.',
      potentialImpact: 'Only duplicate copy is staged; primary download preserved.',
      isSelected: true,
      isExcluded: false,
    },
  ],
};

export const mockSafetyRules: SafetyRule[] = [
  {
    ruleId: 'rule-win-sys',
    name: 'Windows Operating System Root',
    type: 'PATH_GLOB',
    pattern: 'C:\\Windows\\**',
    riskTier: 'danger',
    description: 'Prevents modification or deletion of Windows core libraries and drivers.',
    isBuiltin: true,
    isEnabled: true,
  },
  {
    ruleId: 'rule-prog-files',
    name: 'Installed Program Executables',
    type: 'PATH_GLOB',
    pattern: 'C:\\Program Files\\**',
    riskTier: 'danger',
    description: 'Protects installed desktop application binaries from accidental deletion.',
    isBuiltin: true,
    isEnabled: true,
  },
  {
    ruleId: 'rule-git-dirty',
    name: 'Uncommitted Git Repository Guard',
    type: 'MANIFEST_CHECK',
    pattern: '.git/index',
    riskTier: 'high',
    description: 'Demotes any project cache to High Risk if the parent Git repository has uncommitted changes.',
    isBuiltin: true,
    isEnabled: true,
  },
  {
    ruleId: 'rule-rust-target',
    name: 'Cargo Target Build Artifacts',
    type: 'MANIFEST_CHECK',
    pattern: '**/target/** (Cargo.toml)',
    riskTier: 'safe',
    description: 'Classifies target/ folders as safe to clean if accompanied by a valid Cargo manifest in a clean repository.',
    isBuiltin: true,
    isEnabled: true,
  },
  {
    ruleId: 'rule-proc-lock',
    name: 'Active Process Lock Check',
    type: 'PROCESS_LOCK',
    pattern: 'Win32 Restart Manager API',
    riskTier: 'danger',
    description: 'Blocks operations on files held open by active Windows background processes.',
    isBuiltin: true,
    isEnabled: true,
  },
];

export const mockDeveloperProjects: DeveloperProject[] = [
  {
    id: 'proj-1',
    name: 'FileMind',
    path: 'D:\\Projects\\FileMind',
    ecosystem: 'Rust',
    gitStatus: 'clean',
    lastActiveDaysAgo: 0,
    totalArtifactSizeBytes: 1024 * 1024 * 1024 * 14.8,
    artifacts: [
      { name: 'target', relativePath: 'src-tauri\\target', sizeBytes: 1024 * 1024 * 1024 * 14.8, isDisposable: true, category: 'Build' },
    ],
  },
  {
    id: 'proj-2',
    name: 'WebPlatform',
    path: 'D:\\Projects\\WebPlatform',
    ecosystem: 'Node.js',
    gitStatus: 'dirty',
    lastActiveDaysAgo: 14,
    totalArtifactSizeBytes: 1024 * 1024 * 1024 * 2.3,
    artifacts: [
      { name: 'node_modules', relativePath: 'frontend\\node_modules', sizeBytes: 1024 * 1024 * 840, isDisposable: true, category: 'Dependencies' },
      { name: '.next', relativePath: 'frontend\\.next', sizeBytes: 1024 * 1024 * 1460, isDisposable: true, category: 'Build Cache' },
    ],
  },
  {
    id: 'proj-3',
    name: 'DataPipeline',
    path: 'D:\\Projects\\DataPipeline',
    ecosystem: 'Python',
    gitStatus: 'clean',
    lastActiveDaysAgo: 45,
    totalArtifactSizeBytes: 1024 * 1024 * 1024 * 3.8,
    artifacts: [
      { name: '.venv', relativePath: '.venv', sizeBytes: 1024 * 1024 * 1024 * 3.6, isDisposable: true, category: 'Virtualenv' },
      { name: '__pycache__', relativePath: 'src\\__pycache__', sizeBytes: 1024 * 1024 * 200, isDisposable: true, category: 'Bytecode' },
    ],
  },
];

export const mockAutomations: AutomationRule[] = [
  {
    id: 'auto-1',
    name: 'Weekly Stale Downloads Triage',
    trigger: 'schedule',
    scheduleExpression: 'Every Sunday at 02:00 AM',
    criteriaDescription: 'Files in ~/Downloads older than 60 days with size > 100MB',
    targetPath: 'C:\\Users\\Developer\\Downloads',
    targetTier: 'caution',
    autoQuarantine: false, // Require manual review
    lastExecutedAt: '2026-10-25 02:00:14',
    nextScheduledAt: '2026-11-01 02:00:00',
    isEnabled: true,
  },
  {
    id: 'auto-2',
    name: 'Developer Build Cache Pruning',
    trigger: 'disk-pressure',
    scheduleExpression: 'When free space on D: < 15%',
    criteriaDescription: 'Clean cargo target/ and node_modules on projects inactive > 30 days with clean git',
    targetPath: 'D:\\Projects',
    targetTier: 'safe',
    autoQuarantine: true,
    lastExecutedAt: '2026-10-10 16:32:00',
    isEnabled: true,
  },
];

export const mockActivityLogs: ActivityLog[] = [
  {
    id: 'act-1',
    timestamp: '2026-10-28 14:15:22',
    type: 'scan',
    severity: 'info',
    title: 'Completed Volume Scan of C:',
    description: 'Indexed 412,850 files and 58,120 directories in 8.4 seconds.',
    itemCount: 412850,
    affectedBytes: 1024 * 1024 * 1024 * 418.5,
  },
  {
    id: 'act-2',
    timestamp: '2026-10-28 14:30:00',
    type: 'plan-generated',
    severity: 'info',
    title: 'Generated Cleanup Plan #001',
    description: 'Identified 21.4 GB of reclaimable space across 3 candidate targets.',
    itemCount: 3,
    affectedBytes: 1024 * 1024 * 1024 * 21.39,
  },
  {
    id: 'act-3',
    timestamp: '2026-10-26 11:05:40',
    type: 'quarantine',
    severity: 'info',
    title: 'Staged 8 Items into Quarantine Vault',
    description: 'Moved obsolete Gradle build caches to D:\\.filemind_quarantine.',
    itemCount: 8,
    affectedBytes: 1024 * 1024 * 1024 * 6.2,
    manifestId: 'man-20261026-88',
    canUndo: true,
  },
];

export const mockQuarantineRecords: QuarantineRecord[] = [
  {
    id: 'qr-1',
    manifestId: 'man-20261026-88',
    originalPath: 'D:\\Projects\\OldAndroidApp\\.gradle\\caches',
    quarantineDate: '2026-10-26 11:05:40',
    sizeBytes: 1024 * 1024 * 1024 * 4.1,
    blake3Hash: 'b3_90918ef1234acde1873199824419280149021e102',
    status: 'quarantined',
    restorable: true,
  },
  {
    id: 'qr-2',
    manifestId: 'man-20261026-88',
    originalPath: 'D:\\Projects\\OldAndroidApp\\app\\build',
    quarantineDate: '2026-10-26 11:05:40',
    sizeBytes: 1024 * 1024 * 1024 * 2.1,
    blake3Hash: 'b3_11894acdf81920387410928341029834109283410',
    status: 'quarantined',
    restorable: true,
  },
  {
    id: 'qr-3',
    manifestId: 'man-20261001-12',
    originalPath: 'C:\\Users\\Developer\\AppData\\Local\\Temp\\installer_old.tmp',
    quarantineDate: '2026-10-01 09:22:15',
    sizeBytes: 1024 * 1024 * 540,
    blake3Hash: 'b3_88192410928340192834109283410928341092834',
    status: 'purged',
    restorable: false,
  },
];

export const mockCopilotMessages: CopilotMessage[] = [
  {
    id: 'cop-1',
    sender: 'assistant',
    timestamp: '2026-10-28 14:32:00',
    content:
      'Greetings. I have analyzed your storage topology across drives C:, D:, and E:.\n\nKey finding: Your D: drive has 20.9 GB tied up in build artifacts and duplicate files that are categorized as Safe or Caution. Would you like to review the Cleanup Plan?',
    suggestedAction: {
      type: 'navigate',
      label: 'Open Cleanup Planner',
      screen: 'cleanup-planner',
    },
  },
];

export const mockNotifications: NotificationItem[] = [
  {
    id: 'notif-1',
    timestamp: '14:30',
    title: 'Cleanup Plan Ready',
    message: '21.4 GB available for safe reclamation across 3 items.',
    type: 'info',
    read: false,
    actionScreen: 'cleanup-planner',
  },
  {
    id: 'notif-2',
    timestamp: '11:15',
    title: 'Stale Scan Warning',
    message: 'Drive E: has not been scanned in over 6 days.',
    type: 'warning',
    read: false,
    actionScreen: 'overview',
  },
];

export const defaultSettings: AppSettings = {
  theme: 'light',
  defaultScanRoot: 'C:',
  lowPriorityIo: true,
  quarantineRetentionDays: 30,
  ollamaEndpoint: 'http://127.0.0.1:11434',
  ollamaModel: 'llama3:latest',
  enableCopilot: true,
  enableVoiceInput: true,
  enableVoiceOutput: false,
  voiceRate: 1.0,
  voicePitch: 1.0,
  excludedPaths: [
    'C:\\Windows',
    'C:\\Program Files\\WindowsApps',
    'C:\\System Volume Information',
  ],
  autoCheckUpdates: true,
};

