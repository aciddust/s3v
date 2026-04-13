const STORAGE_KEY = 's3v-settings';

interface Settings {
  folderDragDownload: boolean;
  autoShowTransfers: boolean;
}

const defaults: Settings = {
  autoShowTransfers: true,
  folderDragDownload: false,
};

class SettingsStore {
  autoShowTransfers = $state(defaults.autoShowTransfers);
  folderDragDownload = $state(defaults.folderDragDownload);

  load() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        const parsed = JSON.parse(raw) as Partial<Settings>;
        if (typeof parsed.folderDragDownload === 'boolean') {
          this.folderDragDownload = parsed.folderDragDownload;
        }
        if (typeof parsed.autoShowTransfers === 'boolean') {
          this.autoShowTransfers = parsed.autoShowTransfers;
        }
      }
    } catch {
      // ignore
    }
  }

  private save() {
    const data: Settings = {
      folderDragDownload: this.folderDragDownload,
      autoShowTransfers: this.autoShowTransfers,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  }

  setFolderDragDownload(value: boolean) {
    this.folderDragDownload = value;
    this.save();
  }

  setAutoShowTransfers(value: boolean) {
    this.autoShowTransfers = value;
    this.save();
  }
}

export const settingsStore = new SettingsStore();
