/**
 * Mouse-based drag state for in-app file dragging.
 * HTML5 DnD doesn't work in Tauri/WKWebView because the native drag handler
 * intercepts all drag events before they reach the WebView.
 */

import { settingsStore } from './settings.svelte';

export interface DragPayload {
  profileId: string;
  bucket: string;
  keys: string[];
  sourcePrefix: string;
}

const EDGE_THRESHOLD = 20;

class DragStore {
  payload = $state<DragPayload | null>(null);
  active = $state(false);
  x = $state(0);
  y = $state(0);
  /** The data-drop-key of the element currently hovered, or '__panel__' for panel bg. */
  hoverDropKey = $state<string | null>(null);
  modifierKey = $state<'meta' | 'shift' | null>(null);
  nearEdge = $state(false);
  nativeDragStarted = $state(false);
  private isMac = navigator.platform.toUpperCase().includes('MAC');

  start(data: DragPayload, e: MouseEvent): void {
    console.log('[drag] start', { keys: data.keys, sourcePrefix: data.sourcePrefix });
    this.payload = data;
    this.active = true;
    this.x = e.clientX;
    this.y = e.clientY;
    this.hoverDropKey = null;

    const updateModifier = (e: { metaKey: boolean; ctrlKey: boolean; shiftKey: boolean }) => {
      if (this.isMac ? e.metaKey : e.ctrlKey) {
        this.modifierKey = 'meta';
      } else if (e.shiftKey) {
        this.modifierKey = 'shift';
      } else {
        this.modifierKey = null;
      }
    };
    const onMove = (me: MouseEvent) => {
      this.x = me.clientX;
      this.y = me.clientY;
      // Update hover target
      const el = document.elementFromPoint(me.clientX, me.clientY) as HTMLElement | null;
      const dropRow = el?.closest('[data-drop-key]') as HTMLElement | null;
      this.hoverDropKey = dropRow ? dropRow.dataset.dropKey! : '__panel__';
      updateModifier(me);

      // Edge detection for native drag
      if (!this.nativeDragStarted) {
        const nearEdge =
          me.clientX < EDGE_THRESHOLD ||
          me.clientY < EDGE_THRESHOLD ||
          me.clientX > window.innerWidth - EDGE_THRESHOLD ||
          me.clientY > window.innerHeight - EDGE_THRESHOLD;
        this.nearEdge = nearEdge;

        if (nearEdge && this.payload) {
          const { profileId, bucket, keys } = this.payload;
          const plainKeys = [...keys];

          // Block folder native drag if disabled in settings
          const hasFolders = plainKeys.some(k => k.endsWith('/'));
          if (hasFolders && !settingsStore.folderDragDownload) {
            return;
          }

          this.nativeDragStarted = true;
          // Listen for debug event from backend
          import('@tauri-apps/api/event').then(({ listen }) => {
            listen('native-drag-debug', (e) => {
              console.log('[drag] backend debug:', e.payload);
            });
          });
          console.log('[drag] near edge, invoking start_native_drag', { profileId, bucket, keys: plainKeys });
          import('@tauri-apps/api/core').then(({ invoke }) => {
            invoke('start_native_drag', { profileId, bucket, keys: plainKeys }).then(() => {
              console.log('[drag] start_native_drag succeeded');
              // Success: clean up custom drag (native drag takes over mouse events)
              window.removeEventListener('mousemove', onMove);
              window.removeEventListener('mouseup', onUp as EventListener);
              window.removeEventListener('keydown', onKey);
              window.removeEventListener('keyup', onKey);
              setTimeout(() => {
                this.active = false;
                this.payload = null;
                this.hoverDropKey = null;
                this.modifierKey = null;
                this.nearEdge = false;
                this.nativeDragStarted = false;
              }, 100);
            }).catch((err: unknown) => {
              console.error('Native drag failed:', err);
              console.error('Native drag error type:', typeof err, JSON.stringify(err));
              this.nativeDragStarted = false;
              this.nearEdge = false;
            });
          });
          return;
        }
      }
    };
    const onKey = (ke: KeyboardEvent) => updateModifier(ke);
    const onUp = (me: MouseEvent) => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp as EventListener);
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('keyup', onKey);
      updateModifier(me);
      const modifier = this.modifierKey;
      const dropTarget = document.elementFromPoint(this.x, this.y);
      if (dropTarget) {
        dropTarget.dispatchEvent(new CustomEvent('internaldrop', { bubbles: true, detail: { modifier } }));
      }
      setTimeout(() => {
        this.active = false;
        this.payload = null;
        this.hoverDropKey = null;
        this.modifierKey = null;
        this.nearEdge = false;
        this.nativeDragStarted = false;
      }, 0);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp as EventListener);
    window.addEventListener('keydown', onKey);
    window.addEventListener('keyup', onKey);
  }

  consume(): DragPayload | null {
    const data = this.payload;
    this.payload = null;
    return data;
  }
}

export const dragStore = new DragStore();
