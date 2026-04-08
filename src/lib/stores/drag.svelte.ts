/**
 * Mouse-based drag state for in-app file dragging.
 * HTML5 DnD doesn't work in Tauri/WKWebView because the native drag handler
 * intercepts all drag events before they reach the WebView.
 */

export interface DragPayload {
  profileId: string;
  bucket: string;
  keys: string[];
  sourcePrefix: string;
}

class DragStore {
  payload = $state<DragPayload | null>(null);
  active = $state(false);
  x = $state(0);
  y = $state(0);
  /** The data-drop-key of the element currently hovered, or '__panel__' for panel bg. */
  hoverDropKey = $state<string | null>(null);

  start(data: DragPayload, e: MouseEvent): void {
    this.payload = data;
    this.active = true;
    this.x = e.clientX;
    this.y = e.clientY;
    this.hoverDropKey = null;

    const onMove = (me: MouseEvent) => {
      this.x = me.clientX;
      this.y = me.clientY;
      // Update hover target
      const el = document.elementFromPoint(me.clientX, me.clientY) as HTMLElement | null;
      const dropRow = el?.closest('[data-drop-key]') as HTMLElement | null;
      this.hoverDropKey = dropRow ? dropRow.dataset.dropKey! : '__panel__';
    };
    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
      const dropTarget = document.elementFromPoint(this.x, this.y);
      if (dropTarget) {
        dropTarget.dispatchEvent(new CustomEvent('internaldrop', { bubbles: true }));
      }
      setTimeout(() => {
        this.active = false;
        this.payload = null;
        this.hoverDropKey = null;
      }, 0);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  consume(): DragPayload | null {
    const data = this.payload;
    this.payload = null;
    return data;
  }
}

export const dragStore = new DragStore();
