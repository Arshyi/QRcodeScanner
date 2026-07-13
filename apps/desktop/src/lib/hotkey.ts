const modifierCodes = new Set([
  'ControlLeft',
  'ControlRight',
  'AltLeft',
  'AltRight',
  'ShiftLeft',
  'ShiftRight',
  'MetaLeft',
  'MetaRight',
]);

export function hotkeyFromKeyboard(event: KeyboardEvent): string | null {
  if (modifierCodes.has(event.code)) {
    return null;
  }
  const key = portableKey(event.code);
  if (key === null) {
    return null;
  }
  const parts: string[] = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Super');
  if (parts.length === 0) {
    return null;
  }
  parts.push(key);
  return parts.join('+');
}

function portableKey(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  const functionMatch = /^F([1-9]|1[0-9]|2[0-4])$/.exec(code);
  return functionMatch === null ? null : `F${functionMatch[1]}`;
}
