import { createMemo, createSignal, Show } from "solid-js";
import { useApp } from "../context/AppContext.tsx";
import { addBibliography, loadSettings, selectBibFile } from "../tauri.ts";
import { SvgIcon } from "./ui/SvgIcon";

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
        <div class="modal-box w-[min(42rem,92vw)] max-w-2xl rounded-box border border-base-300 bg-base-100 shadow-xl animate-fade-in">
          <div>
            {/* Header */}
            <div class="mb-6">
              <h3 class="text-xl font-semibold text-base-content">新增文献库</h3>
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
                        <SvgIcon name="x" class="text-error" size={20} aria-hidden />
                      }
                    >
                      <SvgIcon name="check" class="text-success" size={20} aria-hidden />
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
                  class="btn btn-primary rounded-field gap-2"
                  onClick={handleSelectFile}
                >
                  <SvgIcon name="folderOpen" size={16} aria-hidden />
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
                  <SvgIcon name="alert" size={16} class="shrink-0" aria-hidden />
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
                class="btn btn-primary rounded-field disabled:opacity-40 disabled:cursor-not-allowed"
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
          aria-label="关闭弹窗"
          onKeyDown={(e) => e.key === "Escape" && handleClose()}
        />
      </div>
    </Show>
  );
}

export default AddBibliography;
