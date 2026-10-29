import React, { useState, useRef, useEffect } from 'react';
import { useApp } from '@/context/AppContext';
import { formatBytes } from '@/utils/format';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import {
  Bot,
  Send,
  ShieldCheck,
  ArrowRight,
  Mic,
  MicOff,
  Volume2,
  VolumeX,
  Sparkles,
  ChevronDown,
  ChevronRight,
  FolderOpen,
  Info,
  AlertTriangle,
  Play,
  RotateCcw,
} from 'lucide-react';
import type { CopilotMessage, CopilotEvidenceItem, CopilotToolResult } from '@/types';

export const JarvisScreen: React.FC = () => {
  const {
    copilotMessages,
    sendCopilotMessage,
    setActiveScreen,
    setSelectedFile,
    setIsInspectorOpen,
    openFileFolder,
    startScan,
    settings,
    updateSettings,
    ollamaStatus,
    fetchOllama,
    isVoiceListening,
    startVoiceInput,
    stopVoiceInput,
    stopSpeakingVoice,
  } = useApp();

  const [input, setInput] = useState('');
  const [expandedToolIds, setExpandedToolIds] = useState<Record<string, boolean>>({});
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [copilotMessages]);

  const handleSend = () => {
    if (!input.trim()) return;
    sendCopilotMessage(input.trim());
    setInput('');
  };

  const toggleToolExpanded = (toolKey: string) => {
    setExpandedToolIds((prev) => ({
      ...prev,
      [toolKey]: !prev[toolKey],
    }));
  };

  const sampleQueries = [
    'What is taking up the most space on my C drive?',
    'Find duplicate archives in Downloads.',
    'Show me the largest files in this folder.',
    'Find JavaScript projects that contain large dependency folders.',
    'Explain why this file is protected.',
    'Prepare a plan to free 10 GB.',
    'Show the files that changed most recently.',
    'Find files related to my Linux installation.',
    'Tell me which cleanup actions are reversible.',
    "Summarize today's storage scan.",
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', padding: '14px', gap: '12px' }}>
      {/* Header Toolbar */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Bot size={18} style={{ color: 'var(--accent-base)' }} />
            <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
              JARVIS — Desktop Storage Copilot
            </h2>

            {/* Provider / Model Status Badge */}
            {settings.enableCopilot ? (
              ollamaStatus?.isAvailable ? (
                <Badge variant="safe">
                  Ollama Local ({settings.ollamaModel})
                </Badge>
              ) : (
                <Badge variant="warning">
                  Local Rules Engine (Ollama Offline)
                </Badge>
              )
            ) : (
              <Badge variant="neutral">AI Disabled in Settings</Badge>
            )}
          </div>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Grounded storage inquiries over local SQLite metadata index. Zero cloud telemetry.
          </span>
        </div>

        {/* Audio & Settings Quick Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          {/* Voice Output Toggle */}
          <Button
            size="sm"
            variant="ghost"
            icon={settings.enableVoiceOutput ? <Volume2 size={13} style={{ color: 'var(--accent-base)' }} /> : <VolumeX size={13} />}
            title={settings.enableVoiceOutput ? 'Mute vocal responses' : 'Enable vocal responses'}
            onClick={() => {
              if (settings.enableVoiceOutput) {
                stopSpeakingVoice();
              }
              updateSettings({ enableVoiceOutput: !settings.enableVoiceOutput });
            }}
          >
            {settings.enableVoiceOutput ? 'Vocal' : 'Muted'}
          </Button>

          {/* Refresh Ollama models */}
          <Button
            size="sm"
            variant="ghost"
            icon={<RotateCcw size={12} />}
            onClick={() => fetchOllama()}
            title="Refresh local Ollama status"
          >
            Check Ollama
          </Button>
        </div>
      </div>

      {/* Security Architecture Isolation Notice */}
      <div
        style={{
          padding: '8px 12px',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          fontSize: 'var(--text-caption)',
          color: 'var(--text-secondary)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        }}
      >
        <ShieldCheck size={16} style={{ color: 'var(--safe-base)', flexShrink: 0 }} />
        <span>
          <strong>Grounded Security Boundary:</strong> The assistant is strictly an interface to local tools and has <strong>zero direct filesystem write privileges</strong>. Cleanups require explicit user approval in the Cleanup Planner.
        </span>
      </div>

      {/* Messages Thread Container */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '14px',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px',
        }}
      >
        {copilotMessages.map((msg: CopilotMessage) => {
          const isUser = msg.sender === 'user';

          return (
            <div
              key={msg.id}
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: isUser ? 'flex-end' : 'flex-start',
                gap: '6px',
                width: '100%',
              }}
            >
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  fontSize: 'var(--text-micro)',
                  color: 'var(--text-muted)',
                }}
              >
                <span>{isUser ? 'You' : 'JARVIS Storage Copilot'}</span>
                {msg.provider && (
                  <span style={{ color: 'var(--accent-base)', fontWeight: 500 }}>
                    via {msg.provider}
                  </span>
                )}
                <span className="tabular-nums">{msg.timestamp}</span>
              </div>

              <div
                style={{
                  maxWidth: isUser ? '75%' : '88%',
                  padding: '12px 14px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: isUser ? 'var(--accent-base)' : 'var(--bg-surface-1)',
                  color: isUser ? 'var(--accent-contrast)' : 'var(--text-primary)',
                  fontSize: 'var(--text-body)',
                  lineHeight: 1.5,
                  border: isUser ? 'none' : '1px solid var(--border-subtle)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '10px',
                }}
              >
                {/* Main Text Content */}
                <div style={{ whiteSpace: 'pre-wrap' }}>
                  {msg.content}
                </div>

                {/* Stale Index Banner with 1-Click Scan */}
                {msg.isStaleIndex && (
                  <div
                    style={{
                      padding: '8px 12px',
                      backgroundColor: 'rgba(234, 179, 8, 0.1)',
                      border: '1px solid var(--warning-border)',
                      borderRadius: 'var(--radius-sm)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      gap: '8px',
                      fontSize: 'var(--text-caption)',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <AlertTriangle size={14} style={{ color: 'var(--warning-text)', flexShrink: 0 }} />
                      <span>Index is empty or older than 7 days. Recommend running a read-only scan.</span>
                    </div>
                    <Button size="sm" variant="secondary" icon={<Play size={11} />} onClick={() => startScan()}>
                      Run Scan
                    </Button>
                  </div>
                )}

                {/* Tool Execution Activity Pills */}
                {msg.toolExecutions && msg.toolExecutions.length > 0 && (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', fontWeight: 600 }}>
                      Grounded Tool Traces ({msg.toolExecutions.length}):
                    </span>
                    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
                      {msg.toolExecutions.map((tool: CopilotToolResult, tIdx: number) => {
                        const toolKey = `${msg.id}-tool-${tIdx}`;
                        const isExpanded = !!expandedToolIds[toolKey];

                        return (
                          <div key={toolKey} style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                            <button
                              type="button"
                              onClick={() => toggleToolExpanded(toolKey)}
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '4px',
                                padding: '3px 8px',
                                borderRadius: 'var(--radius-xs)',
                                border: '1px solid var(--border-subtle)',
                                backgroundColor: 'var(--bg-surface-0)',
                                color: 'var(--text-secondary)',
                                fontSize: 'var(--text-micro)',
                                cursor: 'pointer',
                              }}
                            >
                              <Sparkles size={11} style={{ color: 'var(--accent-base)' }} />
                              <span style={{ fontWeight: 600 }}>{tool.tool}</span>
                              {isExpanded ? <ChevronDown size={11} /> : <ChevronRight size={11} />}
                            </button>

                            {isExpanded && (
                              <div
                                style={{
                                  padding: '8px',
                                  borderRadius: 'var(--radius-xs)',
                                  backgroundColor: 'var(--bg-surface-0)',
                                  border: '1px solid var(--border-subtle)',
                                  fontFamily: 'var(--font-mono)',
                                  fontSize: 'var(--text-micro)',
                                  maxHeight: '160px',
                                  overflowY: 'auto',
                                  whiteSpace: 'pre-wrap',
                                }}
                              >
                                {tool.summary}
                              </div>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  </div>
                )}

                {/* Evidence Ledger Cards */}
                {msg.evidence && msg.evidence.length > 0 && (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', marginTop: '4px' }}>
                    <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', fontWeight: 600 }}>
                      Verified Local File Evidence:
                    </span>
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: '6px' }}>
                      {msg.evidence.map((item: CopilotEvidenceItem, eIdx: number) => (
                        <div
                          key={eIdx}
                          style={{
                            padding: '8px 10px',
                            borderRadius: 'var(--radius-sm)',
                            backgroundColor: 'var(--bg-surface-0)',
                            border: '1px solid var(--border-subtle)',
                            display: 'flex',
                            flexDirection: 'column',
                            gap: '4px',
                          }}
                        >
                          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px' }}>
                            <span className="font-mono truncate" style={{ fontWeight: 600, fontSize: 'var(--text-caption)' }}>
                              {item.filename}
                            </span>
                            <Badge tier={item.riskTier} />
                          </div>

                          <div className="font-mono truncate" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }} title={item.path}>
                            {item.path}
                          </div>

                          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: '2px' }}>
                            <span className="tabular-nums" style={{ fontSize: 'var(--text-micro)', color: 'var(--text-secondary)', fontWeight: 500 }}>
                              {item.sizeBytes > 0 ? formatBytes(item.sizeBytes) : 'System Tree'}
                            </span>

                            <div style={{ display: 'flex', gap: '4px' }}>
                              <Button
                                size="sm"
                                variant="ghost"
                                icon={<Info size={11} />}
                                onClick={() => {
                                  setSelectedFile({
                                    id: `ev-${eIdx}`,
                                    path: item.path,
                                    name: item.filename,
                                    parentPath: item.path.substring(0, item.path.lastIndexOf('\\')),
                                    extension: item.filename.split('.').pop() || '',
                                    sizeBytes: item.sizeBytes,
                                    physicalSizeBytes: item.sizeBytes,
                                    category: 'Other',
                                    mtime: new Date().toISOString(),
                                    ctime: new Date().toISOString(),
                                    atime: new Date().toISOString(),
                                    attributes: {
                                      isHidden: false,
                                      isSystem: false,
                                      isReadOnly: false,
                                      isReparsePoint: false,
                                    },
                                    riskTier: item.riskTier,
                                    safetyRationale: item.reason,
                                    isLocked: false,
                                  });
                                  setIsInspectorOpen(true);
                                }}
                              >
                                Inspect
                              </Button>

                              <Button
                                size="sm"
                                variant="ghost"
                                icon={<FolderOpen size={11} />}
                                onClick={() => openFileFolder(item.path)}
                              >
                                Explorer
                              </Button>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Structured Operation Proposal Card (Strict Approval Boundary) */}
                {msg.proposal && (
                  <div
                    style={{
                      padding: '12px 14px',
                      borderRadius: 'var(--radius-md)',
                      backgroundColor: 'var(--bg-surface-0)',
                      border: '1px solid var(--accent-border)',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: '8px',
                      marginTop: '4px',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                        <ShieldCheck size={16} style={{ color: 'var(--accent-base)' }} />
                        <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                          {msg.proposal.title}
                        </span>
                      </div>
                      <Badge variant="accent">
                        {formatBytes(msg.proposal.estimatedFreedBytes)} Projected
                      </Badge>
                    </div>

                    <p style={{ fontSize: 'var(--text-caption)', color: 'var(--text-secondary)', margin: 0 }}>
                      {msg.proposal.description}
                    </p>

                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px', flexWrap: 'wrap', paddingTop: '4px', borderTop: '1px solid var(--border-subtle)' }}>
                      <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                        <strong>Reversibility:</strong> {msg.proposal.reversibility}
                      </span>

                      <Button
                        size="sm"
                        variant="primary"
                        icon={<ArrowRight size={12} />}
                        onClick={() => {
                          if (msg.proposal?.screenAction) {
                            setActiveScreen(msg.proposal.screenAction);
                          }
                        }}
                      >
                        Review Proposal in Cleanup Planner
                      </Button>
                    </div>
                  </div>
                )}

                {/* Followup Question Pills */}
                {msg.suggestedFollowups && msg.suggestedFollowups.length > 0 && (
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px', marginTop: '4px' }}>
                    {msg.suggestedFollowups.map((q: string, qIdx: number) => (
                      <button
                        key={qIdx}
                        type="button"
                        onClick={() => sendCopilotMessage(q)}
                        style={{
                          padding: '3px 8px',
                          borderRadius: 'var(--radius-xs)',
                          backgroundColor: 'var(--bg-surface-0)',
                          border: '1px solid var(--border-subtle)',
                          fontSize: 'var(--text-micro)',
                          color: 'var(--text-secondary)',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '4px',
                        }}
                      >
                        <Sparkles size={10} style={{ color: 'var(--accent-base)' }} />
                        {q}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      {/* Suggested Quick Prompts Ribbon */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', overflowX: 'auto', paddingBottom: '2px' }}>
        <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', whiteSpace: 'nowrap' }}>
          Suggested:
        </span>
        {sampleQueries.map((q: string, idx: number) => (
          <button
            key={idx}
            type="button"
            onClick={() => setInput(q)}
            style={{
              padding: '3px 8px',
              backgroundColor: 'var(--bg-surface-0)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-micro)',
              color: 'var(--text-secondary)',
              cursor: 'pointer',
              whiteSpace: 'nowrap',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.borderColor = 'var(--border-strong)')}
            onMouseLeave={(e) => (e.currentTarget.style.borderColor = 'var(--border-subtle)')}
          >
            {q}
          </button>
        ))}
      </div>

      {/* Active Voice Listening Banner */}
      {isVoiceListening && (
        <div
          style={{
            padding: '8px 12px',
            backgroundColor: 'var(--accent-subtle)',
            border: '1px solid var(--accent-border)',
            borderRadius: 'var(--radius-sm)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            fontSize: 'var(--text-caption)',
            color: 'var(--accent-active)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span className="pulse-dot" style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: 'var(--danger-base)' }} />
            <span>Listening... Speak your storage request clearly into your microphone.</span>
          </div>
          <Button size="sm" variant="danger" icon={<MicOff size={11} />} onClick={stopVoiceInput}>
            Cancel Listening
          </Button>
        </div>
      )}

      {/* Chat Input Bar */}
      <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
        {/* Voice Input Button */}
        <Button
          variant={isVoiceListening ? 'danger' : 'secondary'}
          icon={isVoiceListening ? <MicOff size={14} /> : <Mic size={14} />}
          onClick={() => {
            if (isVoiceListening) {
              stopVoiceInput();
            } else {
              startVoiceInput();
            }
          }}
          title={isVoiceListening ? 'Stop listening' : 'Start voice input (Click-to-talk)'}
        />

        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleSend();
          }}
          placeholder="Ask an advisory storage question grounded in your local index..."
          style={{
            flex: 1,
            padding: '8px 12px',
            fontSize: 'var(--text-body)',
            backgroundColor: 'var(--bg-surface-0)',
            borderColor: 'var(--border-subtle)',
            borderRadius: 'var(--radius-sm)',
          }}
        />

        <Button variant="primary" icon={<Send size={13} />} onClick={handleSend} disabled={!input.trim()}>
          Ask
        </Button>
      </div>
    </div>
  );
};
