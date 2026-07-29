<script lang="ts">
  import ResultsPanel from './ResultsPanel.svelte';
  import SettingsPanel from './SettingsPanel.svelte';

  const surface = new URLSearchParams(window.location.search).get('surface');
</script>

{#if surface === 'results'}
  <ResultsPanel />
{:else}
  <SettingsPanel />
{/if}
