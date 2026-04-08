export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  if (i === 0) return `${bytes} B`;
  return `${value.toFixed(1)} ${units[i]}`;
}

export function formatSpeed(bytesPerSec: number): string {
  return `${formatFileSize(bytesPerSec)}/s`;
}

export function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString('ko-KR', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

export function getFileExtension(key: string): string {
  const name = getFileName(key);
  const dotIndex = name.lastIndexOf('.');
  if (dotIndex < 0 || dotIndex === 0) return '';
  return name.slice(dotIndex + 1).toLowerCase();
}

export function getFileName(key: string): string {
  const trimmed = key.endsWith('/') ? key.slice(0, -1) : key;
  const slashIndex = trimmed.lastIndexOf('/');
  return slashIndex >= 0 ? trimmed.slice(slashIndex + 1) : trimmed;
}
