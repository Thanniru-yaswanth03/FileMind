// FileMind Domain Type Definitions

export type NavScreen =
  | 'overview'
  | 'storage-explorer'
  | 'search'
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
  ruleId?: string;
  name: string;
  description?: string;
  trigger: 'schedule' | 'disk-pressure' | 'folder-watch' | 'manual' | any;
  scheduleExpression: string;
  criteriaDescription: string;
  targetPath?: string;
  targetTier?: RiskTier;
  autoQuarantine?: boolean;
  lastExecutedAt?: string;
  nextScheduledAt?: string;
  isEnabled: boolean;
  requiresExplicitApproval?: boolean;
  dryRunMode?: boolean;
  scopePaths?: string[];
  exclusionPaths?: string[];
  conditions?: any[];
  actions?: any[];
  lastRunAt?: number;
  lastStatus?: string;
}

export interface AutomationExecutionHistory {
  runId: string;
  ruleId: string;
  ruleName: string;
  triggerSource: string;
  startedAt: number;
  completedAt?: number;
  status: 'success' | 'partial' | 'blocked' | 'failed' | 'dry_run_completed' | 'skipped' | string;
  inputsSummary: string;
  actionsAttempted: string[];
  policyDecisions: string[];
  resultSummary: string;
  errorMessage?: string;
  retriesAttempted?: number;
  filesAffectedCount: number;
  bytesAffected: number;
  planId?: string;
}

export interface SchedulerStatusResponse {
  isRunning: boolean;
  activeWatchersCount: number;
  watchedPaths: string[];
  scheduledRulesCount: number;
  windowsScheduledTaskConfigured: boolean;
  windowsScheduledTaskName: string;
  operatingMode: string;
}

export interface DeveloperArtifactDto {
  name: string;
  relativePath: string;
  fullPath: string;
  sizeBytes: number;
  category: string;
  isRegenerable: boolean;
  reversibility: string;
  cleanupProcedure: string;
  riskTier: RiskTier;
  risksAndQuestions: string;
}

export interface DeveloperProjectDto {
  id: string;
  name: string;
  path: string;
  ecosystem: string;
  manifestFile: string;
  lockfileFile?: string;
  gitStatus: 'clean' | 'dirty' | 'untracked_work' | 'no_repo' | string;
  totalArtifactSizeBytes: number;
  artifacts: DeveloperArtifactDto[];
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
  quarantinePath?: string;
  quarantineDate: string;
  sizeBytes: number;
  blake3Hash: string;
  status: 'quarantined' | 'restored' | 'purged';
  restorable: boolean;
  reason?: string;
  restoreError?: string;
}

export interface CopilotToolResult {
  tool: string;
  success: boolean;
  summary: string;
  data: any;
  error?: string;
}

export interface CopilotEvidenceItem {
  path: string;
  filename: string;
  sizeBytes: number;
  riskTier: RiskTier;
  reason: string;
  actionType: string;
}

export interface CopilotOperationProposal {
  planId?: string;
  title: string;
  description: string;
  estimatedFreedBytes: number;
  targetPathsCount: number;
  reversibility: string;
  requiresExplicitApproval: boolean;
  screenAction: NavScreen;
}

export interface CopilotMessage {
  id: string;
  sender: 'user' | 'assistant';
  timestamp: string;
  content: string;
  toolExecutions?: CopilotToolResult[];
  evidence?: CopilotEvidenceItem[];
  proposal?: CopilotOperationProposal;
  provider?: string;
  isStaleIndex?: boolean;
  suggestedFollowups?: string[];
  suggestedAction?: {
    type: 'navigate' | 'create-plan' | 'inspect';
    label: string;
    screen?: NavScreen;
    targetPath?: string;
  };
}

export interface OllamaModelInfo {
  name: string;
  sizeBytes: number;
  modifiedAt: string;
  estimatedRamMb: number;
  parameterSize?: string;
  family?: string;
  description: string;
}

export interface OllamaStatusResponse {
  isAvailable: boolean;
  endpoint: string;
  models: OllamaModelInfo[];
  activeModel?: string;
  error?: string;
}

export interface CopilotQueryResponse {
  answer: string;
  toolExecutions: CopilotToolResult[];
  evidence: CopilotEvidenceItem[];
  proposal?: CopilotOperationProposal;
  provider: string;
  isStaleIndex: boolean;
  suggestedFollowups: string[];
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
  enableVoiceInput: boolean;
  enableVoiceOutput: boolean;
  voiceRate: number;
  voicePitch: number;
  excludedPaths: string[];
  autoCheckUpdates: boolean;
}

export interface ScanErrorItem {
  id: number;
  sessionId: string;
  path: string;
  errorKind: 'permission_denied' | 'locked' | 'disappeared' | 'path_too_long' | 'io_error' | 'reparse_policy' | 'other';
  message: string;
  occurredAt: string;
}

export interface StorageAccounting {
  volumeTotalBytes: number;
  volumeFreeBytes: number;
  volumeUsedBytes: number;
  logicalSizeBytes: number;
  physicalSizeBytes: number;
  indexedFilesTotal: number;
  inaccessibleRegionsCount: number;
  exactFiguresNote: string;
  estimatedFiguresNote: string;
  hardlinksDeduplicatedCount: number;
  hardlinksBytesSaved: number;
}

export interface ScanSessionItem {
  sessionId: string;
  rootPath: string;
  startedAt: string;
  completedAt?: string;
  status: 'running' | 'completed' | 'cancelled' | 'failed' | 'partial';
  totalFiles: number;
  totalDirectories: number;
  logicalSizeBytes: number;
  physicalSizeBytes: number;
  errorCount: number;
}

export interface ScanSummary {
  session: ScanSessionItem;
  storageAccounting: StorageAccounting;
  recentErrors: ScanErrorItem[];
}

export interface FolderRanking {
  path: string;
  totalSizeBytes: number;
  fileCount: number;
  dirCount: number;
  percentageOfTotal: number;
}

export interface CategoryBreakdown {
  category: FileCategory;
  totalBytes: number;
  fileCount: number;
  percentage: number;
}

// ---------------------------------------------------------------------------
// Phase 3: Duplicate Detection & Content Search
// ---------------------------------------------------------------------------

export type DuplicateConfidence =
  | 'unverified'
  | 'size_match_only'
  | 'prefix_hash_match'
  | 'cryptographic_hash_match'
  | 'byte_for_byte_verified';

export type DuplicateFileStatus =
  | 'ok'
  | 'changed_during_hash'
  | 'inaccessible'
  | 'lock_error'
  | 'disappeared';

export type LocationTag =
  | 'recommended_keep'
  | 'protected_system'
  | 'project_member'
  | 'user_preferred'
  | 'backup_location'
  | 'redundant_copy';

export interface DuplicateFileItem {
  id: number | string;
  path: string;
  filename: string;
  directoryPath: string;
  sizeBytes: number;
  mtimeEpochMs: number;
  formattedMtime: string;
  blake3Hash?: string;
  status: DuplicateFileStatus;
  locationTags: LocationTag[];
  isRecommendedKeep: boolean;
  recommendationRationale: string;
  isSelectedForRetention: boolean;
  isProtected: boolean;
}

export interface RealDuplicateGroup {
  groupId: string;
  sizeBytes: number;
  fileCount: number;
  wastedBytes: number;
  confidence: DuplicateConfidence;
  hash?: string;
  groupRationale: string;
  files: DuplicateFileItem[];
}

export interface DuplicateScanResult {
  groups: RealDuplicateGroup[];
  totalDuplicateFiles: number;
  totalWastedBytes: number;
  unverifiedCandidatesCount: number;
  verifiedGroupsCount: number;
}

export interface ContentSearchResultItem {
  fileId: number;
  path: string;
  filename: string;
  extension: string;
  sizeBytes: number;
  mtimeEpochMs: number;
  snippet: string;
  isStale: boolean;
  status: string;
}

export interface ContentSearchResponse {
  totalMatches: number;
  results: ContentSearchResultItem[];
  executionTimeMs: number;
}

export interface ContentIndexStatus {
  totalIndexedDocuments: number;
  totalExtractedCharacters: number;
  skippedSensitiveCount: number;
  truncatedDocumentsCount: number;
  isActive: boolean;
}

// ---------------------------------------------------------------------------
// Phase 4: Smart Delete Guardian and Dependency Intelligence
// ---------------------------------------------------------------------------

export type SafetyClassification =
  | 'LowRiskCandidate'
  | 'ReviewRequired'
  | 'HighRiskOrUnknown'
  | 'ProtectedAndBlocked';

export type AllowedOperation =
  | 'permitted'
  | 'requires_explicit_user_confirmation'
  | 'blocked';

export type EvidenceSource =
  | 'windows_policy'
  | 'wsl_detector'
  | 'developer_manifest'
  | 'git_repository'
  | 'file_system_attributes'
  | 'lock_inspection'
  | 'toctou_verification'
  | 'user_exclusion';

export interface TypedEvidenceItem {
  source: EvidenceSource;
  claim: string;
  supportsSafety: boolean;
  verificationStatus: string;
  details: string;
}

export interface SafetyAssessment {
  targetPath: string;
  canonicalPath: string;
  fileIdentity?: string;
  isDirectory: boolean;
  sizeBytes: number;
  classification: SafetyClassification;
  allowedOperation: AllowedOperation;
  isPermitted: boolean;
  ruleMatches: string[];
  missingEvidence: string[];
  consequences: string[];
  recoveryPossibility: string;
  blockReason?: string;
  evidence: TypedEvidenceItem[];
  evaluatedAtEpochMs: number;
}

export interface SafetyAuditRecord {
  auditId: number;
  targetPath: string;
  canonicalPath: string;
  operation: string;
  classification: SafetyClassification;
  isPermitted: boolean;
  ruleMatches: string[];
  evidence: TypedEvidenceItem[];
  blockReasons: string[];
  toctouVerified: boolean;
  executed: boolean;
  executionResult?: string;
  timestampEpochMs: number;
}

export interface SafetyEvaluationResponse {
  assessments: SafetyAssessment[];
  totalEvaluated: number;
  permittedCount: number;
  reviewRequiredCount: number;
  blockedCount: number;
}

export interface CleanupExecutionResult {
  path: string;
  success: boolean;
  error?: string;
  toctouVerified: boolean;
  methodUsed: string;
}

export interface ExecuteCleanupResponse {
  results: CleanupExecutionResult[];
  totalSucceeded: number;
  totalFailed: number;
  totalBytesReclaimed: number;
}

// ---------------------------------------------------------------------------
// Phase 5: Intelligent Cleanup Planning & Quarantine Recovery
// ---------------------------------------------------------------------------

export type CandidateSource =
  | 'developer_artifact'
  | 'duplicate_group'
  | 'large_file'
  | 'temporary_artifact'
  | 'user_selected';

export type PlanItemOperation = 'quarantine' | 'recycle' | 'permanent';

export type PlanItemState =
  | 'pending'
  | 'dry_run_ok'
  | 'executing'
  | 'succeeded'
  | 'failed'
  | 'skipped_toctou'
  | 'skipped_conflict';

export type PlanStatus =
  | 'draft'
  | 'dry_run_validated'
  | 'executing'
  | 'completed'
  | 'partially_completed'
  | 'aborted'
  | 'failed';

export interface CleanupPlanItemDto {
  itemId: string;
  planId: string;
  path: string;
  canonicalPath: string;
  logicalSizeBytes: number;
  physicalSizeBytes: number;
  source: CandidateSource;
  riskTier: RiskTier;
  classification: SafetyClassification;
  operation: PlanItemOperation;
  isReversible: boolean;
  state: PlanItemState;
  isSelected: boolean;
  isExcluded: boolean;
  quarantineId?: string;
  errorMessage?: string;
  evidenceSummary: string;
  potentialImpact: string;
}

export interface CleanupPlanDto {
  planId: string;
  createdAtEpochMs: number;
  goalType: string;
  targetBytes?: number;
  status: PlanStatus;
  totalPlannedBytes: number;
  totalPhysicalReclaimableBytes: number;
  totalReclaimedBytes: number;
  itemsCount: number;
  completedCount: number;
  failedCount: number;
  isTargetAttainableSafely: boolean;
  unmetTargetExplanation?: string;
  riskCounts: Record<string, number>;
  items: CleanupPlanItemDto[];
}

export interface DryRunItemPrediction {
  itemId: string;
  path: string;
  operation: PlanItemOperation;
  canProceed: boolean;
  reversibility: string;
  expectedFreedBytes: number;
  warning?: string;
}

export interface DryRunResponse {
  planId: string;
  canExecute: boolean;
  validatedItemsCount: number;
  totalExpectedFreedBytes: number;
  conflictsDetected: string[];
  predictions: DryRunItemPrediction[];
}

export interface ExecutePlanResponse {
  planId: string;
  status: PlanStatus;
  succeededCount: number;
  failedCount: number;
  totalBytesReclaimed: number;
  quarantinedCount: number;
  recycledCount: number;
  deletedCount: number;
  errors: string[];
}

export interface QuarantineRecordDto {
  quarantineId: string;
  originalPath: string;
  canonicalPath: string;
  quarantinePath: string;
  filename: string;
  sizeBytes: number;
  originalCtime: number;
  originalMtime: number;
  attributes: number;
  blake3Hash?: string;
  quarantinedAtEpochMs: number;
  status: 'quarantined' | 'restored' | 'purged';
  reason: string;
  restoredAtEpochMs?: number;
  restoreError?: string;
}

export interface RestoreItemResult {
  quarantineId: string;
  originalPath: string;
  restoredPath: string;
  success: boolean;
  error?: string;
}

export interface RestoreResponse {
  results: RestoreItemResult[];
  totalRestored: number;
  totalFailed: number;
}

export interface PurgeResponse {
  purgedCount: number;
  bytesPermanentlyFreed: number;
}





