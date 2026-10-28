import React, { useState, useRef, useEffect } from 'react';
import { useApp } from '@/context/AppContext';
import { Badge } from '@/components/common/Badge';
import { Button } from '@/components/common/Button';
import { Bot, Send, ShieldCheck, ArrowRight } from 'lucide-react';
import type { CopilotMessage } from '@/types';

export const JarvisScreen: React.FC = () => {
  const { copilotMessages, sendCopilotMessage, setActiveScreen, settings } = useApp();
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [copilotMessages]);

  const handleSend = () => {
    if (!input.trim()) return;
    sendCopilotMessage(input.trim());
    setInput('');
  };

  const sampleQueries = [
    'What is taking up the most space on D:?',
    'Why is ext4.vhdx locked and cannot be deleted?',
    'Find all inactive developer build caches older than 30 days',
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', padding: '14px', gap: '12px' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Bot size={18} style={{ color: 'var(--accent-base)' }} />
            <h2 style={{ fontSize: 'var(--text-subhead)', fontWeight: 600, color: 'var(--text-primary)' }}>
              JARVIS — Local Advisory Copilot
            </h2>
            <Badge variant="safe">Ollama Local ({settings.ollamaModel})</Badge>
          </div>
          <span style={{ fontSize: 'var(--text-caption)', color: 'var(--text-muted)' }}>
            Natural-language inquiries over local metadata index. Zero cloud telemetry.
          </span>
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
          <strong>Strict Security Boundary:</strong> The copilot has <strong>zero direct filesystem write privileges</strong>. Suggested actions generate typed proposals requiring deterministic Safety Engine validation and explicit user confirmation.
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
          gap: '12px',
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
                gap: '4px',
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
                <span>{isUser ? 'You' : 'JARVIS Advisory'}</span>
                <span className="tabular-nums">{msg.timestamp}</span>
              </div>

              <div
                style={{
                  maxWidth: '75%',
                  padding: '10px 12px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: isUser ? 'var(--accent-base)' : 'var(--bg-surface-1)',
                  color: isUser ? 'var(--accent-contrast)' : 'var(--text-primary)',
                  fontSize: 'var(--text-body)',
                  lineHeight: 1.5,
                  whiteSpace: 'pre-wrap',
                  border: isUser ? 'none' : '1px solid var(--border-subtle)',
                }}
              >
                {msg.content}

                {/* Structured Suggestion Action Button */}
                {msg.suggestedAction && (
                  <div style={{ marginTop: '8px', paddingTop: '6px', borderTop: '1px solid var(--border-subtle)' }}>
                    <Button
                      size="sm"
                      variant="primary"
                      icon={<ArrowRight size={11} />}
                      onClick={() => {
                        if (msg.suggestedAction?.screen) {
                          setActiveScreen(msg.suggestedAction.screen);
                        }
                      }}
                    >
                      {msg.suggestedAction.label}
                    </Button>
                  </div>
                )}
              </div>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      {/* Suggested Quick Prompts */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', overflowX: 'auto' }}>
        <span style={{ fontSize: 'var(--text-micro)', color: 'var(--text-muted)', whiteSpace: 'nowrap' }}>
          Suggested:
        </span>
        {sampleQueries.map((q: string, idx: number) => (
          <button
            key={idx}
            type="button"
            onClick={() => setInput(q)}
            style={{
              padding: '2px 8px',
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

      {/* Chat Input Bar */}
      <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleSend();
          }}
          placeholder="Ask an advisory question about your storage topology..."
          style={{
            flex: 1,
            padding: '7px 12px',
            fontSize: 'var(--text-body)',
            backgroundColor: 'var(--bg-surface-0)',
            borderColor: 'var(--border-subtle)',
          }}
        />
        <Button variant="primary" icon={<Send size={13} />} onClick={handleSend}>
          Send
        </Button>
      </div>
    </div>
  );
};
