export interface VirtualFile {
  id: string;
  name: string;
  type: 'file' | 'folder';
  size: number;
  path: string;
  contentType: string | null;
  lastModified: string | null;
}

export interface DesktopItem {
  file: VirtualFile;
  x: number;
  y: number;
}

export interface PanelState {
  bucket: string;
  currentPath: string;
}

export type SortField = 'name' | 'size' | 'lastModified';
export type SortOrder = 'asc' | 'desc';

export interface TransferItem {
  id: string;
  file: VirtualFile;
  direction: 'upload' | 'download' | 'copy';
  progress: number;
  status: 'queued' | 'active' | 'completed' | 'failed';
}

export interface DragState {
  active: boolean;
  file: VirtualFile | null;
  source: 'desktop' | 'left-panel' | 'right-panel';
  x: number;
  y: number;
  modifier: boolean;
}

export interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  target: VirtualFile | null;
  source: 'desktop' | 'left-panel' | 'right-panel';
}
