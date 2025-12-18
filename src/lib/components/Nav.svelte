<script lang="ts">
  import { goto } from '$app/navigation';
  import { drawerOpen, drawerReference } from '$lib/stores/state';
  import { openHelperWindow } from '$lib/tauri';
  import ChunksComp from './ChunksComp.svelte';
  import ReferenceDrawer from './reference/ReferenceDrawer.svelte';

  const CMD_CTRL = navigator.platform.includes('Mac') ? '⌘' : 'Win';

  async function openSpotlightWindow() {
    try {
      await openHelperWindow();
    } catch (e) {
      console.error('Failed to open helper window:', e);
    }
  }

  function closeDrawer() {
    drawerOpen.set(false);
  }

  $: drawerTitle = $drawerReference?.title || [];
  $: drawerKey = $drawerReference?.cite_key || '';
</script>

<div class="h-screen flex flex-col bg-base-100">
  <!-- Navbar -->
  <div class="navbar bg-base-100/80 backdrop-blur-md border-b border-base-content/5 shrink-0 z-40 sticky top-0">
    <div class="navbar-start pl-4">
      <a
        href="/"
        class="flex items-center gap-3 hover:opacity-80 transition-opacity"
        onclick={(e) => { e.preventDefault(); goto('/'); }}
      >
        <div class="w-10 h-10 relative">
          <img
            src="/favicon.png"
            alt="BibCiTeX Logo"
            class="w-full h-full object-contain"
          />
        </div>
        <span class="font-bold text-xl tracking-tight gradient-text hidden sm:block">
          BibCiTeX
        </span>
      </a>
    </div>

    <div class="navbar-center"></div>

    <div class="navbar-end pr-4">
      <button
        class="btn btn-ghost btn-sm gap-2 hover:bg-base-content/5 font-normal text-base-content/70"
        onclick={openSpotlightWindow}
      >
        <span>快捷助手</span>
        <div class="hidden md:flex gap-1">
          <kbd class="kbd kbd-sm font-mono bg-base-200 border-base-300">{CMD_CTRL}</kbd>
          <kbd class="kbd kbd-sm font-mono bg-base-200 border-base-300">shift</kbd>
          <kbd class="kbd kbd-sm font-mono bg-base-200 border-base-300">k</kbd>
        </div>
      </button>
    </div>
  </div>

  <!-- Drawer Layout -->
  <div class="drawer drawer-end flex-1 overflow-hidden relative">
    <input
      id="global-drawer"
      type="checkbox"
      class="drawer-toggle"
      checked={$drawerOpen}
      onchange={() => {}}
    />

    <!-- Main Content -->
    <div class="drawer-content h-full overflow-hidden bg-base-200/30">
      <slot />
    </div>

    <!-- Drawer Side Panel -->
    <div class="drawer-side z-50">
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
      <label
        class="drawer-overlay backdrop-blur-sm"
        for="global-drawer"
        onclick={closeDrawer}
      ></label>
      <div class="min-h-full w-120 max-w-[90vw] bg-base-100 shadow-2xl p-0 flex flex-col border-l border-base-content/5">
        <!-- Drawer Header -->
        <div class="p-4 border-b border-base-content/5 flex justify-between items-start bg-base-100/95 backdrop-blur sticky top-0 z-10">
          <div class="flex-1 pr-4">
            <h3 class="text-lg font-bold leading-tight">
              <ChunksComp chunks={drawerTitle} citeKey={drawerKey} />
            </h3>
          </div>
          <button
            class="btn btn-sm btn-circle btn-ghost"
            onclick={closeDrawer}
          >
            ✕
          </button>
        </div>

        <!-- Drawer Content -->
        <div class="flex-1 overflow-y-auto p-4">
          {#if $drawerReference}
            <ReferenceDrawer entry={$drawerReference} />
          {:else}
            <div class="flex flex-col items-center justify-center h-full text-base-content/50">
              <span class="text-4xl mb-2">📄</span>
              <span>尚未选择任何参考文献</span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>
