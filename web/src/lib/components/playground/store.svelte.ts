import type { VirtualFile, DesktopItem, TransferItem, DragState, ContextMenuState, SortField, SortOrder } from './types';
import { BUCKET_NAME, BUCKET_FILES, INITIAL_DESKTOP_FILES } from './data';

let _transferId = 0;

function filesAtPath(allFiles: VirtualFile[], path: string): VirtualFile[] {
  return allFiles.filter((f) => f.path === path);
}

function sortFiles(files: VirtualFile[], field: SortField, order: SortOrder): VirtualFile[] {
  const folders = files.filter((f) => f.type === 'folder');
  const items = files.filter((f) => f.type === 'file');
  const sorted = [...items].sort((a, b) => {
    let cmp = 0;
    if (field === 'name') cmp = a.name.localeCompare(b.name);
    else if (field === 'size') cmp = a.size - b.size;
    else if (field === 'lastModified') cmp = (a.lastModified ?? '').localeCompare(b.lastModified ?? '');
    return order === 'asc' ? cmp : -cmp;
  });
  folders.sort((a, b) => a.name.localeCompare(b.name));
  return [...folders, ...sorted];
}

class PlaygroundStore {
  bucketFiles = $state<VirtualFile[]>([...BUCKET_FILES]);
  desktopFiles = $state<DesktopItem[]>(INITIAL_DESKTOP_FILES.map((d) => ({ ...d, file: { ...d.file } })));

  leftPanel = $state({ bucket: BUCKET_NAME, currentPath: '' });
  rightPanel = $state({ bucket: BUCKET_NAME, currentPath: 'project-assets/images/' });

  sortField = $state<SortField>('name');
  sortOrder = $state<SortOrder>('asc');
  transferPanelVisible = $state(true);
  transferPanelExpanded = $state(false);
  dualPanel = $state(false);
  selectedFiles = $state<Set<string>>(new Set());
  selectedDesktopFiles = $state<Set<string>>(new Set());
  searchQuery = $state('');
  uploadDialogOpen = $state(false);
  uploadDialogTarget = $state<'left' | 'right'>('left');
  profileManagerOpen = $state(false);

  // Profile tabs
  profiles = $state<{ id: string; name: string; provider: string; connected: boolean }[]>([
    { id: 'demo', name: 'demo-bucket', provider: 'aws', connected: true },
  ]);
  activeProfileId = $state('demo');

  // Mock bucket data per profile: profileId → { bucketName → files[] }
  private profileData: Record<string, Record<string, VirtualFile[]>> = {
    demo: {
      [BUCKET_NAME]: [...BUCKET_FILES],
      'static-assets': [
        { id: 'demo-sa1', name: 'css', type: 'folder', size: 0, path: '', contentType: null, lastModified: '2026-04-08T09:00:00Z' },
        { id: 'demo-sa2', name: 'js', type: 'folder', size: 0, path: '', contentType: null, lastModified: '2026-04-08T09:00:00Z' },
        { id: 'demo-sa3', name: 'index.html', type: 'file', size: 4_200, path: '', contentType: 'text/html', lastModified: '2026-04-09T14:00:00Z' },
        { id: 'demo-sa4', name: 'style.css', type: 'file', size: 18_000, path: 'css/', contentType: 'text/css', lastModified: '2026-04-09T14:00:00Z' },
        { id: 'demo-sa5', name: 'app.js', type: 'file', size: 52_000, path: 'js/', contentType: 'application/javascript', lastModified: '2026-04-09T14:00:00Z' },
      ],
      'logs': [
        { id: 'demo-lg1', name: 'access-2026-04.log', type: 'file', size: 8_900_000, path: '', contentType: 'text/plain', lastModified: '2026-04-10T06:00:00Z' },
        { id: 'demo-lg2', name: 'error-2026-04.log', type: 'file', size: 340_000, path: '', contentType: 'text/plain', lastModified: '2026-04-10T06:00:00Z' },
      ],
    },
  };
  // All bucket names for current profile
  currentBuckets = $state<string[]>(Object.keys({
    [BUCKET_NAME]: true, 'static-assets': true, 'logs': true,
  }));

  connectProfile(profile: { id: string; name: string; provider: string }) {
    if (!this.profiles.find((p) => p.id === profile.id)) {
      this.profiles.push({ ...profile, connected: true });
    }
    this.activeProfileId = profile.id;

    // Generate mock multi-bucket data for new profiles
    if (!this.profileData[profile.id]) {
      const ts = '2026-04-10T09:00:00Z';
      const p = profile.id;

      if (profile.provider === 'minio') {
        this.profileData[p] = {
          'dev-data': [
            { id: `${p}-1`, name: 'uploads', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-2`, name: 'cache', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-3`, name: 'seed-data.sql', type: 'file', size: 8_500_000, path: '', contentType: 'application/sql', lastModified: ts },
            { id: `${p}-4`, name: 'docker-compose.yaml', type: 'file', size: 1_200, path: '', contentType: 'text/yaml', lastModified: ts },
            { id: `${p}-5`, name: 'test-image.png', type: 'file', size: 340_000, path: 'uploads/', contentType: 'image/png', lastModified: ts },
            { id: `${p}-6`, name: 'sample.csv', type: 'file', size: 52_000, path: 'uploads/', contentType: 'text/csv', lastModified: ts },
          ],
          'logs': [
            { id: `${p}-l1`, name: 'app.log', type: 'file', size: 2_400_000, path: '', contentType: 'text/plain', lastModified: ts },
            { id: `${p}-l2`, name: 'error.log', type: 'file', size: 180_000, path: '', contentType: 'text/plain', lastModified: ts },
          ],
          'backups': [
            { id: `${p}-b1`, name: 'daily', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-b2`, name: 'db-snapshot.sql', type: 'file', size: 25_000_000, path: '', contentType: 'application/sql', lastModified: ts },
          ],
          'staging': [
            { id: `${p}-s1`, name: 'build-output', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-s2`, name: 'manifest.json', type: 'file', size: 3_200, path: '', contentType: 'application/json', lastModified: ts },
          ],
          'temp': [],
        };
      } else if (profile.provider === 'r2') {
        this.profileData[p] = {
          'assets': [
            { id: `${p}-1`, name: 'images', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-2`, name: 'fonts', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-3`, name: 'robots.txt', type: 'file', size: 120, path: '', contentType: 'text/plain', lastModified: ts },
            { id: `${p}-4`, name: 'og-banner.png', type: 'file', size: 580_000, path: 'images/', contentType: 'image/png', lastModified: ts },
            { id: `${p}-5`, name: 'inter-var.woff2', type: 'file', size: 98_000, path: 'fonts/', contentType: 'font/woff2', lastModified: ts },
          ],
        };
      } else if (profile.provider === 'aws') {
        this.profileData[p] = {
          'production-assets': [
            { id: `${p}-1`, name: 'media', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-2`, name: 'index.html', type: 'file', size: 4_800, path: '', contentType: 'text/html', lastModified: ts },
          ],
          'data-lake': [
            { id: `${p}-d1`, name: 'raw', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-d2`, name: 'processed', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
          ],
          'logs-archive': [
            { id: `${p}-la1`, name: '2026-04.tar.gz', type: 'file', size: 45_000_000, path: '', contentType: 'application/gzip', lastModified: ts },
          ],
        };
      } else {
        this.profileData[p] = {
          'storage': [
            { id: `${p}-1`, name: 'data', type: 'folder', size: 0, path: '', contentType: null, lastModified: ts },
            { id: `${p}-2`, name: 'config.json', type: 'file', size: 2_400, path: '', contentType: 'application/json', lastModified: ts },
          ],
          'backup': [
            { id: `${p}-b1`, name: 'snapshot.tar.gz', type: 'file', size: 12_000_000, path: '', contentType: 'application/gzip', lastModified: ts },
          ],
        };
      }
    }

    // Update panels with first bucket of this profile
    const buckets = Object.keys(this.profileData[profile.id]);
    this.currentBuckets = buckets;
    const firstBucket = buckets[0] ?? '';
    this.bucketFiles = [...(this.profileData[profile.id][firstBucket] ?? [])];
    this.leftPanel = { bucket: firstBucket, currentPath: '' };
    this.rightPanel = { bucket: firstBucket, currentPath: '' };
    this.clearSelection();
  }

  selectBucket(bucket: string, side: 'left' | 'right' = 'left') {
    this.saveCurrentBucket();
    const panel = side === 'left' ? this.leftPanel : this.rightPanel;
    panel.bucket = bucket;
    panel.currentPath = '';
    const profileData = this.profileData[this.activeProfileId];
    if (profileData && profileData[bucket]) {
      this.bucketFiles = [...profileData[bucket]];
    }
    this.clearSelection();
  }

  getFolderPathsForBucket(bucket: string): Set<string> {
    const pd = this.profileData[this.activeProfileId];
    if (!pd || !pd[bucket]) return new Set();
    const paths = new Set<string>();
    for (const f of pd[bucket]) {
      if (f.path) {
        const parts = f.path.split('/').filter(Boolean);
        let acc = '';
        for (const part of parts) {
          acc += part + '/';
          paths.add(acc);
        }
      }
      if (f.type === 'folder') {
        paths.add(f.path + f.name + '/');
      }
    }
    return paths;
  }

  disconnectProfile(id: string) {
    this.profiles = this.profiles.filter((p) => p.id !== id);
    if (this.activeProfileId === id && this.profiles.length > 0) {
      // Switch to another profile
      this.connectProfile(this.profiles[0]);
    } else if (this.profiles.length === 0) {
      this.activeProfileId = '';
      this.bucketFiles = [];
      this.leftPanel = { bucket: '', currentPath: '' };
      this.rightPanel = { bucket: '', currentPath: '' };
    }
  }

  // Save current bucket files before switching
  saveCurrentBucket() {
    const pd = this.profileData[this.activeProfileId];
    if (pd && this.leftPanel.bucket) {
      pd[this.leftPanel.bucket] = [...this.bucketFiles];
    }
  }

  switchProfile(id: string) {
    if (id === this.activeProfileId) return;
    this.saveCurrentBucket();
    const profile = this.profiles.find((p) => p.id === id);
    if (profile) this.connectProfile(profile);
  }

  // S3V window size (percentage of desktop width)
  s3vWidth = $state(78);
  s3vHeight = $state(100);

  leftPanelFiles = $derived.by(() => {
    let files = sortFiles(filesAtPath(this.bucketFiles, this.leftPanel.currentPath), this.sortField, this.sortOrder);
    if (this.searchQuery) files = files.filter((f) => f.name.toLowerCase().includes(this.searchQuery.toLowerCase()));
    return files;
  });
  rightPanelFiles = $derived.by(() => {
    let files = sortFiles(filesAtPath(this.bucketFiles, this.rightPanel.currentPath), this.sortField, this.sortOrder);
    if (this.searchQuery) files = files.filter((f) => f.name.toLowerCase().includes(this.searchQuery.toLowerCase()));
    return files;
  });

  folderPaths = $derived.by(() => {
    const paths = new Set<string>();
    for (const f of this.bucketFiles) {
      // Add parent path segments
      if (f.path) {
        const parts = f.path.split('/').filter(Boolean);
        let acc = '';
        for (const part of parts) {
          acc += part + '/';
          paths.add(acc);
        }
      }
      // Add folder itself as a path
      if (f.type === 'folder') {
        paths.add(f.path + f.name + '/');
      }
    }
    return paths;
  });

  transfers = $state<TransferItem[]>([]);
  drag = $state<DragState>({ active: false, file: null, source: 'desktop', x: 0, y: 0 });
  contextMenu = $state<ContextMenuState>({ visible: false, x: 0, y: 0, target: null, source: 'desktop' });

  activeTransferCount = $derived(this.transfers.filter((t) => t.status === 'active' || t.status === 'queued').length);

  setSort(field: SortField) {
    if (this.sortField === field) {
      this.sortOrder = this.sortOrder === 'asc' ? 'desc' : 'asc';
    } else {
      this.sortField = field;
      this.sortOrder = 'asc';
    }
  }

  toggleSelect(fileId: string) {
    const next = new Set(this.selectedFiles);
    if (next.has(fileId)) next.delete(fileId);
    else next.add(fileId);
    this.selectedFiles = next;
  }

  clearSelection() {
    this.selectedFiles = new Set();
  }

  toggleDesktopSelect(fileId: string, additive = false) {
    const next = additive ? new Set(this.selectedDesktopFiles) : new Set<string>();
    if (next.has(fileId)) next.delete(fileId);
    else next.add(fileId);
    this.selectedDesktopFiles = next;
  }

  setDesktopSelection(ids: Set<string>) {
    this.selectedDesktopFiles = ids;
  }

  clearDesktopSelection() {
    this.selectedDesktopFiles = new Set();
  }

  deleteSelected() {
    if (this.selectedFiles.size > 0) {
      // Collect folder prefixes to cascade-delete children
      const folderPrefixes: string[] = [];
      for (const f of this.bucketFiles) {
        if (this.selectedFiles.has(f.id) && f.type === 'folder') {
          folderPrefixes.push(f.path + f.name + '/');
        }
      }
      this.bucketFiles = this.bucketFiles.filter((f) => {
        if (this.selectedFiles.has(f.id)) return false;
        return !folderPrefixes.some((prefix) => f.path.startsWith(prefix));
      });
      this.clearSelection();
    }
  }

  downloadSelected() {
    const files = this.bucketFiles.filter((f) => this.selectedFiles.has(f.id) && f.type === 'file');
    for (const file of files) {
      this.downloadToDesktop(file);
    }
    this.clearSelection();
  }

  navigatePanel(side: 'left' | 'right', path: string) {
    const panel = side === 'left' ? this.leftPanel : this.rightPanel;
    panel.currentPath = path;
    this.clearSelection();
  }

  startDrag(file: VirtualFile, source: DragState['source'], x: number, y: number) {
    this.drag = { active: true, file, source, x, y, modifier: false };
  }

  updateDrag(x: number, y: number, modifier: boolean = false) {
    this.drag.x = x;
    this.drag.y = y;
    this.drag.modifier = modifier;
  }

  endDrag() {
    this.drag = { active: false, file: null, source: 'desktop', x: 0, y: 0, modifier: false };
  }

  moveBetweenPanels(file: VirtualFile, targetSide: 'left' | 'right', sourceSide: 'left' | 'right') {
    const panel = targetSide === 'left' ? this.leftPanel : this.rightPanel;
    const newFile: VirtualFile = { ...file, id: `vf-mv-${Date.now()}`, path: panel.currentPath, lastModified: new Date().toISOString() };
    this.bucketFiles.push(newFile);
    // Remove from source
    this.bucketFiles = this.bucketFiles.filter((f) => f.id !== file.id);
    this.addTransfer(file, 'copy');
  }

  addTransfer(file: VirtualFile, direction: TransferItem['direction']): string {
    const id = `tr-${++_transferId}`;
    this.transfers.push({ id, file: { ...file }, direction, progress: 0, status: 'queued' });
    if (!this.transferPanelExpanded) this.transferPanelExpanded = true;
    this.simulateTransfer(id);
    return id;
  }

  private simulateTransfer(id: string) {
    const step = () => {
      const item = this.transfers.find((t) => t.id === id);
      if (!item) return;
      if (item.status === 'queued') item.status = 'active';
      item.progress = Math.min(item.progress + Math.random() * 25 + 10, 100);
      if (item.progress >= 100) {
        item.progress = 100;
        item.status = 'completed';
        return;
      }
      setTimeout(step, 200 + Math.random() * 300);
    };
    setTimeout(step, 100);
  }

  clearCompletedTransfers() {
    this.transfers = this.transfers.filter((t) => t.status !== 'completed' && t.status !== 'failed');
  }

  removeTransfer(id: string) {
    this.transfers = this.transfers.filter((t) => t.id !== id);
  }

  uploadToPanel(file: VirtualFile, side: 'left' | 'right') {
    const panel = side === 'left' ? this.leftPanel : this.rightPanel;
    const newFile: VirtualFile = { ...file, id: `vf-upload-${Date.now()}`, path: panel.currentPath, lastModified: new Date().toISOString() };
    this.bucketFiles.push(newFile);
    this.addTransfer(file, 'upload');
  }

  private nextDesktopSlot(): { x: number; y: number } {
    // Grid: columns from right, rows from top. Cell = 80x90px approx = ~6% x ~14%
    const colW = 6;
    const rowH = 14;
    const maxCols = 3;
    const maxRows = 7;
    const occupied = new Set(this.desktopFiles.map((d) => `${d.x},${d.y}`));
    for (let col = 0; col < maxCols; col++) {
      for (let row = 0; row < maxRows; row++) {
        const x = 92 - col * colW;
        const y = 2 + row * rowH;
        if (!occupied.has(`${x},${y}`)) return { x, y };
      }
    }
    return { x: 83, y: 2 };
  }

  downloadToDesktop(file: VirtualFile, x?: number, y?: number) {
    const newFile: VirtualFile = { ...file, id: `vf-dl-${Date.now()}` };
    const pos = (x !== undefined && y !== undefined) ? { x, y } : this.nextDesktopSlot();
    this.desktopFiles.push({ file: newFile, ...pos });
    this.addTransfer(file, 'download');
  }

  copyBetweenPanels(file: VirtualFile, targetSide: 'left' | 'right') {
    const panel = targetSide === 'left' ? this.leftPanel : this.rightPanel;
    const newFile: VirtualFile = { ...file, id: `vf-cp-${Date.now()}`, path: panel.currentPath, lastModified: new Date().toISOString() };
    this.bucketFiles.push(newFile);
    this.addTransfer(file, 'copy');
  }

  deleteFile(file: VirtualFile, source: DragState['source']) {
    if (source === 'desktop') {
      this.desktopFiles = this.desktopFiles.filter((d) => d.file.id !== file.id);
    } else if (file.type === 'folder') {
      // Delete folder and all children whose path starts with this folder's full path
      const folderPrefix = file.path + file.name + '/';
      this.bucketFiles = this.bucketFiles.filter((f) => f.id !== file.id && !f.path.startsWith(folderPrefix));
    } else {
      this.bucketFiles = this.bucketFiles.filter((f) => f.id !== file.id);
    }
  }

  renameFile(file: VirtualFile, newName: string) {
    const target = this.bucketFiles.find((f) => f.id === file.id);
    if (target) target.name = newName;
  }

  openContextMenu(x: number, y: number, target: VirtualFile, source: ContextMenuState['source']) {
    this.contextMenu = { visible: true, x, y, target, source };
  }

  closeContextMenu() {
    this.contextMenu.visible = false;
  }
}

export const store = new PlaygroundStore();
