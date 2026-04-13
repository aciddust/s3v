import type { VirtualFile, DesktopItem } from './types';

let _id = 0;
function uid(): string {
  return `vf-${++_id}`;
}

export const BUCKET_NAME = 'demo-bucket';

export const BUCKET_FILES: VirtualFile[] = [
  { id: uid(), name: 'project-assets', type: 'folder', size: 0, path: '', contentType: null, lastModified: '2026-04-08T09:00:00Z' },
  { id: uid(), name: 'backups', type: 'folder', size: 0, path: '', contentType: null, lastModified: '2026-04-05T14:30:00Z' },

  { id: uid(), name: 'images', type: 'folder', size: 0, path: 'project-assets/', contentType: null, lastModified: '2026-04-09T11:20:00Z' },
  { id: uid(), name: 'logs', type: 'folder', size: 0, path: 'project-assets/', contentType: null, lastModified: '2026-04-10T06:00:00Z' },
  { id: uid(), name: 'config.json', type: 'file', size: 4_096, path: 'project-assets/', contentType: 'application/json', lastModified: '2026-04-07T15:45:00Z' },
  { id: uid(), name: 'data.csv', type: 'file', size: 890_000, path: 'project-assets/', contentType: 'text/csv', lastModified: '2026-04-09T08:30:00Z' },

  { id: uid(), name: 'hero.png', type: 'file', size: 2_400_000, path: 'project-assets/images/', contentType: 'image/png', lastModified: '2026-04-06T10:00:00Z' },
  { id: uid(), name: 'logo.svg', type: 'file', size: 12_000, path: 'project-assets/images/', contentType: 'image/svg+xml', lastModified: '2026-04-03T16:20:00Z' },
  { id: uid(), name: 'thumbnail.jpg', type: 'file', size: 340_000, path: 'project-assets/images/', contentType: 'image/jpeg', lastModified: '2026-04-08T13:10:00Z' },

  { id: uid(), name: 'access.log', type: 'file', size: 1_100_000, path: 'project-assets/logs/', contentType: 'text/plain', lastModified: '2026-04-10T05:55:00Z' },
  { id: uid(), name: 'error.log', type: 'file', size: 256_000, path: 'project-assets/logs/', contentType: 'text/plain', lastModified: '2026-04-10T05:55:00Z' },

  { id: uid(), name: 'db-2026-04.sql', type: 'file', size: 15_000_000, path: 'backups/', contentType: 'application/sql', lastModified: '2026-04-01T02:00:00Z' },
  { id: uid(), name: 'env-snapshot.tar.gz', type: 'file', size: 3_200_000, path: 'backups/', contentType: 'application/gzip', lastModified: '2026-04-04T22:00:00Z' },
];

export const INITIAL_DESKTOP_FILES: DesktopItem[] = [
  { file: { id: uid(), name: 'report.csv', type: 'file', size: 524_000, path: '', contentType: 'text/csv', lastModified: '2026-04-10T09:00:00Z' }, x: 92, y: 2 },
  { file: { id: uid(), name: 'banner.png', type: 'file', size: 1_800_000, path: '', contentType: 'image/png', lastModified: '2026-04-09T17:30:00Z' }, x: 92, y: 16 },
  { file: { id: uid(), name: 'backups', type: 'folder', size: 0, path: '', contentType: null, lastModified: null }, x: 92, y: 30 },
];
