x<script lang="ts">
  import { settings } from '$lib/stores/state';
  import { addBibliography, loadSettings, selectBibFile } from '$lib/tauri';

  interface Props {
    show: boolean;
  }

  let { show = $bindable() }: Props = $props();

  let name = $state('');
  let path = $state<string | null>(null);
  let description = $state('');
  let errorMessage = $state<string | null>(null);

  const existNames = $derived(Object.keys($settings.bibliographies));
  const nameIsValid = $derived(name.trim() !== '' && !existNames.includes(name));
  const saveAvailable = $derived(path !== null && nameIsValid);

  async function handleSelectFile() {
    try {
      const selected = await selectBibFile();
      if (selected) {
        path = selected;
        errorMessage = null;
      }
    } catch (e) {
      errorMessage = `选择文件失败: ${e}`;
    }
  }

  async function handleSave() {
    if (!path || !nameIsValid) return;

    try {
      await addBibliography(name, path, description || undefined);
      const loadedSettings = await loadSettings();
      settings.set(loadedSettings);
      closeModal();
    } catch (e) {
      errorMessage = `保存失败: ${e}`;
    }
  }

  function closeModal() {
    show = false;
    name = '';
    path = null;
    description = '';
    errorMessage = null;
  }

  function abbrPath(p: string, maxLen: number = 40): string {
    if (p.length <= maxLen) return p;
    const parts = p.split('/');
    if (parts.length <= 2) return '...' + p.slice(-maxLen + 3);
    return parts[0] + '/.../' + parts.slice(-2).join('/');
  }
</script>

{#if show}
  <div class="modal modal-open backdrop-blur-sm">
    <div class="modal-box w-1/2 max-w-2xl glass-panel shadow-2xl">
      <h3 class="text-2xl font-bold mb-6 gradient-text">新增文献库</h3>

      <!-- Name Input -->
      <div class="form-control w-full mb-4">
        <label class="label" for="bib-name">
          <span class="label-text font-medium">文献库名称</span>
        </label>
        <label class="input input-bordered flex items-center gap-2 focus-within:input-primary transition-colors">
          <input
            id="bib-name"
            class="grow"
            type="text"
            placeholder="输入名称"
            bind:value={name}
          />
          {#if name.trim() !== ''}
            {#if nameIsValid}
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            {:else}
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            {/if}
          {/if}
        </label>
        {#if name.trim() !== '' && !nameIsValid}
          <div class="label">
            <span class="label-text-alt text-error">名称已存在</span>
          </div>
        {/if}
      </div>

      <!-- File Path -->
      <div class="form-control w-full mb-4">
        <label class="label" for="bib-path">
          <span class="label-text font-medium">文件路径</span>
        </label>
        <div class="join w-full">
          <input
            id="bib-path"
            class="input input-bordered join-item grow focus:outline-none cursor-default bg-base-200/50"
            type="text"
            readonly
            value={path ? abbrPath(path) : '请选择 .bib 文件'}
            title={path || ''}
          />
          <button class="btn btn-primary join-item" onclick={handleSelectFile}>
            选择文件
          </button>
        </div>
      </div>

      <!-- Description -->
      <div class="form-control w-full mb-6">
        <label class="label" for="bib-desc">
          <span class="label-text font-medium">描述（可选）</span>
        </label>
        <textarea
          id="bib-desc"
          class="textarea textarea-bordered"
          placeholder="添加描述..."
          bind:value={description}
        ></textarea>
      </div>

      <!-- Error Message -->
      {#if errorMessage}
        <div class="alert alert-error mb-4">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{errorMessage}</span>
        </div>
      {/if}

      <!-- Actions -->
      <div class="modal-action">
        <button class="btn btn-ghost" onclick={closeModal}>取消</button>
        <button
          class="btn btn-primary"
          disabled={!saveAvailable}
          onclick={handleSave}
        >
          保存
        </button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={closeModal} role="button" tabindex="-1" aria-label="Close modal" onkeydown={(e) => e.key === 'Escape' && closeModal()}></div>
  </div>
{/if}
