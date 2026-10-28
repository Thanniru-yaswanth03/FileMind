import { RiskTier } from '../types';

export function formatBytes(bytes: number, decimals: number = 1): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = parseFloat((bytes / Math.pow(k, i)).toFixed(dm));
  return `${value} ${sizes[i]}`;
}

export function formatNumber(num: number): string {
  return new Intl.NumberFormat('en-US').format(num);
}

export function truncatePath(path: string, maxLength: number = 42): string {
  if (path.length <= maxLength) return path;
  const parts = path.split('\\');
  if (parts.length <= 2) return `${path.slice(0, maxLength - 3)}...`;

  const drive = parts[0];
  const file = parts[parts.length - 1];
  const middle = '...';

  if (`${drive}\\${middle}\\${file}`.length >= maxLength) {
    return `${drive}\\...\\${file.slice(0, 16)}...`;
  }

  return `${drive}\\${middle}\\${file}`;
}

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
    // Fallback for non-secure contexts
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.opacity = '0';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    const successful = document.execCommand('copy');
    document.body.removeChild(textArea);
    return successful;
  } catch (err) {
    console.error('Failed to copy to clipboard:', err);
    return false;
  }
}

export function getRiskTierLabel(tier: RiskTier): string {
  switch (tier) {
    case 'safe':
      return 'Safe to Clear';
    case 'caution':
      return 'Caution';
    case 'high':
      return 'High Risk';
    case 'danger':
      return 'Blocked (System)';
  }
}

export function getRiskTierBadgeClass(tier: RiskTier): string {
  switch (tier) {
    case 'safe':
      return 'badge-safe';
    case 'caution':
      return 'badge-caution';
    case 'high':
      return 'badge-high';
    case 'danger':
      return 'badge-danger';
  }
}

