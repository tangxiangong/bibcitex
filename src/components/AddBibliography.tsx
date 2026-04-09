import { createMemo, createSignal, Show } from "solid-js";
import { useApp } from "../context/AppContext.tsx";
import { addBibliography, loadSettings, selectBibFile } from "../tauri.ts";
import { CANCEL_ICON, ERROR_ICON, OK_ICON } from "../constants/icons.ts";

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
        <div class="modal-box w-1/2 max-w-2xl glass-panel shadow-2xl">
          <h3 class="text-2xl font-bold mb-6 gradient-text">新增文献库</h3>

          {/* Name Input */}
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
                value={name()}
                onInput={(e) => setName(e.currentTarget.value)}
              />
              <Show when={name().trim() !== ""}>
                <Show
                  when={nameIsValid()}
                  fallback={
                    <img
                      src={CANCEL_ICON}
                      alt="Invalid"
                      class="h-5 w-5 text-error"
                    />
                  }
                >
                  <img
                    src={OK_ICON}
                    alt="Valid"
                    class="h-5 w-5 text-success"
                  />
                </Show>
              </Show>
            </label>
            <Show when={name().trim() !== "" && !nameIsValid()}>
              <div class="label">
                <span class="label-text-alt text-error">名称已存在</span>
              </div>
            </Show>
          </div>

          {/* File Path */}
          <div class="form-control w-full mb-4">
            <label class="label" for="bib-path">
              <span class="label-text font-medium">文件路径</span>
            </label>
            <div class="join w-full">
              <input
                id="bib-path"
                class="input input-bordered join-item grow focus:outline-none cursor-default bg-base-200/50"
                type="text"
                readOnly
                value={path() ? abbrPath(path()!) : "请选择 .bib 文件"}
                title={path() || ""}
              />
              <button
                type="button"
                class="btn btn-primary join-item"
                onClick={handleSelectFile}
              >
                选择文件
              </button>
            </div>
          </div>

          {/* Description */}
          <div class="form-control w-full mb-6">
            <label class="label" for="bib-desc">
              <span class="label-text font-medium">描述（可选）</span>
            </label>
            <textarea
              id="bib-desc"
              class="textarea textarea-bordered"
              placeholder="添加描述..."
              value={description()}
              onInput={(e) => setDescription(e.currentTarget.value)}
            />
          </div>

          {/* Error Message */}
          <Show when={errorMessage()}>
            <div class="alert alert-error mb-4">
              <img
                src={ERROR_ICON}
                alt="Error"
                class="h-5 w-5"
              />
              <span>{errorMessage()}</span>
            </div>
          </Show>

          {/* Actions */}
          <div class="modal-action">
            <button type="button" class="btn btn-ghost" onClick={handleClose}>
              取消
            </button>
            <button
              type="button"
              class="btn btn-primary"
              disabled={!saveAvailable()}
              onClick={handleSave}
            >
              保存
            </button>
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
