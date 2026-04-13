export type Locale = 'ko' | 'en' | 'ja';

const translations: Record<Locale, Record<string, string>> = {
  ko: {
    // Header
    'nav.features': '기능 소개',
    'nav.playground': '체험',
    'nav.download': '다운로드',
    'nav.contact': '문의하기',

    // Hero
    'hero.title': '여러 S3 버킷의 파일을\n동시에 관리하는 가장 쉬운 방법',
    'hero.cta': '지금 다운로드',

    // Features
    'features.heading': '기능 소개',
    'features.dualPanel.title': '듀얼 패널',
    'features.dualPanel.desc':
      '두 개의 버킷을 나란히 열어 파일을 직관적으로 비교하고 관리할 수 있습니다.',
    'features.dnd.title': '드래그 앤 드롭',
    'features.dnd.desc': '파일을 드래그하여 이동, 복사, 업로드, 다운로드할 수 있습니다.',
    'features.multiConnect.title': '멀티 커넥트',
    'features.multiConnect.desc':
      '여러 S3 호환 스토리지 프로필을 등록하고 자유롭게 전환할 수 있습니다.',

    // Playground
    'playground.heading': '직접 체험해보세요',
    'playground.desc': 'S3V의 핵심 기능을 브라우저에서 바로 사용해볼 수 있습니다.',
    'playground.mobile': '데스크톱 브라우저에서 S3V의 드래그 앤 드롭 체험을 이용해보세요.',

    // Download
    'download.heading': '다운로드',
    'download.desc': 'S3V를 다운로드하고 바로 시작하세요.',
    'download.button': '다운로드',
    'download.preparing': '준비 중',
    'download.mac.title': 'macOS 보안 안내',
    'download.mac.desc':
      'macOS는 인터넷에서 다운로드한 앱의 실행을 차단할 수 있습니다. 설치 후 보안 경고가 표시되면 터미널에서 다음 명령어를 실행하세요:',
    'download.mac.hint':
      '설치 후 실행이 안된다면 이 명령어를 실행해주세요. macOS가 적용한 격리 플래그를 제거합니다.',
    'download.mac.cancel': '취소',
    'download.mac.proceed': '다운로드 계속',
    'download.tab.direct': '직접 다운로드',
    'download.tab.homebrew': 'Homebrew',
    'download.brew.desc': 'macOS 사용자는 Homebrew로 간편하게 설치할 수 있습니다.',
    'download.brew.update': '업데이트:',

    // Contact
    'contact.heading': '문의하기',
    'contact.name': '이름',
    'contact.email': '이메일',
    'contact.message': '문의 내용',
    'contact.submit': '문의하기',
    'contact.sending': '전송 중...',
    'contact.success': '문의가 접수되었습니다. 감사합니다!',
    'contact.error': '전송에 실패했습니다. 다시 시도해주세요.',
  },
  en: {
    // Header
    'nav.features': 'Features',
    'nav.playground': 'Playground',
    'nav.download': 'Download',
    'nav.contact': 'Contact',

    // Hero
    'hero.title': 'The easiest way to manage files across multiple S3-compatible buckets',
    'hero.cta': 'Download Now',

    // Features
    'features.heading': 'Features',
    'features.dualPanel.title': 'Dual Panel',
    'features.dualPanel.desc':
      'Open two buckets side by side to intuitively compare and manage files.',
    'features.dnd.title': 'Drag & Drop',
    'features.dnd.desc': 'Drag files to move, copy, upload, and download seamlessly.',
    'features.multiConnect.title': 'Multi Connect',
    'features.multiConnect.desc':
      'Register multiple S3-compatible storage profiles and switch freely.',

    // Playground
    'playground.heading': 'Try it yourself',
    'playground.desc': 'Experience the core features of S3V right in your browser.',
    'playground.mobile': 'Try the S3V drag & drop experience on a desktop browser.',

    // Download
    'download.heading': 'Download',
    'download.desc': 'Download S3V and get started right away.',
    'download.button': 'Download',
    'download.preparing': 'Coming Soon',
    'download.mac.title': 'macOS Security Notice',
    'download.mac.desc':
      'macOS may block the app from opening because it was downloaded from the internet. After installing, if you see a security warning, run the following command in Terminal:',
    'download.mac.hint':
      'If the app does not launch after installation, run this command. It removes the quarantine flag applied by macOS.',
    'download.mac.cancel': 'Cancel',
    'download.mac.proceed': 'Download Anyway',
    'download.tab.direct': 'Direct Download',
    'download.tab.homebrew': 'Homebrew',
    'download.brew.desc': 'macOS users can install S3V easily via Homebrew.',
    'download.brew.update': 'Update:',

    // Contact
    'contact.heading': 'Contact',
    'contact.name': 'Name',
    'contact.email': 'Email',
    'contact.message': 'Message',
    'contact.submit': 'Send',
    'contact.sending': 'Sending...',
    'contact.success': 'Your message has been sent. Thank you!',
    'contact.error': 'Failed to send. Please try again.',
  },
  ja: {
    // Header
    'nav.features': '機能紹介',
    'nav.playground': '体験',
    'nav.download': 'ダウンロード',
    'nav.contact': 'お問い合わせ',

    // Hero
    'hero.title': '複数のS3バケットのファイルを\n同時に管理する最も簡単な方法',
    'hero.cta': '今すぐダウンロード',

    // Features
    'features.heading': '機能紹介',
    'features.dualPanel.title': 'デュアルパネル',
    'features.dualPanel.desc': '2つのバケットを並べて開き、ファイルを直感的に比較・管理できます。',
    'features.dnd.title': 'ドラッグ＆ドロップ',
    'features.dnd.desc': 'ファイルをドラッグして移動、コピー、アップロード、ダウンロードできます。',
    'features.multiConnect.title': 'マルチコネクト',
    'features.multiConnect.desc':
      '複数のS3互換ストレージプロファイルを登録し、自由に切り替えられます。',

    // Playground
    'playground.heading': '実際に体験してみましょう',
    'playground.desc': 'S3Vの主要機能をブラウザですぐにお試しいただけます。',
    'playground.mobile': 'デスクトップブラウザでS3Vのドラッグ＆ドロップ体験をお試しください。',

    // Download
    'download.heading': 'ダウンロード',
    'download.desc': 'S3Vをダウンロードしてすぐに始めましょう。',
    'download.button': 'ダウンロード',
    'download.preparing': '準備中',
    'download.mac.title': 'macOSセキュリティについて',
    'download.mac.desc':
      'macOSはインターネットからダウンロードしたアプリの実行をブロックする場合があります。インストール後にセキュリティ警告が表示された場合は、ターミナルで以下のコマンドを実行してください：',
    'download.mac.hint':
      'インストール後に起動できない場合は、このコマンドを実行してください。macOSが適用した隔離フラグを解除します。',
    'download.mac.cancel': 'キャンセル',
    'download.mac.proceed': 'ダウンロードを続行',
    'download.tab.direct': '直接ダウンロード',
    'download.tab.homebrew': 'Homebrew',
    'download.brew.desc': 'macOSユーザーはHomebrewで簡単にインストールできます。',
    'download.brew.update': '更新:',

    // Contact
    'contact.heading': 'お問い合わせ',
    'contact.name': 'お名前',
    'contact.email': 'メールアドレス',
    'contact.message': 'お問い合わせ内容',
    'contact.submit': '送信',
    'contact.sending': '送信中...',
    'contact.success': 'お問い合わせを受け付けました。ありがとうございます！',
    'contact.error': '送信に失敗しました。もう一度お試しください。',
  },
};

let _locale = $state<Locale>('ko');

export function getLocale(): Locale {
  return _locale;
}

export function setLocale(locale: Locale) {
  _locale = locale;
}

export function t(key: string): string {
  return translations[_locale][key] ?? key;
}
