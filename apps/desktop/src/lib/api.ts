import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  schemaVersion: number;
  hotkey: string;
  launchAtStartup: boolean;
  autoOpenSafeUrls: boolean;
  copyNonUrlPayloads: boolean;
  notificationsEnabled: boolean;
}

export interface SettingsSnapshot {
  settings: AppSettings;
  activeHotkey: string | null;
  hotkeyRegistered: boolean;
}

export interface SettingsView {
  snapshot: SettingsSnapshot;
  version: string;
  build: string;
}

export interface SettingsUpdate {
  hotkey: string;
  launchAtStartup: boolean;
  autoOpenSafeUrls: boolean;
  copyNonUrlPayloads: boolean;
  notificationsEnabled: boolean;
}

export interface CommandError {
  code: string;
  message: string;
}

export function getSettings(): Promise<SettingsView> {
  return invoke<SettingsView>('get_settings');
}

export function updateSettings(request: SettingsUpdate): Promise<SettingsView> {
  return invoke<SettingsView>('update_settings', { request });
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
