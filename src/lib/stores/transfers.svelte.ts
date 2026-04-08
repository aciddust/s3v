import { type UnlistenFn } from '@tauri-apps/api/event';
import {
  listTransfers,
  onTransferProgress,
  onTransferStatusChanged,
  onTransferCompleted,
  type TransferJobSummary,
  type TransferProgressEvent,
  type TransferStatusEvent,
  type TransferCompletedEvent,
} from '$lib/api/transfers';

class TransferStore {
  jobs = $state<TransferJobSummary[]>([]);

  activeCount = $derived(this.jobs.filter((j) => j.status === 'active').length);

  queuedCount = $derived(this.jobs.filter((j) => j.status === 'queued').length);

  private unlistenFns: UnlistenFn[] = [];
  private dismissed = new Set<string>();

  async init(): Promise<void> {
    // Load initial state
    try {
      this.jobs = await listTransfers();
    } catch {
      this.jobs = [];
    }

    const [unProgress, unStatus, unCompleted] = await Promise.all([
      onTransferProgress((e: TransferProgressEvent) => {
        this.jobs = this.jobs.map((job) =>
          job.id === e.id
            ? {
                ...job,
                progress: {
                  bytes_transferred: e.bytes_transferred,
                  total_bytes: e.total_bytes,
                },
              }
            : job,
        );
      }),

      onTransferStatusChanged(async (e: TransferStatusEvent) => {
        if (this.dismissed.has(e.id)) return;
        const exists = this.jobs.some((j) => j.id === e.id);
        if (exists) {
          this.jobs = this.jobs.map((job) =>
            job.id === e.id ? { ...job, status: e.status, error: e.error } : job,
          );
        } else {
          // New job appeared — reload full list, excluding dismissed
          try {
            const all = await listTransfers();
            this.jobs = all.filter((j) => !this.dismissed.has(j.id));
          } catch {
            /* ignore */
          }
        }
      }),

      onTransferCompleted((e: TransferCompletedEvent) => {
        this.jobs = this.jobs.map((job) =>
          job.id === e.id ? { ...job, status: 'completed' } : job,
        );
      }),
    ]);

    this.unlistenFns = [unProgress, unStatus, unCompleted];
  }

  async reload(): Promise<void> {
    try {
      const all = await listTransfers();
      this.jobs = all.filter((j) => !this.dismissed.has(j.id));
    } catch {
      /* ignore */
    }
  }

  removeJob(id: string): void {
    this.dismissed.add(id);
    this.jobs = this.jobs.filter((j) => j.id !== id);
  }

  clearCompleted(): void {
    for (const job of this.jobs) {
      if (job.status === 'completed' || job.status === 'failed' || job.status === 'cancelled') {
        this.dismissed.add(job.id);
      }
    }
    this.jobs = this.jobs.filter(
      (j) => j.status !== 'completed' && j.status !== 'failed' && j.status !== 'cancelled',
    );
  }

  destroy(): void {
    for (const fn of this.unlistenFns) {
      fn();
    }
    this.unlistenFns = [];
  }
}

export const transferStore = new TransferStore();
