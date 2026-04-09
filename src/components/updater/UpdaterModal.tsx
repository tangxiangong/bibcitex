import { createEffect, createSignal, Show, Switch, Match } from "solid-js";
import { checkUpdate, installUpdate } from "../../tauri.ts";

interface UpdateInfo {
  available: boolean;
  version?: string;
  notes?: string;
}

enum UpdaterStatus {
  Checking = "checking",
  Available = "available",
  Downloading = "downloading",
  UpToDate = "upToDate",
  Failed = "failed",
}

interface UpdaterModalProps {
  isOpen: boolean;
  onClose: () => void;
}

function UpdaterModal(props: UpdaterModalProps) {
  const [status, setStatus] = createSignal<UpdaterStatus>(UpdaterStatus.Checking);
  const [updateInfo, setUpdateInfo] = createSignal<UpdateInfo | null>(null);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const [downloadProgress, setDownloadProgress] = createSignal(0);

  const currentVersion = "0.6.0";

  createEffect(() => {
    if (props.isOpen) {
      handleCheckUpdate();
    }
  });

  const handleCheckUpdate = async () => {
    setStatus(UpdaterStatus.Checking);
    setErrorMessage(null);

    try {
      const result = await checkUpdate();
      setUpdateInfo(result);

      if (result.available) {
        setStatus(UpdaterStatus.Available);
      } else {
        setStatus(UpdaterStatus.UpToDate);
      }
    } catch (e) {
      setStatus(UpdaterStatus.Failed);
      setErrorMessage(`检查更新失败: ${e}`);
    }
  };

  const handleDownload = async () => {
    setStatus(UpdaterStatus.Downloading);
    setDownloadProgress(0);

    try {
      const progressInterval = setInterval(() => {
        setDownloadProgress((prev) => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return prev;
          }
          return prev + 10;
        });
      }, 500);

      await installUpdate();
      clearInterval(progressInterval);
      setDownloadProgress(100);

      setTimeout(() => {
        props.onClose();
      }, 2000);
    } catch (e) {
      setStatus(UpdaterStatus.Failed);
      setErrorMessage(`下载失败: ${e}`);
    }
  };

  const handleRetry = () => {
    handleCheckUpdate();
  };

  return (
    <Show when={props.isOpen}>
      <div class="modal modal-open">
        <div class="modal-box max-w-md">
          <Switch>
            {/* Checking Status */}
            <Match when={status() === UpdaterStatus.Checking}>
              <div class="flex flex-col items-center justify-center py-4">
                <img
                  src="/transparent_logo.png"
                  alt="Logo"
                  class="w-20 h-20 mb-4 drop-shadow-lg"
                />
                <h3 class="text-xl font-bold gradient-text animate-pulse">
                  正在检查更新...
                </h3>
                <progress class="progress progress-primary w-full h-2 mt-6" />
                <button
                  type="button"
                  class="btn btn-ghost text-base-content/70 hover:text-error mt-4"
                  onClick={props.onClose}
                >
                  取消
                </button>
              </div>
            </Match>

            {/* Available Update */}
            <Match when={status() === UpdaterStatus.Available && updateInfo()}>
              <div class="flex flex-col items-center">
                <img
                  src="/transparent_logo.png"
                  alt="Logo"
                  class="w-20 h-20 mb-4 drop-shadow-lg"
                />
                <h3 class="text-2xl font-bold gradient-text mb-2">
                  发现新版本!
                </h3>
                <div class="badge badge-primary badge-soft">
                  v{updateInfo()!.version}
                </div>

                <div class="bg-base-200/50 rounded-lg p-4 my-6 text-sm space-y-2 w-full">
                  <div class="flex justify-between">
                    <span class="text-base-content/60">当前版本</span>
                    <span class="font-mono">v{currentVersion}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-base-content/60">新版本</span>
                    <span class="font-mono">v{updateInfo()!.version}</span>
                  </div>
                </div>

                <Show when={updateInfo()!.notes}>
                  <div class="bg-base-200/30 rounded-lg p-4 mb-4 text-sm w-full max-h-32 overflow-y-auto">
                    <p class="text-base-content/70 whitespace-pre-wrap">
                      {updateInfo()!.notes}
                    </p>
                  </div>
                </Show>

                <div class="flex justify-end gap-3 w-full mt-4">
                  <button
                    type="button"
                    class="btn btn-ghost text-base-content/70 hover:bg-base-200"
                    onClick={props.onClose}
                  >
                    稍后
                  </button>
                  <button
                    type="button"
                    class="btn btn-primary btn-soft shadow-lg hover:scale-105 transition-transform"
                    onClick={handleDownload}
                  >
                    立即下载
                  </button>
                </div>
              </div>
            </Match>

            {/* Downloading */}
            <Match when={status() === UpdaterStatus.Downloading}>
              <div class="flex flex-col items-center">
                <img
                  src="/transparent_logo.png"
                  alt="Logo"
                  class="w-20 h-20 mb-4 drop-shadow-lg"
                />
                <h3 class="text-xl font-bold gradient-text">
                  {downloadProgress() < 100
                    ? `正在下载更新... ${downloadProgress()}%`
                    : "下载完成"}
                </h3>

                <div class="w-full space-y-2 my-6">
                  <progress
                    class="progress progress-primary w-full h-3"
                    value={downloadProgress()}
                    max="100"
                  />
                  <div class="flex justify-between text-xs text-base-content/50 px-1">
                    <span>{downloadProgress()}%</span>
                  </div>

                  <Show when={downloadProgress() === 100}>
                    <div class="flex flex-col items-center gap-4 py-4">
                      <div class="text-success text-5xl mb-2 animate-bounce">
                        ✓
                      </div>
                      <p class="text-lg font-medium">下载完成</p>
                      <p class="text-sm text-base-content/60">
                        准备就绪，请重启应用以完成更新
                      </p>
                    </div>
                  </Show>
                </div>

                <Show when={downloadProgress() < 100}>
                  <button
                    type="button"
                    class="btn btn-ghost text-base-content/70 hover:text-error"
                    onClick={props.onClose}
                  >
                    取消下载
                  </button>
                </Show>
              </div>
            </Match>

            {/* Up to Date */}
            <Match when={status() === UpdaterStatus.UpToDate}>
              <div class="flex flex-col items-center text-center py-4">
                <img
                  src="/transparent_logo.png"
                  alt="Logo"
                  class="w-20 h-20 mb-4 drop-shadow-lg"
                />
                <h3 class="text-2xl font-bold gradient-text mb-2">
                  已是最新版本
                </h3>
                <p class="text-base-content/60 mb-8">
                  BibCiTeX v{currentVersion} 目前是最新的
                </p>
                <button
                  type="button"
                  class="btn btn-primary btn-soft w-full"
                  onClick={props.onClose}
                >
                  太棒了
                </button>
              </div>
            </Match>

            {/* Failed */}
            <Match when={status() === UpdaterStatus.Failed}>
              <div class="flex flex-col items-center text-center border-t-4 border-error rounded-t-lg">
                <img
                  src="/transparent_logo.png"
                  alt="Logo"
                  class="w-20 h-20 mt-4 mb-4 grayscale opacity-50"
                />
                <h3 class="text-xl font-bold text-error mb-4">检查更新失败</h3>
                <div class="bg-error/10 text-error p-4 rounded-lg mb-8 text-sm text-left w-full">
                  {errorMessage() || "未知错误"}
                </div>

                <div class="flex justify-end gap-3 w-full">
                  <button
                    type="button"
                    class="btn btn-ghost hover:bg-base-200"
                    onClick={props.onClose}
                  >
                    关闭
                  </button>
                  <button
                    type="button"
                    class="btn btn-error btn-soft shadow-lg"
                    onClick={handleRetry}
                  >
                    重试
                  </button>
                </div>
              </div>
            </Match>
          </Switch>
        </div>
        <div class="modal-backdrop" onClick={props.onClose} />
      </div>
    </Show>
  );
}

export default UpdaterModal;
