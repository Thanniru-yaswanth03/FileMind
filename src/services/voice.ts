/**
 * FileMind Voice Service
 * Zero cloud transmission: Uses browser native Web Speech API (SpeechRecognition & SpeechSynthesis).
 * Strictly requires explicit user interaction (click-to-talk). No continuous background recording.
 */

// Extend window for Web Speech API types
interface IWindow extends Window {
  webkitSpeechRecognition?: any;
  SpeechRecognition?: any;
}

export type VoiceState = 'idle' | 'listening' | 'processing' | 'speaking' | 'error';

export interface VoiceError {
  code: string;
  message: string;
}

let activeRecognition: any = null;

export const isSpeechRecognitionSupported = (): boolean => {
  if (typeof window === 'undefined') return false;
  const win = window as unknown as IWindow;
  return !!(win.SpeechRecognition || win.webkitSpeechRecognition);
};

export const isSpeechSynthesisSupported = (): boolean => {
  if (typeof window === 'undefined') return false;
  return 'speechSynthesis' in window;
};

/**
 * Starts a single-turn voice listening session.
 */
export const startVoiceRecognition = (
  onResult: (transcript: string) => void,
  onError: (err: VoiceError) => void,
  onStateChange: (state: VoiceState) => void
): (() => void) => {
  if (!isSpeechRecognitionSupported()) {
    onError({
      code: 'not_supported',
      message: 'Speech recognition is not supported in this browser environment. Using text input.',
    });
    return () => {};
  }

  // Stop any active session
  if (activeRecognition) {
    try {
      activeRecognition.abort();
    } catch {
      // ignore
    }
  }

  const win = window as unknown as IWindow;
  const SpeechRecognitionClass = win.SpeechRecognition || win.webkitSpeechRecognition;
  const recognition = new SpeechRecognitionClass();

  recognition.continuous = false;
  recognition.interimResults = false;
  recognition.lang = 'en-US';
  recognition.maxAlternatives = 1;

  activeRecognition = recognition;
  onStateChange('listening');

  recognition.onstart = () => {
    onStateChange('listening');
  };

  recognition.onresult = (event: any) => {
    onStateChange('processing');
    if (event.results && event.results[0] && event.results[0][0]) {
      const transcript = event.results[0][0].transcript;
      onResult(transcript);
    }
    onStateChange('idle');
  };

  recognition.onerror = (event: any) => {
    let friendlyMessage = 'Voice recognition error';
    switch (event.error) {
      case 'not-allowed':
        friendlyMessage = 'Microphone permission was denied. Please grant microphone access in browser settings.';
        break;
      case 'no-speech':
        friendlyMessage = 'No speech was detected. Please try speaking closer to the microphone.';
        break;
      case 'audio-capture':
        friendlyMessage = 'No microphone was found or microphone is busy.';
        break;
      case 'network':
        friendlyMessage = 'Voice recognition network error.';
        break;
      default:
        friendlyMessage = `Recognition error: ${event.error || 'Unknown'}`;
    }

    onStateChange('error');
    onError({
      code: event.error || 'unknown',
      message: friendlyMessage,
    });
    setTimeout(() => onStateChange('idle'), 2000);
  };

  recognition.onend = () => {
    activeRecognition = null;
    onStateChange('idle');
  };

  try {
    recognition.start();
  } catch (err: unknown) {
    onStateChange('error');
    onError({
      code: 'start_failed',
      message: err instanceof Error ? err.message : 'Failed to initialize microphone',
    });
  }

  return () => {
    if (activeRecognition) {
      try {
        activeRecognition.abort();
      } catch {
        // ignore
      }
      activeRecognition = null;
      onStateChange('idle');
    }
  };
};

/**
 * Cancels active listening.
 */
export const cancelVoiceRecognition = () => {
  if (activeRecognition) {
    try {
      activeRecognition.abort();
    } catch {
      // ignore
    }
    activeRecognition = null;
  }
};

/**
 * Speaks text using local native speech synthesis.
 */
export const speakGroundedAnswer = (
  text: string,
  rate = 1.0,
  pitch = 1.0,
  onEnd?: () => void
) => {
  if (!isSpeechSynthesisSupported()) return;

  // Cancel any ongoing speech
  window.speechSynthesis.cancel();

  // Strip markdown formatting for natural vocalization
  const cleanText = text
    .replace(/[#*_`>\[\]]/g, '')
    .replace(/http\S+/g, '')
    .slice(0, 350); // limit spoken response to concise overview

  const utterance = new SpeechSynthesisUtterance(cleanText);
  utterance.rate = Math.max(0.7, Math.min(rate, 1.5));
  utterance.pitch = Math.max(0.8, Math.min(pitch, 1.2));
  utterance.lang = 'en-US';

  if (onEnd) {
    utterance.onend = onEnd;
    utterance.onerror = onEnd;
  }

  window.speechSynthesis.speak(utterance);
};

/**
 * Stops any speech synthesis output immediately.
 */
export const stopSpeech = () => {
  if (isSpeechSynthesisSupported()) {
    window.speechSynthesis.cancel();
  }
};

