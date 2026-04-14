import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface LogEntry {
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  target: string;
  message: string;
}

const MAX_ENTRIES = 500;

class LogStore {
  entries = $state<LogEntry[]>([]);
  open = $state(false);
  private unlisten: UnlistenFn | null = null;

  async init() {
    this.unlisten = await listen<LogEntry>('log:entry', (e) => {
      this.entries = [...this.entries.slice(-(MAX_ENTRIES - 1)), e.payload];
    });
  }

  toggle() {
    this.open = !this.open;
  }

  formatEntry(entry: LogEntry): string {
    return `${entry.timestamp} ${entry.level.toUpperCase().padEnd(5)} [${entry.target}] ${entry.message}`;
  }

  copyLine(entry: LogEntry) {
    void navigator.clipboard.writeText(this.formatEntry(entry));
  }

  copyAll() {
    const text = this.entries.map((e) => this.formatEntry(e)).join('\n');
    void navigator.clipboard.writeText(text);
  }

  downloadLog(bucketName?: string) {
    const now = new Date();
    const ts = now
      .toISOString()
      .replace(/[-:T]/g, (m) => (m === 'T' ? '_' : ''))
      .replace(/\..+/, '');
    const prefix = bucketName || 'S3V';
    const filename = `${prefix}_${ts}.log`;
    const text = this.entries.map((e) => this.formatEntry(e)).join('\n');
    const blob = new Blob([text], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  clear() {
    this.entries = [];
  }

  destroy() {
    this.unlisten?.();
    this.unlisten = null;
  }
}

export const logStore = new LogStore();
