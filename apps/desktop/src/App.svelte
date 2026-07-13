<script lang="ts">
  import { onMount } from 'svelte';
  import {
    commandMessage,
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

  onMount(async () => {
    try {
      view = await getSettings();
    } catch (error) {
      errorMessage = commandMessage(error);
    } finally {
      loading = false;
    }
  });

  async function save(settings: AppSettings): Promise<void> {
    if (saving) return;
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
      });
      savedMessage = 'Saved';
    } catch (error) {
      errorMessage = commandMessage(error);
    } finally {
      saving = false;
    }
  }

  function change(field: keyof AppSettings, value: boolean): void {
    if (view === null) return;
    void save({ ...view.snapshot.settings, [field]: value });
  }

  function captureHotkey(event: KeyboardEvent): void {
    event.preventDefault();
    if (!capturingHotkey || view === null) return;
    if (event.code === 'Escape') {
      capturingHotkey = false;
      return;
    }
    const hotkey = hotkeyFromKeyboard(event);
    if (hotkey === null) {
      errorMessage = 'Hold at least one modifier and press a letter, digit, or F-key.';
      return;
    }
    capturingHotkey = false;
    void save({ ...view.snapshot.settings, hotkey });
  }
</script>

<svelte:head>
  <title>QRForge Settings</title>
</svelte:head>

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
    <p class="state">Loading settings…</p>
  {:else if view === null}
    <section class="notice error" role="alert">{errorMessage}</section>
  {:else}
    <section class="card hotkey-card">
      <div>
        <p class="label">Scan shortcut</p>
        <p class="hint">Works globally while QRForge is running.</p>
      </div>
      <button
        class:capturing={capturingHotkey}
        class="hotkey"
        type="button"
        onkeydown={captureHotkey}
        onclick={() => {
          capturingHotkey = true;
          errorMessage = '';
        }}
        disabled={saving}
      >
        {capturingHotkey ? 'Press shortcut…' : view.snapshot.settings.hotkey}
      </button>
      <p class:warning={!view.snapshot.hotkeyRegistered} class="registration">
        {view.snapshot.hotkeyRegistered
          ? `Active: ${view.snapshot.activeHotkey}`
          : 'Not registered — choose an available shortcut'}
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
            >Only a single valid HTTP or HTTPS result.</small
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
            >Blocked schemes may be copied, but are never opened.</small
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

    {#if errorMessage}
      <p class="notice error" role="alert">{errorMessage}</p>
    {:else if savedMessage}
      <p class="notice success" role="status">{savedMessage}</p>
    {/if}

    <footer>
      <span>Version {view.version}</span>
      <span>{view.build}</span>
    </footer>
  {/if}
</main>
