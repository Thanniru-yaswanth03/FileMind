import React from 'react';
import { useApp } from '@/context/AppContext';
import { Button } from '@/components/common/Button';
import { Badge } from '@/components/common/Badge';
import {
  Sun,
  Moon,
  Bot,
  Trash2,
} from 'lucide-react';

interface ThemeOption {
  id: 'light' | 'dark';
  label: string;
  desc: string;
  icon: React.ReactNode;
}

export const SettingsScreen: React.FC = () => {
  const { theme, setTheme, settings, updateSettings, showToast } = useApp();

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

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '16px', gap: '20px' }}>
      {/* Header */}
      <div>
        <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
          Application Settings & Preferences
        </h2>
        <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
          Configure desktop environment, background I/O priority, local AI providers, and scan exclusions.
        </span>
      </div>

      {/* Section 1: Appearance & Design Tokens */}
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

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '10px' }}>
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

      {/* Section 2: Storage Scanning & Resource Priority */}
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

        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
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
                Ensures scans run with lowest disk and CPU priority, preventing micro-stutters during gaming or compiling.
              </div>
            </div>
          </label>

          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingTop: '8px', borderTop: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-caption)', color: 'var(--text-primary)' }}>
                Quarantine Retention Policy
              </div>
              <div style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)' }}>
                Retain staged files in .filemind_quarantine before prompting for permanent purge.
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

      {/* Section 3: Scan Exclusions List */}
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
              Paths skipped unconditionally during indexing and automated workflows.
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

      {/* Section 4: Optional Local Copilot (Ollama) */}
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
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Bot size={16} style={{ color: 'var(--accent-base)' }} />
            <span style={{ fontWeight: 600, fontSize: 'var(--text-ui)', color: 'var(--text-primary)' }}>
              Optional Local AI Copilot (Ollama)
            </span>
          </div>
          <Badge variant="safe">Localhost Only (127.0.0.1)</Badge>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
          <div>
            <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
              Ollama Server Endpoint
            </label>
            <input
              type="text"
              value={settings.ollamaEndpoint}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ ollamaEndpoint: e.target.value })}
              style={{ width: '100%' }}
            />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: 'var(--text-caption)', fontWeight: 500, marginBottom: '4px' }}>
              Model Identifier
            </label>
            <input
              type="text"
              value={settings.ollamaModel}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateSettings({ ollamaModel: e.target.value })}
              style={{ width: '100%' }}
            />
          </div>
        </div>
      </div>

      {/* Section 5: Data & Index Management */}
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
            Re-initializes local SQLite WAL database. Preserves quarantine records and user settings.
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
