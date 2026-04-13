import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type MovePhase = 'start' | 'copying' | 'deleting' | 'moved' | 'completed' | 'failed';
export type FolderOpPhase = 'started' | 'copying' | 'deleting' | 'completed' | 'failed' | 'cancelled';

export interface MoveJob {
  id: string;
  op: 'move' | 'copy';
  phase: MovePhase | FolderOpPhase;
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

interface FolderOpProgressEvent {
  id: string;
  op: string;
  phase: FolderOpPhase;
  done: number;
  total: number;
  key: string;
  source_prefix: string;
  dest_prefix: string;
  error: string | null;
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
    this.jobs.filter(
      (j) =>
        j.phase !== 'completed' &&
        j.phase !== 'failed' &&
        j.phase !== 'cancelled',
    ).length,
  );

  private unlisten: UnlistenFn | undefined;
  private unlistenFolderOp: UnlistenFn | undefined;
  private removeTimers = new Map<string, ReturnType<typeof setTimeout>>();

  async init(): Promise<void> {
    this.unlisten = await listen<MoveProgressEvent>('move-progress', (e) => {
      const p = e.payload;
      const existing = this.jobs.find((j) => j.id === p.id);
      const startedAt = existing?.startedAt ?? Date.now();

      const job: MoveJob = {
        id: p.id,
        op: 'move',
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

    this.unlistenFolderOp = await listen<FolderOpProgressEvent>('folder-op-progress', (e) => {
      const p = e.payload;
      const existing = this.jobs.find((j) => j.id === p.id);

      if (p.phase === 'started') {
        const job: MoveJob = {
          id: p.id,
          op: p.op === 'copy' ? 'copy' : 'move',
          phase: p.phase,
          done: p.done,
          total: p.total,
          key: p.key,
          destPrefix: p.dest_prefix,
          startedAt: Date.now(),
          eta: null,
        };
        this.jobs = [...this.jobs, job];
      } else if (p.phase === 'copying' || p.phase === 'deleting') {
        if (existing) {
          const updated: MoveJob = {
            ...existing,
            phase: p.phase,
            done: p.done,
            total: p.total,
            key: p.key,
            eta: calcEta(existing.startedAt, p.done, p.total),
          };
          this.jobs = this.jobs.map((j) => (j.id === updated.id ? updated : j));
        }
      } else if (p.phase === 'completed' || p.phase === 'failed' || p.phase === 'cancelled') {
        if (existing) {
          const updated: MoveJob = {
            ...existing,
            phase: p.phase,
            done: p.done,
            total: p.total,
          };
          this.jobs = this.jobs.map((j) => (j.id === updated.id ? updated : j));
        }

        const prev = this.removeTimers.get(p.id);
        if (prev) clearTimeout(prev);
        this.removeTimers.set(
          p.id,
          setTimeout(() => {
            this.jobs = this.jobs.filter((j) => j.id !== p.id);
            this.removeTimers.delete(p.id);
          }, 3000),
        );
      }
    });
  }

  /** Add a job directly from the frontend (for operations without backend events) */
  addJob(op: 'move' | 'copy', key: string, destPrefix: string): string {
    const id = crypto.randomUUID();
    this.jobs = [
      ...this.jobs,
      {
        id,
        op,
        phase: 'started',
        done: 0,
        total: 1,
        key,
        destPrefix,
        startedAt: Date.now(),
        eta: null,
      },
    ];
    return id;
  }

  /** Mark a frontend-added job as completed */
  completeJob(id: string, phase: MovePhase | FolderOpPhase = 'completed') {
    const idx = this.jobs.findIndex((j) => j.id === id);
    if (idx >= 0) {
      this.jobs[idx] = { ...this.jobs[idx], phase, done: 1, total: 1 };
      this.jobs = [...this.jobs];

      const prev = this.removeTimers.get(id);
      if (prev) clearTimeout(prev);
      this.removeTimers.set(
        id,
        setTimeout(() => {
          this.jobs = this.jobs.filter((j) => j.id !== id);
          this.removeTimers.delete(id);
        }, 3000),
      );
    }
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
      if (job.phase === 'completed' || job.phase === 'failed' || job.phase === 'cancelled') {
        const timer = this.removeTimers.get(job.id);
        if (timer) {
          clearTimeout(timer);
          this.removeTimers.delete(job.id);
        }
      }
    }
    this.jobs = this.jobs.filter(
      (j) => j.phase !== 'completed' && j.phase !== 'failed' && j.phase !== 'cancelled',
    );
  }

  destroy(): void {
    this.unlisten?.();
    this.unlistenFolderOp?.();
    for (const timer of this.removeTimers.values()) clearTimeout(timer);
    this.removeTimers.clear();
  }
}

export const moveStore = new MoveStore();
