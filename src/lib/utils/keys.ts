export interface KeyBinding {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  action: string;
}

export const keyBindings: KeyBinding[] = [
  { key: 'Delete', action: 'delete' },
  { key: 'Backspace', action: 'delete' },
  { key: 'a', ctrl: true, action: 'selectAll' },
  { key: 'c', ctrl: true, action: 'copyKey' },
  { key: 'F2', action: 'rename' },
  { key: 'F5', action: 'refresh' },
];

export function matchBinding(e: KeyboardEvent): string | null {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;

  for (const binding of keyBindings) {
    const ctrlMatch = binding.ctrl ? (isMac ? e.metaKey : e.ctrlKey) : true;
    const shiftMatch = binding.shift ? e.shiftKey : true;
    const altMatch = binding.alt ? e.altKey : true;

    if (e.key === binding.key && ctrlMatch && shiftMatch && altMatch) {
      return binding.action;
    }
  }

  return null;
}
