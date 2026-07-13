import { describe, expect, it } from 'vitest';
import { hotkeyFromKeyboard } from './hotkey';

describe('hotkeyFromKeyboard', () => {
  it('creates the canonical portable order', () => {
    const event = new KeyboardEvent('keydown', {
      code: 'KeyK',
      ctrlKey: true,
      altKey: true,
      shiftKey: true,
    });
    expect(hotkeyFromKeyboard(event)).toBe('Ctrl+Alt+Shift+K');
  });

  it('requires a modifier', () => {
    expect(hotkeyFromKeyboard(new KeyboardEvent('keydown', { code: 'KeyQ' }))).toBeNull();
  });

  it('rejects unsupported keys and modifier-only events', () => {
    expect(
      hotkeyFromKeyboard(new KeyboardEvent('keydown', { code: 'Space', ctrlKey: true })),
    ).toBeNull();
    expect(
      hotkeyFromKeyboard(new KeyboardEvent('keydown', { code: 'ControlLeft', ctrlKey: true })),
    ).toBeNull();
  });
});
