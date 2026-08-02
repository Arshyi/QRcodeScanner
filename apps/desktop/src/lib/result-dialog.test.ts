import { describe, expect, it } from 'vitest';
import { actionForDialogKey, resultKindLabel } from './result-dialog';

describe('result chooser keyboard and labels', () => {
  it('routes Escape to the safe dismiss action', () => {
    expect(actionForDialogKey('Escape')).toBe('dismiss');
    expect(actionForDialogKey('Enter')).toBeNull();
  });

  it('uses explicit non-color classification labels', () => {
    expect(resultKindLabel('https_url')).toBe('HTTPS URL');
    expect(resultKindLabel('malformed_url')).toBe('Malformed URL-like text');
    expect(resultKindLabel('blocked_scheme')).toBe('Blocked scheme');
  });
});
