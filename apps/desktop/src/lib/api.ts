import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  schemaVersion: number;
  hotkey: string;
  launchAtStartup: boolean;
  autoOpenSafeUrls: boolean;
  copyNonUrlPayloads: boolean;
  notificationsEnabled: boolean;
  scanMonitorId: string | null;
  onboardingCompleted: boolean;
}

export interface MonitorInfo {
  id: string;
  label: string;
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactorPercent: number;
  rotationDegrees: number;
  isPrimary: boolean;
}

export interface SettingsSnapshot {
  settings: AppSettings;
  activeHotkey: string | null;
  hotkeyRegistered: boolean;
}

export interface SettingsView {
  snapshot: SettingsSnapshot;
  version: string;
  commit: string;
  build: string;
  monitors: MonitorInfo[];
  configuredMonitorAvailable: boolean;
  monitorError: string | null;
}

export interface SettingsUpdate {
  hotkey: string;
  launchAtStartup: boolean;
  autoOpenSafeUrls: boolean;
  copyNonUrlPayloads: boolean;
  notificationsEnabled: boolean;
  scanMonitorId: string | null;
}

export type ResultKind =
  | 'http_url'
  | 'https_url'
  | 'plain_text'
  | 'malformed_url'
  | 'blocked_scheme'
  | 'blocked_url'
  | 'unsafe_text'
  | 'binary';

export interface ResultItemView {
  index: number;
  kind: ResultKind;
  preview: string;
  detail: string | null;
  canOpen: boolean;
  canCopy: boolean;
}

export interface PendingResultsView {
  sessionId: number;
  items: ResultItemView[];
}

export type ResultAction = 'open' | 'copy' | 'copy_all' | 'dismiss';

export interface ResultActionRequest {
  sessionId: number;
  action: ResultAction;
  index: number | null;
}

export interface ResultActionOutcome {
  message: string;
  close: boolean;
}

export interface CommandError {
  code: string;
  message: string;
}

export interface CopyDiagnosticsOutcome {
  message: string;
}

export function getSettings(): Promise<SettingsView> {
  return invoke<SettingsView>('get_settings');
}

export function updateSettings(request: SettingsUpdate): Promise<SettingsView> {
  return invoke<SettingsView>('update_settings', { request });
}

export function completeOnboarding(): Promise<SettingsView> {
  return invoke<SettingsView>('complete_onboarding');
}

export function copyDiagnostics(): Promise<CopyDiagnosticsOutcome> {
  return invoke<CopyDiagnosticsOutcome>('copy_diagnostics');
}

export function getPendingResults(): Promise<PendingResultsView> {
  return invoke<PendingResultsView>('get_pending_results');
}

export function performResultAction(request: ResultActionRequest): Promise<ResultActionOutcome> {
  return invoke<ResultActionOutcome>('perform_result_action', { request });
}

export function commandMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }
  return 'QRForge could not update this setting.';
}
