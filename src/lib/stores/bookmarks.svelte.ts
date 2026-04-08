export interface Bookmark {
  id: string;
  profileId: string;
  bucket: string;
  prefix: string;
  label: string;
}

const STORAGE_KEY = 's3v:bookmarks';

class BookmarkStore {
  bookmarks = $state<Bookmark[]>([]);

  load(): void {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      this.bookmarks = raw ? (JSON.parse(raw) as Bookmark[]) : [];
    } catch {
      this.bookmarks = [];
    }
  }

  private save(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.bookmarks));
    } catch {
      // localStorage may be unavailable in some contexts
    }
  }

  add(bookmark: Omit<Bookmark, 'id'>): void {
    const id = `bm_${Date.now()}_${Math.random().toString(36).slice(2)}`;
    this.bookmarks = [...this.bookmarks, { id, ...bookmark }];
    this.save();
  }

  remove(id: string): void {
    this.bookmarks = this.bookmarks.filter((b) => b.id !== id);
    this.save();
  }

  getForProfile(profileId: string): Bookmark[] {
    return this.bookmarks.filter((b) => b.profileId === profileId);
  }
}

export const bookmarkStore = new BookmarkStore();
