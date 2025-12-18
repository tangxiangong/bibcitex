<script lang="ts">
  import { drawerOpen, drawerReference, closeDrawer } from '$lib/stores/state';
  import ReferenceDrawer from './reference/ReferenceDrawer.svelte';
</script>

<div class="drawer drawer-end">
  <input
    id="reference-drawer"
    type="checkbox"
    class="drawer-toggle"
    checked={$drawerOpen}
    onchange={() => closeDrawer()}
  />

  <div class="drawer-side z-50">
    <label for="reference-drawer" aria-label="close sidebar" class="drawer-overlay"></label>
    <div class="menu bg-base-100 text-base-content min-h-full w-96 p-4 shadow-xl">
      <div class="flex justify-between items-center mb-4">
        <h3 class="text-lg font-bold">文献详情</h3>
        <button class="btn btn-sm btn-circle btn-ghost" onclick={() => closeDrawer()}>
          ✕
        </button>
      </div>

      {#if $drawerReference}
        <ReferenceDrawer entry={$drawerReference} />
      {/if}
    </div>
  </div>
</div>
