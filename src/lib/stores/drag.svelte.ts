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

export interface TabDragPayload {
  profileId: string;
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
  /** The profileId of the tab currently hovered during drag, or null */
  hoverTabProfileId = $state<string | null>(null);
  /** Whether the hovered tab accepts drops (same profile as drag source) */
  hoverTabDroppable = $state(false);
  /** Tab drag state (separate from file drag) */
  tabDragPayload = $state<TabDragPayload | null>(null);
  tabDragActive = $state(false);
  tabDragX = $state(0);
  tabDragY = $state(0);
  /** Which panel side is being hovered during tab drag */
  hoverPanelSide = $state<'left' | 'right' | null>(null);
  private isMac = navigator.platform.toUpperCase().includes('MAC');

  start(data: DragPayload, e: MouseEvent): void {
    console.log('[drag] start', { keys: data.keys, sourcePrefix: data.sourcePrefix });
    this.payload = data;
    this.active = true;
    this.x = e.clientX;
    this.y = e.clientY;
    this.hoverDropKey = null;

    const updateModifier = (ev: { metaKey: boolean; ctrlKey: boolean; shiftKey: boolean }) => {
      if (this.isMac ? ev.metaKey : ev.ctrlKey) {
        this.modifierKey = 'meta';
      } else if (ev.shiftKey) {
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

      // Check if hovering over a tab
      const tabEl = el?.closest('[data-tab-profile-id]') as HTMLElement | null;
      if (tabEl) {
        const tabProfileId = tabEl.dataset.tabProfileId!;
        this.hoverTabProfileId = tabProfileId;
        const sourceProfileId = this.payload?.profileId?.replace(/::right$/, '') ?? null;
        // Droppable on any tab except the one the drag originated from
        this.hoverTabDroppable = sourceProfileId !== tabProfileId;
        this.hoverDropKey = null; // Don't highlight file rows while over tabs
      } else {
        this.hoverTabProfileId = null;
        this.hoverTabDroppable = false;
        const dropRow = el?.closest('[data-drop-key]') as HTMLElement | null;
        this.hoverDropKey = dropRow ? dropRow.dataset.dropKey! : '__panel__';
      }
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
          const hasFolders = plainKeys.some((k) => k.endsWith('/'));
          if (hasFolders && !settingsStore.folderDragDownload) {
            return;
          }

          this.nativeDragStarted = true;
          // Listen for debug event from backend
          void import('@tauri-apps/api/event').then(({ listen }) => {
            void listen('native-drag-debug', (ev) => {
              console.log('[drag] backend debug:', ev.payload);
            });
          });
          console.log('[drag] near edge, invoking start_native_drag', {
            profileId,
            bucket,
            keys: plainKeys,
          });
          void import('@tauri-apps/api/core').then(({ invoke }) => {
            invoke('start_native_drag', { profileId, bucket, keys: plainKeys })
              .then(() => {
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
                  this.hoverTabProfileId = null;
                  this.hoverTabDroppable = false;
                }, 100);
              })
              .catch((err: unknown) => {
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
        dropTarget.dispatchEvent(
          new CustomEvent('internaldrop', { bubbles: true, detail: { modifier } }),
        );
      }
      setTimeout(() => {
        this.active = false;
        this.payload = null;
        this.hoverDropKey = null;
        this.modifierKey = null;
        this.nearEdge = false;
        this.nativeDragStarted = false;
        this.hoverTabProfileId = null;
        this.hoverTabDroppable = false;
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
    this.hoverTabProfileId = null;
    this.hoverTabDroppable = false;
    return data;
  }

  startTabDrag(data: TabDragPayload, e: MouseEvent): void {
    this.tabDragPayload = data;
    this.tabDragActive = true;
    this.tabDragX = e.clientX;
    this.tabDragY = e.clientY;
    this.hoverPanelSide = null;

    const onMove = (me: MouseEvent) => {
      this.tabDragX = me.clientX;
      this.tabDragY = me.clientY;

      // Detect panel side from DOM
      const el = document.elementFromPoint(me.clientX, me.clientY) as HTMLElement | null;
      const panelEl = el?.closest('[data-panel-side]') as HTMLElement | null;
      if (panelEl) {
        this.hoverPanelSide = panelEl.dataset.panelSide as 'left' | 'right';
      } else {
        // In single panel mode, use horizontal center of the main content area
        const mainArea = document.querySelector('[data-main-area]') as HTMLElement | null;
        if (mainArea) {
          const rect = mainArea.getBoundingClientRect();
          const isInside =
            me.clientX >= rect.left &&
            me.clientX <= rect.right &&
            me.clientY >= rect.top &&
            me.clientY <= rect.bottom;
          if (isInside) {
            this.hoverPanelSide = me.clientX < rect.left + rect.width / 2 ? 'left' : 'right';
          } else {
            this.hoverPanelSide = null;
          }
        } else {
          this.hoverPanelSide = null;
        }
      }
    };

    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);

      if (this.hoverPanelSide && this.tabDragPayload) {
        const dropTarget = document.elementFromPoint(this.tabDragX, this.tabDragY);
        if (dropTarget) {
          dropTarget.dispatchEvent(
            new CustomEvent('internaltabdrop', {
              bubbles: true,
              detail: {
                profileId: this.tabDragPayload.profileId,
                side: this.hoverPanelSide,
              },
            }),
          );
        }
      }

      this.tabDragPayload = null;
      this.tabDragActive = false;
      this.tabDragX = 0;
      this.tabDragY = 0;
      this.hoverPanelSide = null;
    };

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }
}

export const dragStore = new DragStore();
