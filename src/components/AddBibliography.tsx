import { createMemo, createSignal, Show } from "solid-js";
import { useApp } from "../context/AppContext.tsx";
import { addBibliography, loadSettings, selectBibFile } from "../tauri.ts";

interface AddBibliographyProps {
  show: boolean;
  onClose: () => void;
}

function AddBibliography(props: AddBibliographyProps) {
  const { settings, updateSettings } = useApp();
  const [name, setName] = createSignal("");
  const [path, setPath] = createSignal<string | null>(null);
  const [description, setDescription] = createSignal("");
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);

  const existNames = createMemo(() =>
    Object.keys(settings().bibliographies)
  );

  const nameIsValid = createMemo(() =>
    name().trim() !== "" && !existNames().includes(name())
  );

  const saveAvailable = createMemo(() =>
    path() !== null && nameIsValid()
  );

  const handleSelectFile = async () => {
    try {
      const selected = await selectBibFile();
      if (selected) {
        setPath(selected);
        setErrorMessage(null);
      }
    } catch (e) {
      setErrorMessage(`选择文件失败: ${e}`);
    }
  };

  const handleSave = async () => {
    if (!path() || !nameIsValid()) return;

    try {
      await addBibliography(name(), path()!, description() || undefined);
      const loadedSettings = await loadSettings();
      updateSettings(loadedSettings);
      handleClose();
    } catch (e) {
      setErrorMessage(`保存失败: ${e}`);
    }
  };

  const handleClose = () => {
    props.onClose();
    setName("");
    setPath(null);
    setDescription("");
    setErrorMessage(null);
  };

  const abbrPath = (p: string, maxLen: number = 40): string => {
    if (p.length <= maxLen) return p;
    const parts = p.split("/");
    if (parts.length <= 2) return "..." + p.slice(-maxLen + 3);
    return parts[0] + "/.../" + parts.slice(-2).join("/");
  };

  return (
    <Show when={props.show}>
      <div class="modal modal-open backdrop-blur-sm">
        <div class="modal-box w-1/2 max-w-2xl glass-panel rounded-3xl shadow-2xl border border-white/10 relative overflow-hidden animate-fade-in">
          {/* Decorative blobs */}
          <div class="absolute -top-24 -right-24 w-48 h-48 bg-primary/10 rounded-full blur-3xl animate-blob pointer-events-none" />
          <div class="absolute -bottom-24 -left-24 w-48 h-48 bg-secondary/10 rounded-full blur-3xl animate-blob animation-delay-2000 pointer-events-none" />

          {/* Content */}
          <div class="relative z-10">
            {/* Header */}
            <div class="mb-8">
              <h3 class="text-2xl font-bold gradient-text">新增文献库</h3>
              <p class="text-base-content/50 text-sm mt-1">添加一个 .bib 文件到你的工作空间</p>
            </div>

            {/* Name Input */}
            <div class="form-control w-full mb-5">
              <label class="label pb-1" for="bib-name">
                <span class="text-sm font-medium text-base-content/80">文献库名称</span>
              </label>
              <div class="flex gap-2">
                <input
                  id="bib-name"
                  class="input input-bordered flex-1 rounded-xl bg-base-200/30 border-base-content/10 focus:border-primary/50 focus:bg-base-100/50 transition-all duration-300 pr-10"
                  type="text"
                  placeholder="为文献库起一个名字"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                />
                {/* Placeholder to match the "选择文件" button width */}
                <div class="flex items-center justify-center shrink-0" style={{ width: "7.25rem" }}>
                  <Show when={name().trim() !== ""}>
                    <Show
                      when={nameIsValid()}
                      fallback={
                        <svg class="w-5 h-5 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      }
                    >
                      <svg class="w-5 h-5 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                      </svg>
                    </Show>
                  </Show>
                </div>
              </div>
              <Show when={name().trim() !== "" && !nameIsValid()}>
                <p class="text-error text-xs mt-1.5 ml-1">该名称已存在，请换一个</p>
              </Show>
            </div>

            {/* File Path */}
            <div class="form-control w-full mb-5">
              <label class="label pb-1" for="bib-path">
                <span class="text-sm font-medium text-base-content/80">文件路径</span>
              </label>
              <div class="flex gap-2">
                <input
                  id="bib-path"
                  class="input input-bordered flex-1 rounded-xl bg-base-200/30 border-base-content/10 cursor-default font-mono text-sm"
                  type="text"
                  readOnly
                  value={path() ? abbrPath(path()!) : ""}
                  placeholder="尚未选择文件"
                  title={path() || ""}
                />
                <button
                  type="button"
                  class="btn rounded-xl border-none shadow-md hover:shadow-lg text-white bg-linear-to-r from-primary to-secondary hover:-translate-y-0.5 transition-all duration-300"
                  onClick={handleSelectFile}
                >
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  选择文件
                </button>
              </div>
            </div>

            {/* Description */}
            <div class="form-control w-full mb-6">
              <label class="label pb-1" for="bib-desc">
                <span class="text-sm font-medium text-base-content/80">描述</span>
                <span class="text-xs text-base-content/40">可选</span>
              </label>
              <div class="flex gap-2">
                <textarea
                  id="bib-desc"
                  class="textarea textarea-bordered flex-1 rounded-xl bg-base-200/30 border-base-content/10 focus:border-primary/50 focus:bg-base-100/50 transition-all duration-300 min-h-20"
                  placeholder="简单描述一下这个文献库..."
                  value={description()}
                  onInput={(e) => setDescription(e.currentTarget.value)}
                />
                <div class="shrink-0" style={{ width: "7.25rem" }} />
              </div>
            </div>

            {/* Error Message */}
            <Show when={errorMessage()}>
              <div class="mb-4 animate-fade-in">
                <div class="flex items-center gap-2 px-4 py-3 rounded-xl bg-error/10 border border-error/20 text-sm text-error">
                  <svg class="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                  <span>{errorMessage()}</span>
                </div>
              </div>
            </Show>

            {/* Actions */}
            <div class="flex justify-end gap-3 pt-2">
              <button
                type="button"
                class="btn btn-ghost rounded-xl hover:bg-base-content/5"
                onClick={handleClose}
              >
                取消
              </button>
              <button
                type="button"
                class="btn rounded-xl border-none shadow-lg text-white bg-linear-to-r from-primary to-secondary hover:shadow-primary/30 hover:-translate-y-0.5 transition-all duration-300 disabled:opacity-40 disabled:shadow-none disabled:translate-y-0 disabled:cursor-not-allowed"
                disabled={!saveAvailable()}
                onClick={handleSave}
              >
                保存
              </button>
            </div>
          </div>
        </div>
        <div
          class="modal-backdrop"
          onClick={handleClose}
          role="button"
          tabIndex={-1}
          aria-label="Close modal"
          onKeyDown={(e) => e.key === "Escape" && handleClose()}
        />
      </div>
    </Show>
  );
}

export default AddBibliography;
