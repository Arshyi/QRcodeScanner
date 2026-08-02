<script lang="ts">
  import { onMount } from 'svelte';
  import {
    commandMessage,
    completeOnboarding,
    getSettings,
    updateSettings,
    type AppSettings,
    type SettingsView,
  } from './lib/api';
  import { hotkeyFromKeyboard } from './lib/hotkey';

  let view: SettingsView | null = null;
  let loading = true;
  let saving = false;
  let errorMessage = '';
  let savedMessage = '';
  let capturingHotkey = false;
  let onboardingButton: HTMLButtonElement | null = null;

  onMount(async () => {
    await refresh();
    if (view !== null && !view.snapshot.settings.onboardingCompleted) {
      onboardingButton?.focus();
    }
  });

  async function refresh(): Promise<void> {
    loading = true;
    errorMessage = '';
    try {
      view = await getSettings();
    } catch (error) {
      errorMessage = commandMessage(error);
    } finally {
      loading = false;
    }
  }

  async function save(settings: AppSettings): Promise<boolean> {
    if (saving) return false;
    saving = true;
    errorMessage = '';
    savedMessage = '';
    try {
      view = await updateSettings({
        hotkey: settings.hotkey,
        launchAtStartup: settings.launchAtStartup,
        autoOpenSafeUrls: settings.autoOpenSafeUrls,
        copyNonUrlPayloads: settings.copyNonUrlPayloads,
        notificationsEnabled: settings.notificationsEnabled,
        scanMonitorId: settings.scanMonitorId,
      });
      savedMessage = 'Saved';
      return true;
    } catch (error) {
      errorMessage = commandMessage(error);
      return false;
    } finally {
      saving = false;
    }
  }

  function change(field: keyof AppSettings, value: boolean): void {
    if (view === null) return;
    void save({ ...view.snapshot.settings, [field]: value });
  }

  function changeMonitor(value: string): void {
    if (view === null) return;
    void save({
      ...view.snapshot.settings,
      scanMonitorId: value === '' ? null : value,
    });
  }

  async function captureHotkey(event: KeyboardEvent): Promise<void> {
    event.preventDefault();
    if (!capturingHotkey || view === null) return;
    if (event.key === 'Escape' || event.code === 'Escape') {
      capturingHotkey = false;
      return;
    }
    const hotkey = hotkeyFromKeyboard(event);
    if (hotkey === null) {
      errorMessage = 'Hold at least one modifier and press a non-reserved letter, digit, or F-key.';
      return;
    }
    const saved = await save({ ...view.snapshot.settings, hotkey });
    capturingHotkey = !saved;
  }

  async function dismissOnboarding(): Promise<void> {
    if (saving) return;
    saving = true;
    errorMessage = '';
    try {
      view = await completeOnboarding();
    } catch (error) {
      errorMessage = commandMessage(error);
    } finally {
      saving = false;
    }
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (capturingHotkey) {
      void captureHotkey(event);
      return;
    }
    if (event.key === 'Escape' && view !== null && !view.snapshot.settings.onboardingCompleted) {
      event.preventDefault();
      void dismissOnboarding();
    }
  }
</script>

<svelte:head>
  <title>QRForge Settings</title>
</svelte:head>

<svelte:window onkeydown={handleWindowKeydown} />

<main>
  <header>
    <div class="mark" aria-hidden="true"><span></span><span></span><span></span></div>
    <div>
      <p class="eyebrow">QRForge</p>
      <h1>Settings</h1>
    </div>
    <span class="local-pill">Local only</span>
  </header>

  {#if loading}
    <p class="state" role="status">Loading settings…</p>
  {:else if view === null}
    <section class="notice error" role="alert">{errorMessage}</section>
  {:else if !view.snapshot.settings.onboardingCompleted}
    <dialog
      open
      class="card onboarding"
      aria-labelledby="welcome-heading"
      aria-describedby="welcome-summary"
    >
      <p class="eyebrow">Welcome</p>
      <h2 id="welcome-heading">Scan your screen without uploading it</h2>
      <p id="welcome-summary">
        QRForge captures the selected display in native memory, decodes locally, and immediately
        releases the pixels. Screenshots are never uploaded or retained.
      </p>
      <ul>
        <li>
          Press <kbd>{view.snapshot.settings.hotkey}</kbd> anywhere while QRForge is running.
        </li>
        <li>Approved HTTP and HTTPS links may open; ordinary text may be copied.</li>
        <li>Blocked, malformed, binary, and multi-code results are never opened automatically.</li>
        <li>Use the tray icon to scan, reopen Settings, or quit QRForge completely.</li>
      </ul>
      <button
        class="primary"
        type="button"
        bind:this={onboardingButton}
        onclick={() => void dismissOnboarding()}
        disabled={saving}
      >
        Continue to settings
      </button>
      <p class="keyboard-hint">Press Escape to dismiss this introduction.</p>
      {#if errorMessage}
        <p class="notice error" role="alert">{errorMessage}</p>
      {/if}
    </dialog>
  {:else}
    <section class="card hotkey-card" aria-labelledby="hotkey-heading">
      <div>
        <p class="label" id="hotkey-heading">Scan shortcut</p>
        <p class="hint" id="hotkey-hint">Works globally while QRForge is running.</p>
      </div>
      <button
        class:capturing={capturingHotkey}
        class="hotkey"
        type="button"
        aria-describedby="hotkey-hint hotkey-registration"
        aria-pressed={capturingHotkey}
        onclick={() => {
          capturingHotkey = !capturingHotkey;
          errorMessage = '';
          savedMessage = '';
        }}
        disabled={saving}
      >
        {capturingHotkey ? 'Press shortcut…' : view.snapshot.settings.hotkey}
      </button>
      <p
        class:warning={!view.snapshot.hotkeyRegistered}
        class="registration"
        id="hotkey-registration"
        role={view.snapshot.hotkeyRegistered ? undefined : 'alert'}
      >
        {view.snapshot.hotkeyRegistered
          ? `Active: ${view.snapshot.activeHotkey}`
          : 'Not registered — choose another shortcut. Tray Scan Now remains available.'}
      </p>
    </section>

    <section class="card monitor-card" aria-labelledby="monitor-heading">
      <div>
        <label class="label" id="monitor-heading" for="scan-monitor">Display to scan</label>
        <p class="hint" id="monitor-hint">Captured at its full physical resolution.</p>
      </div>
      <select
        id="scan-monitor"
        aria-describedby="monitor-hint monitor-status"
        value={view.snapshot.settings.scanMonitorId ?? ''}
        onchange={(event) => changeMonitor(event.currentTarget.value)}
        disabled={saving || view.monitors.length === 0}
      >
        <option value="">Automatic — primary display</option>
        {#if view.snapshot.settings.scanMonitorId !== null && !view.configuredMonitorAvailable}
          <option value={view.snapshot.settings.scanMonitorId}>Saved display unavailable</option>
        {/if}
        {#each view.monitors as monitor (monitor.id)}
          <option value={monitor.id}>{monitor.label}</option>
        {/each}
      </select>
      <button class="secondary" type="button" onclick={() => void refresh()} disabled={saving}>
        Refresh displays
      </button>
      <p
        class:warning={!view.configuredMonitorAvailable || view.monitorError !== null}
        class="registration"
        id="monitor-status"
        role={!view.configuredMonitorAvailable || view.monitorError !== null ? 'status' : undefined}
      >
        {#if view.monitorError !== null}
          {view.monitorError}
        {:else if !view.configuredMonitorAvailable}
          The saved display is disconnected. Scans fall back to the primary display.
        {:else}
          {view.monitors.length} display{view.monitors.length === 1 ? '' : 's'} available.
        {/if}
      </p>
    </section>

    <section class="card options" aria-label="QRForge preferences">
      <label>
        <span
          ><strong>Launch at sign-in</strong><small>Start directly in the system tray.</small></span
        >
        <input
          type="checkbox"
          checked={view.snapshot.settings.launchAtStartup}
          onchange={(event) => change('launchAtStartup', event.currentTarget.checked)}
          disabled={saving}
        />
      </label>
      <label>
        <span
          ><strong>Open safe links automatically</strong><small
            >Only one Rust-approved HTTP or HTTPS result.</small
          ></span
        >
        <input
          type="checkbox"
          checked={view.snapshot.settings.autoOpenSafeUrls}
          onchange={(event) => change('autoOpenSafeUrls', event.currentTarget.checked)}
          disabled={saving}
        />
      </label>
      <label>
        <span
          ><strong>Copy non-link text</strong><small
            >Blocked or malformed content is copied only as inert text.</small
          ></span
        >
        <input
          type="checkbox"
          checked={view.snapshot.settings.copyNonUrlPayloads}
          onchange={(event) => change('copyNonUrlPayloads', event.currentTarget.checked)}
          disabled={saving}
        />
      </label>
      <label>
        <span
          ><strong>Notifications</strong><small>Quiet scan-result and safety feedback.</small></span
        >
        <input
          type="checkbox"
          checked={view.snapshot.settings.notificationsEnabled}
          onchange={(event) => change('notificationsEnabled', event.currentTarget.checked)}
          disabled={saving}
        />
      </label>
    </section>

    <section class="privacy">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M12 2 4.5 5v5.8c0 4.7 3.1 9 7.5 10.7 4.4-1.7 7.5-6 7.5-10.7V5L12 2Zm0 3.1 4.5 1.8v3.9c0 3.1-1.8 6.1-4.5 7.6-2.7-1.5-4.5-4.5-4.5-7.6V6.9L12 5.1Z"
        />
      </svg>
      <div>
        <strong>Your screen stays on this computer.</strong>
        <p>Captures exist only in memory for the scan. QRForge never uploads or saves them.</p>
      </div>
    </section>

    <div class="status-area" aria-live="polite">
      {#if errorMessage}
        <p class="notice error" role="alert">{errorMessage}</p>
      {:else if savedMessage}
        <p class="notice success" role="status">{savedMessage}</p>
      {/if}
    </div>

    <footer>
      <span>Version {view.version}</span>
      <span>{view.build}</span>
    </footer>
  {/if}
</main>
