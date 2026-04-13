export const APP_NAME = 'S3V';
export const APP_VERSION = '0.4.5';
export const APP_DESCRIPTION = 'S3-Compatible Storage Client';
export const GITHUB_URL = 'https://github.com/aciddust/s3v';
export const FEEDBACK_API = 'https://web.d3fau1t.net/api/feedback';
export const SUPPORT_EMAIL = 'acidlab.help@gmail.com';

const RELEASE_BASE = `${GITHUB_URL}/releases/download/v${APP_VERSION}`;

export const DOWNLOADS: Record<string, { url: string; label: string }> = {
  macos: { url: `${RELEASE_BASE}/s3v_${APP_VERSION}_universal.dmg`, label: 'macOS (Universal)' },
  'windows-x64': {
    url: `${RELEASE_BASE}/s3v_${APP_VERSION}_x64-setup.exe`,
    label: 'Windows (x64)',
  },
  'windows-arm64': {
    url: `${RELEASE_BASE}/s3v_${APP_VERSION}_arm64-setup.exe`,
    label: 'Windows (ARM64)',
  },
};

export const FEATURES = [
  {
    title: '듀얼 패널',
    description: '두 개의 버킷을 나란히 열어 파일을 직관적으로 비교하고 관리할 수 있습니다.',
    icon: 'PanelsTopLeft',
  },
  {
    title: '드래그 앤 드롭',
    description: '파일을 드래그하여 이동, 복사, 업로드, 다운로드할 수 있습니다.',
    icon: 'MousePointerClick',
  },
  {
    title: '멀티 커넥트',
    description: '여러 S3 호환 스토리지 프로필을 등록하고 자유롭게 전환할 수 있습니다.',
    icon: 'PlugZap',
  },
] as const;
