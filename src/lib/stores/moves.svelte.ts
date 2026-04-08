import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type MovePhase = 'start' | 'copying' | 'deleting' | 'moved' | 'completed' | 'failed';

export interface MoveJob {
  id: string;
  phase: MovePhase;
  done: number;
  total: number;
  key: string;
  destPrefix: string;
  startedAt: number;
  /** Estimated seconds remaining, null until first file completes */
  eta: number | null;
}

interface MoveProgressEvent {
  id: string;
  phase: MovePhase;
  done: number;
  total: number;
  key: string;
  dest_prefix: string;
}

function calcEta(startedAt: number, done: number, total: number): number | null {
  if (done <= 0) return null;
  const elapsed = (Date.now() - startedAt) / 1000;
  const rate = done / elapsed;
  return Math.round((total - done) / rate);
}

class MoveStore {
  jobs = $state<MoveJob[]>([]);

  activeCount = $derived(
    this.jobs.filter((j) => j.phase !== 'completed' && j.phase !== 'failed').length,
  );

  private unlisten: UnlistenFn | undefined;
  private removeTimers = new Map<string, ReturnType<typeof setTimeout>>();

  async init(): Promise<void> {
    this.unlisten = await listen<MoveProgressEvent>('move-progress', (e) => {
      const p = e.payload;
      const existing = this.jobs.find((j) => j.id === p.id);
      const startedAt = existing?.startedAt ?? Date.now();

      const job: MoveJob = {
        id: p.id,
        phase: p.phase,
        done: p.done,
        total: p.total,
        key: p.key,
        destPrefix: p.dest_prefix,
        startedAt,
        eta: calcEta(startedAt, p.done, p.total),
      };

      if (existing) {
        this.jobs = this.jobs.map((j) => (j.id === job.id ? job : j));
      } else {
        this.jobs = [...this.jobs, job];
      }

      if (job.phase === 'completed' || job.phase === 'failed') {
        const prev = this.removeTimers.get(job.id);
        if (prev) clearTimeout(prev);
        this.removeTimers.set(
          job.id,
          setTimeout(() => {
            this.jobs = this.jobs.filter((j) => j.id !== job.id);
            this.removeTimers.delete(job.id);
          }, 3000),
        );
      }
    });
  }

  removeJob(id: string): void {
    const timer = this.removeTimers.get(id);
    if (timer) {
      clearTimeout(timer);
      this.removeTimers.delete(id);
    }
    this.jobs = this.jobs.filter((j) => j.id !== id);
  }

  clearCompleted(): void {
    for (const job of this.jobs) {
      if (job.phase === 'completed' || job.phase === 'failed') {
        const timer = this.removeTimers.get(job.id);
        if (timer) {
          clearTimeout(timer);
          this.removeTimers.delete(job.id);
        }
      }
    }
    this.jobs = this.jobs.filter((j) => j.phase !== 'completed' && j.phase !== 'failed');
  }

  destroy(): void {
    this.unlisten?.();
    for (const timer of this.removeTimers.values()) clearTimeout(timer);
    this.removeTimers.clear();
  }
}

export const moveStore = new MoveStore();
