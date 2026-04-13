import {
  setLocale as _setLocale,
  getLocale as _getLocale,
  locales,
  baseLocale,
  localStorageKey,
  overwriteGetLocale,
} from '$lib/paraglide/runtime';

export type Locale = (typeof locales)[number];

// Svelte 5 reactive locale — makes all m.xxx() calls reactive
let _locale = $state<Locale>(baseLocale);

// Wire Paraglide's getLocale to read from our $state
overwriteGetLocale(() => _locale);

function isLocale(tag: string): tag is Locale {
  return (locales as readonly string[]).includes(tag);
}

function detectLocale(): Locale {
  const stored = localStorage.getItem(localStorageKey);
  if (stored && isLocale(stored)) return stored;

  const browserLang = navigator.language.split('-')[0];
  if (isLocale(browserLang)) return browserLang;

  return baseLocale;
}

export function initLocale() {
  const locale = detectLocale();
  _locale = locale;
  _setLocale(locale, { reload: false });
}

export function switchLocale(locale: Locale) {
  _locale = locale;
  _setLocale(locale, { reload: false });
  localStorage.setItem(localStorageKey, locale);
}

export function getLocale(): Locale {
  return _locale;
}

export { locales, baseLocale };
