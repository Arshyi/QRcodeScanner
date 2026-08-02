<script lang="ts">
  import { onMount } from 'svelte';
  import {
    commandMessage,
    getPendingResults,
    performResultAction,
    type PendingResultsView,
    type ResultAction,
  } from './lib/api';
  import { actionForDialogKey, resultKindLabel } from './lib/result-dialog';

  let results: PendingResultsView | null = null;
  let loading = true;
  let acting = false;
  let statusMessage = '';
  let errorMessage = '';
  let dismissButton: HTMLButtonElement | null = null;

  onMount(() => {
    const reload = (): void => {
      void load();
    };
    window.addEventListener('focus', reload);
    void load().then(() => dismissButton?.focus({ preventScroll: true }));
    return () => window.removeEventListener('focus', reload);
  });

  async function load(): Promise<void> {
    loading = true;
    errorMessage = '';
    try {
      results = await getPendingResults();
    } catch (error) {
      errorMessage = commandMessage(error);
    } finally {
      loading = false;
    }
  }

  async function act(action: ResultAction, index?: number): Promise<void> {
    if (acting || results === null) return;
    acting = true;
    errorMessage = '';
    statusMessage = '';
    try {
      const outcome = await performResultAction({
        sessionId: results.sessionId,
        action,
        index: index ?? null,
      });
      statusMessage = outcome.message;
    } catch (error) {
      errorMessage = commandMessage(error);
    } finally {
      acting = false;
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (actionForDialogKey(event.key) === 'dismiss') {
      event.preventDefault();
      void act('dismiss');
    }
  }
</script>

<svelte:head>
  <title>QRForge Scan Results</title>
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<main class="results-main">
  <dialog
    open
    class="results-dialog"
    aria-labelledby="results-heading"
    aria-describedby="results-summary"
  >
    <header>
      <div class="mark" aria-hidden="true"><span></span><span></span><span></span></div>
      <div>
        <p class="eyebrow">QRForge</p>
        <h1 id="results-heading">Choose a scan result</h1>
      </div>
      {#if results !== null}
        <span class="local-pill">{results.items.length} found</span>
      {/if}
    </header>

    <p id="results-summary" class="chooser-summary">
      Nothing opens automatically when multiple codes are detected. Rust validates every action.
    </p>

    {#if loading}
      <p class="state" role="status">Loading scan results…</p>
    {:else if results === null}
      <p class="notice error" role="alert">{errorMessage}</p>
    {:else}
      <div class="dialog-actions">
        <button
          class="secondary"
          type="button"
          onclick={() => void act('copy_all')}
          disabled={acting || !results.items.some((item) => item.canCopy)}
        >
          Copy all copyable results
        </button>
        <button
          class="secondary"
          type="button"
          bind:this={dismissButton}
          onclick={() => void act('dismiss')}
          disabled={acting}
        >
          Dismiss
        </button>
      </div>

      <ol class="result-list">
        {#each results.items as item (item.index)}
          <li>
            <article class="result-card">
              <div class="result-heading">
                <strong>Result {item.index + 1}</strong>
                <span class:blocked={!item.canOpen && item.kind !== 'plain_text'} class="kind">
                  {resultKindLabel(item.kind)}
                </span>
              </div>
              {#if item.detail !== null}
                <p class="result-detail">{item.detail}</p>
              {/if}
              <pre class="payload-preview">{item.preview}</pre>
              <div class="result-actions">
                {#if item.canOpen}
                  <button
                    class="primary"
                    type="button"
                    onclick={() => void act('open', item.index)}
                    disabled={acting}
                  >
                    Open approved link
                  </button>
                {/if}
                <button
                  class="secondary"
                  type="button"
                  onclick={() => void act('copy', item.index)}
                  disabled={acting || !item.canCopy}
                >
                  {item.canCopy ? 'Copy result' : 'Not copyable'}
                </button>
              </div>
            </article>
          </li>
        {/each}
      </ol>
    {/if}

    <div class="status-area" aria-live="polite">
      {#if errorMessage && results !== null}
        <p class="notice error" role="alert">{errorMessage}</p>
      {:else if statusMessage}
        <p class="notice success" role="status">{statusMessage}</p>
      {/if}
    </div>
    <p class="keyboard-hint">Tab moves between actions. Enter activates. Escape dismisses.</p>
  </dialog>
</main>
