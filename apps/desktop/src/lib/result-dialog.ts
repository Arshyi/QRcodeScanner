import type { ResultKind } from './api';

const labels: Record<ResultKind, string> = {
  http_url: 'HTTP URL',
  https_url: 'HTTPS URL',
  plain_text: 'Plain text',
  malformed_url: 'Malformed URL-like text',
  blocked_scheme: 'Blocked scheme',
  blocked_url: 'Blocked URL',
  unsafe_text: 'Unsafe text',
  binary: 'Binary payload',
};

export function resultKindLabel(kind: ResultKind): string {
  return labels[kind];
}

export function actionForDialogKey(key: string): 'dismiss' | null {
  return key === 'Escape' ? 'dismiss' : null;
}
