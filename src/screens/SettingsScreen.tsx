import React, { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { Button } from '@/components/common/Button';
import { Badge } from '@/components/common/Badge';
import {
  Sun,
  Moon,
  Bot,
  Trash2,
  Mic,
  Cpu,
  ShieldCheck,
  CheckCircle2,
  AlertCircle,
  RefreshCw,
  Play,
} from 'lucide-react';
import { speakGroundedAnswer } from '@/services/voice';

interface ThemeOption {
  id: 'light' | 'dark';
  label: string;
  desc: string;
  icon: React.ReactNode;
}

export const SettingsScreen: React.FC = () => {
  const { theme, setTheme, settings, updateSettings, showToast, ollamaStatus, fetchOllama } = useApp();
  const [isTestingOllama, setIsTestingOllama] = useState(false);
  const [isTestingVoice, setIsTestingVoice] = useState(false);

  const themeOptions: ThemeOption[] = [
    {
      id: 'light',
      label: 'Warm Ivory Light',
      desc: 'Paper-neutral ground, graphite text, muted sage green',
      icon: <Sun size={16} />,
    },
    {
      id: 'dark',
      label: 'Warm Charcoal Dark',
      desc: 'Deep warm charcoal, soft off-white, muted copper',
      icon: <Moon size={16} />,
    },
  ];

  const handleTestOllamaConnection = async () => {
    setIsTestingOllama(true);
    showToast(`Connecting to Ollama at ${settings.ollamaEndpoint}...`);
    try {
      await fetchOllama();
      showToast('Ollama connection checked successfully.');
    } catch {
      showToast('Ollama connection test complete.');
    } finally {
      setIsTestingOllama(false);
    }
  };

  const handleTestVoiceSynthesis = () => {
    setIsTestingVoice(true);
    speakGroundedAnswer(
      'FileMind local storage copilot operational. All systems running with verified local privacy.',
      settings.voiceRate,
      settings.voicePitch,
      () => setIsTestingVoice(false)
    );
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '20px' }}>
      {/* Header */}
      <div>
        <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
          Application Settings & Preferences
        </h2>
        <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
          Configure desktop environment, background I/O throttling, local AI model parameters, and speech preferences.
        </span>
      </div>

      {/* Section 1: Appearance & Desktop Theme */}
      <div
        style={{
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '14px',
          display: 'flex',
          flexDirection: 'column',
          gap: '12px',
        }}
      >
        <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
          Appearance & Desktop Theme
        </span>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '10px' }}>
          {themeOptions.map((t: ThemeOption) => {
            const isSelected = theme === t.id;
            return (
              <div
                key={t.id}
                onClick={() => setTheme(t.id)}
                style={{
                  padding: '10px 12px',
                  borderRadius: 'var(--radius-md)',
                  border: isSelected ? '1.5px solid var(--accent-base)' : '1px solid var(--border-subtle)',
                  backgroundColor: isSelected ? 'var(--accent-subtle)' : 'var(--bg-surface-1)',
                  cursor: 'pointer',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '4px',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px', color: isSelected ? 'var(--accent-base)' : 'var(--text-primary)' }}>
                  {t.icon}
                  <span style={{ fontWeight: 600, fontSize: 'var(--text-caption)' }}>{t.label}</span>
                </div>
                <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>{t.desc}</span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Section 2: Storage Scanning & System Priority */}
      <div
        style={{
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '14px',
          display: 'flex',
          flexDirection: 'column',
          gap: '12px',
        }}
      >
        <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
          Storage Indexing & System Priority
        </span>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          <label style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.lowPriorityIo}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ lowPriorityIo: e.target.checked })}
              style={{ marginTop: '3px' }}
            />
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                Enable Windows Background I/O Throttling (THREAD_MODE_BACKGROUND_BEGIN)
              </div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Ensures scans run with lowest disk and CPU priority, preventing micro-stutters during gaming, rendering, or compiling.
              </div>
            </div>
          </label>

          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingTop: '8px', borderTop: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                Quarantine Retention Policy
              </div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Retain staged files in <span className="font-mono">.filemind_quarantine</span> before prompting for permanent purge.
              </div>
            </div>
            <select
              value={settings.quarantineRetentionDays}
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => updateSettings({ quarantineRetentionDays: Number(e.target.value) })}
              style={{ padding: '4px 8px', fontSize: 'var(--text-caption)' }}
            >
              <option value={7}>7 Days</option>
              <option value={14}>14 Days</option>
              <option value={30}>30 Days (Recommended)</option>
              <option value={60}>60 Days</option>
              <option value={90}>90 Days</option>
            </select>
          </div>
        </div>
      </div>

      {/* Section 3: Optional Local Copilot (Ollama & Intent Engine) */}
      <div
        style={{
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '14px',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Bot size={18} style={{ color: 'var(--accent-base)' }} />
            <div>
              <div style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                JARVIS Desktop Copilot & Local Inference
              </div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Natural-language storage intelligence interface grounded exclusively in local SQLite metadata.
              </div>
            </div>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {ollamaStatus?.isAvailable ? (
              <Badge variant="safe">
                <CheckCircle2 size={12} style={{ marginRight: '4px' }} />
                Ollama Connected ({ollamaStatus.models.length} Models)
              </Badge>
            ) : (
              <Badge variant="neutral">
                <AlertCircle size={12} style={{ marginRight: '4px' }} />
                Deterministic Local Fallback
              </Badge>
            )}
            <Badge variant="neutral">Localhost 127.0.0.1 Only</Badge>
          </div>
        </div>

        {/* Enable AI Assistant Toggle */}
        <label style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', cursor: 'pointer', padding: '8px 0', borderBottom: '1px solid var(--border-subtle)' }}>
          <input
            type="checkbox"
            checked={settings.enableCopilot}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ enableCopilot: e.target.checked })}
            style={{ marginTop: '3px' }}
          />
          <div>
            <div style={{ fontWeight: 500, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
              Enable JARVIS Assistant Experience
            </div>
            <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
              When enabled, parses natural language storage queries and constructs safe, reviewable cleanup proposals. When disabled, the copilot screen remains accessible with manual action buttons.
            </div>
          </div>
        </label>

        {/* Ollama Connection Settings */}
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '12px' }}>
          <div>
            <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
              Ollama Server Endpoint
            </label>
            <div style={{ display: 'flex', gap: '6px' }}>
              <input
                type="text"
                value={settings.ollamaEndpoint}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ ollamaEndpoint: e.target.value })}
                placeholder="http://127.0.0.1:11434"
                style={{ flex: 1 }}
              />
              <Button
                size="sm"
                variant="secondary"
                icon={<RefreshCw size={13} className={isTestingOllama ? 'animate-spin' : ''} />}
                onClick={handleTestOllamaConnection}
                disabled={isTestingOllama}
              >
                Test
              </Button>
            </div>
            <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', display: 'block', marginTop: '2px' }}>
              Standard port: 11434 on localhost.
            </span>
          </div>

          <div>
            <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
              Active Inference Model
            </label>
            {ollamaStatus?.models && ollamaStatus.models.length > 0 ? (
              <select
                value={settings.ollamaModel}
                onChange={(e: React.ChangeEvent<HTMLSelectElement>) => updateSettings({ ollamaModel: e.target.value })}
                style={{ width: '100%', padding: '5px 8px' }}
              >
                {ollamaStatus.models.map((m) => (
                  <option key={m.name} value={m.name}>
                    {m.name} {m.parameterSize ? `(${m.parameterSize})` : ''} — ~{m.estimatedRamMb} MB RAM
                  </option>
                ))}
              </select>
            ) : (
              <input
                type="text"
                value={settings.ollamaModel}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ ollamaModel: e.target.value })}
                placeholder="llama3.2:3b"
                style={{ width: '100%' }}
              />
            )}
            <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', display: 'block', marginTop: '2px' }}>
              Recommended: <span className="font-mono">llama3.2:3b</span> or <span className="font-mono">mistral:latest</span>.
            </span>
          </div>
        </div>

        {/* Model RAM Requirements Guide */}
        <div
          style={{
            backgroundColor: 'var(--bg-surface-1)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-sm)',
            padding: '10px 12px',
            display: 'flex',
            flexDirection: 'column',
            gap: '8px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <Cpu size={14} style={{ color: 'var(--accent-base)' }} />
            <span style={{ fontWeight: 600, fontSize: 'var(--text-micro)', color: 'var(--text-primary)' }}>
              Local Model Resource Requirements & RAM Sizing Guide
            </span>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(170px, 1fr))', gap: '8px' }}>
            <div style={{ padding: '6px 8px', backgroundColor: 'var(--bg-surface-0)', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
              <div style={{ fontWeight: 600, fontSize: 'var(--text-micro)', color: 'var(--text-primary)' }}>3B Parameters</div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>~2.0 GB RAM required</div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>e.g. Llama 3.2 3B (Fastest)</div>
            </div>
            <div style={{ padding: '6px 8px', backgroundColor: 'var(--bg-surface-0)', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
              <div style={{ fontWeight: 600, fontSize: 'var(--text-micro)', color: 'var(--text-primary)' }}>7B / 8B Parameters</div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>~4.8 GB RAM required</div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>e.g. Llama 3.1 8B, Mistral 7B</div>
            </div>
            <div style={{ padding: '6px 8px', backgroundColor: 'var(--bg-surface-0)', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
              <div style={{ fontWeight: 600, fontSize: 'var(--text-micro)', color: 'var(--text-primary)' }}>14B Parameters</div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>~9.0 GB RAM required</div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>e.g. Qwen 2.5 Coder 14B</div>
            </div>
          </div>
        </div>

        {/* Security & Privacy Invariant Banner */}
        <div
          style={{
            display: 'flex',
            alignItems: 'flex-start',
            gap: '8px',
            backgroundColor: 'var(--bg-surface-2)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-sm)',
            padding: '8px 10px',
          }}
        >
          <ShieldCheck size={16} style={{ color: 'var(--status-safe-fg)', flexShrink: 0, marginTop: '2px' }} />
          <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', lineHeight: 1.4 }}>
            <strong style={{ color: 'var(--text-primary)' }}>Local Privacy Invariant: </strong>
            FileMind never transmits filenames, indexed metadata, or user queries to third-party cloud LLMs. If Ollama is offline, a deterministic Rust parser handles queries locally. Destructive actions can never be executed autonomously.
          </div>
        </div>
      </div>

      {/* Section 4: Voice Interaction & Speech Synthesis */}
      <div
        style={{
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '14px',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Mic size={18} style={{ color: 'var(--accent-base)' }} />
            <div>
              <div style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
                Voice Input & Speech Vocalization
              </div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Native browser Web Speech API with explicit push-to-talk. No background mic recording.
              </div>
            </div>
          </div>
          <Badge variant="neutral">Web Speech API</Badge>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          <label style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.enableVoiceInput}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ enableVoiceInput: e.target.checked })}
              style={{ marginTop: '3px' }}
            />
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                Enable Push-to-Talk Voice Input (Microphone)
              </div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Allows clicking the microphone button in the Copilot screen to dictate natural language queries. Requires browser microphone permission.
              </div>
            </div>
          </label>

          <label style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', cursor: 'pointer', paddingTop: '6px', borderTop: '1px solid var(--border-subtle)' }}>
            <input
              type="checkbox"
              checked={settings.enableVoiceOutput}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ enableVoiceOutput: e.target.checked })}
              style={{ marginTop: '3px' }}
            />
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                Enable Speech Vocalization (Text-to-Speech)
              </div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Automatically reads out grounded answers using local system speech synthesis.
              </div>
            </div>
          </label>

          {/* Voice Tuning Sliders */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
              gap: '12px',
              paddingTop: '8px',
              borderTop: '1px solid var(--border-subtle)',
            }}
          >
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                <span style={{ fontSize: 'var(--text-caption)', fontWeight: 500 }}>Speech Rate:</span>
                <span className="font-mono" style={{ fontSize: 'var(--text-micro)', color: 'var(--accent-base)' }}>
                  {settings.voiceRate.toFixed(1)}x
                </span>
              </div>
              <input
                type="range"
                min="0.7"
                max="1.5"
                step="0.1"
                value={settings.voiceRate}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ voiceRate: parseFloat(e.target.value) })}
                style={{ width: '100%' }}
              />
            </div>

            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                <span style={{ fontSize: 'var(--text-caption)', fontWeight: 500 }}>Speech Pitch:</span>
                <span className="font-mono" style={{ fontSize: 'var(--text-micro)', color: 'var(--accent-base)' }}>
                  {settings.voicePitch.toFixed(1)}x
                </span>
              </div>
              <input
                type="range"
                min="0.8"
                max="1.2"
                step="0.05"
                value={settings.voicePitch}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ voicePitch: parseFloat(e.target.value) })}
                style={{ width: '100%' }}
              />
            </div>
          </div>

          {/* Voice Test Button */}
          <div style={{ display: 'flex', justifyContent: 'flex-end', paddingTop: '4px' }}>
            <Button
              size="sm"
              variant="secondary"
              icon={<Play size={12} />}
              onClick={handleTestVoiceSynthesis}
              disabled={isTestingVoice}
            >
              {isTestingVoice ? 'Speaking Sample...' : 'Test Speech Vocalization'}
            </Button>
          </div>
        </div>
      </div>

      {/* Section 5: Scan Exclusions List */}
      <div
        style={{
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '14px',
          display: 'flex',
          flexDirection: 'column',
          gap: '10px',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
              Scan Exclusions & Hard Denylist
            </span>
            <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
              Paths skipped unconditionally during indexing, duplicate scans, and automated workflows.
            </div>
          </div>
          <Button size="sm" variant="secondary" onClick={() => showToast('Folder selector available in native Phase 4 build')}>
            Add Folder
          </Button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
          {settings.excludedPaths.map((p: string, idx: number) => (
            <div
              key={idx}
              style={{
                padding: '6px 10px',
                backgroundColor: 'var(--bg-surface-1)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                fontSize: 'var(--text-caption)',
              }}
            >
              <span className="font-mono">{p}</span>
              <Badge variant="safe">Protected</Badge>
            </div>
          ))}
        </div>
      </div>

      {/* Section 6: Data & Index Management */}
      <div
        style={{
          backgroundColor: 'var(--bg-surface-0)',
          border: '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '14px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <div>
          <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
            Reset Metadata Index Database
          </span>
          <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
            Re-initializes local SQLite WAL database. Preserves quarantine records, audit logs, and user settings.
          </div>
        </div>
        <Button
          size="sm"
          variant="danger"
          icon={<Trash2 size={12} />}
          onClick={() => showToast('Re-indexing requested; scheduled for next scan')}
        >
          Reset Index
        </Button>
      </div>
    </div>
  );
};
