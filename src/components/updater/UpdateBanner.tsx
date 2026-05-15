import { createSignal, onMount, onCleanup, Show, Switch, Match } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import type { Update, DownloadEvent } from "@tauri-apps/plugin-updater";
import { SvgIcon } from "../ui/SvgIcon";

type BannerStatus = "hidden" | "checking" | "available" | "downloading" | "ready" | "error" | "uptodate";

function UpdateBanner() {
  const [status, setStatus] = createSignal<BannerStatus>("hidden");
  const [update, setUpdate] = createSignal<Update | null>(null);
  const [progress, setProgress] = createSignal(0);
  const [errorMsg, setErrorMsg] = createSignal("");
  const [dismissing, setDismissing] = createSignal(false);

  let contentLength = 0;
  let downloadedBytes = 0;

  const dismiss = () => {
    setDismissing(true);
    setTimeout(() => {
      setStatus("hidden");
      setDismissing(false);
    }, 300);
  };

  const handleCheck = async (silent: boolean) => {
    if (!silent) setStatus("checking");

    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const result = await check();
      if (result) {
        setUpdate(result);
        setStatus("available");
      } else {
        if (!silent) {
          setStatus("uptodate");
          setTimeout(() => dismiss(), 3000);
        }
      }
    } catch (e) {
      if (!silent) {
        const msg = String(e);
        if (msg.includes("fetch") || msg.includes("remote") || msg.includes("JSON")) {
          setErrorMsg("无法连接到更新服务器，请检查网络后重试");
        } else {
          setErrorMsg(`检查更新失败: ${msg}`);
        }
        setStatus("error");
      }
    }
  };

  const handleDownload = async () => {
    const u = update();
    if (!u) return;

    setStatus("downloading");
    setProgress(0);
    contentLength = 0;
    downloadedBytes = 0;

    try {
      await u.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === "Started") {
          contentLength = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloadedBytes += event.data.chunkLength;
          if (contentLength > 0) {
            setProgress(Math.min(Math.round((downloadedBytes / contentLength) * 100), 100));
          }
        } else if (event.event === "Finished") {
          setProgress(100);
          setStatus("ready");
        }
      });
      setStatus("ready");
    } catch (e) {
      setErrorMsg(`下载更新失败: ${e}`);
      setStatus("error");
    }
  };

  const handleRelaunch = async () => {
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  };

  onMount(() => {
    const unlisten = listen("check-update-trigger", () => {
      handleCheck(false);
    });
    onCleanup(() => {
      unlisten.then((f) => f());
    });

    // Silent check on startup — delay to avoid blocking initial render
    const timer = setTimeout(() => handleCheck(true), 3000);
    onCleanup(() => clearTimeout(timer));
  });

  return (
    <Show when={status() !== "hidden"}>
      <div
        class={`${dismissing() ? "animate-fade-out" : "animate-fade-in"}`}
      >
        <Switch>
          {/* Checking */}
          <Match when={status() === "checking"}>
            <div class="flex items-center gap-3 px-4 py-2 bg-info/10 border-b border-info/20 text-sm">
              <span class="loading loading-spinner loading-xs text-info" />
              <span class="text-info font-medium">正在检查更新...</span>
              <div class="flex-1" />
              <button
                type="button"
                class="btn btn-ghost btn-xs text-base-content/50"
                onClick={dismiss}
              >
                取消
              </button>
            </div>
          </Match>

          {/* Available */}
          <Match when={status() === "available" && update()}>
            <div class="flex items-center gap-3 px-4 py-2 bg-primary/10 border-b border-primary/20 text-sm">
              <SvgIcon name="download" size={16} class="text-primary shrink-0" aria-hidden />
              <span class="text-primary font-medium">
                发现新版本
              </span>
              <span class="badge badge-primary badge-sm">
                v{update()!.version}
              </span>
              <Show when={update()!.body}>
                <span class="text-base-content/50 truncate max-w-xs hidden md:inline">
                  {update()!.body}
                </span>
              </Show>
              <div class="flex-1" />
              <button
                type="button"
                class="btn btn-primary btn-soft btn-xs"
                onClick={handleDownload}
              >
                立即更新
              </button>
              <button
                type="button"
                class="btn btn-ghost btn-xs text-base-content/50"
                onClick={dismiss}
              >
                稍后
              </button>
            </div>
          </Match>

          {/* Downloading */}
          <Match when={status() === "downloading"}>
            <div class="flex items-center gap-3 px-4 py-2 bg-info/10 border-b border-info/20 text-sm">
              <span class="loading loading-spinner loading-xs text-info" />
              <span class="text-info font-medium">
                正在下载更新 {progress()}%
              </span>
              <div class="flex-1">
                <progress
                  class="progress progress-info w-full h-1.5"
                  value={progress()}
                  max="100"
                />
              </div>
            </div>
          </Match>

          {/* Ready to restart */}
          <Match when={status() === "ready"}>
            <div class="flex items-center gap-3 px-4 py-2 bg-success/10 border-b border-success/20 text-sm">
              <SvgIcon name="check" size={16} class="text-success shrink-0" aria-hidden />
              <span class="text-success font-medium">
                更新已就绪，重启以完成更新
              </span>
              <div class="flex-1" />
              <button
                type="button"
                class="btn btn-success btn-soft btn-xs"
                onClick={handleRelaunch}
              >
                立即重启
              </button>
              <button
                type="button"
                class="btn btn-ghost btn-xs text-base-content/50"
                onClick={dismiss}
              >
                稍后
              </button>
            </div>
          </Match>

          {/* Up to date */}
          <Match when={status() === "uptodate"}>
            <div class="flex items-center gap-3 px-4 py-2 bg-success/10 border-b border-success/20 text-sm">
              <SvgIcon name="check" size={16} class="text-success shrink-0" aria-hidden />
              <span class="text-success font-medium">已是最新版本</span>
              <div class="flex-1" />
              <button
                type="button"
                class="btn btn-ghost btn-xs text-base-content/50"
                onClick={dismiss}
              >
                关闭
              </button>
            </div>
          </Match>

          {/* Error */}
          <Match when={status() === "error"}>
            <div class="flex items-center gap-3 px-4 py-2 bg-error/10 border-b border-error/20 text-sm">
              <SvgIcon name="alert" size={16} class="text-error shrink-0" aria-hidden />
              <span class="text-error font-medium truncate">
                {errorMsg()}
              </span>
              <div class="flex-1" />
              <button
                type="button"
                class="btn btn-error btn-soft btn-xs"
                onClick={() => handleCheck(false)}
              >
                重试
              </button>
              <button
                type="button"
                class="btn btn-ghost btn-xs text-base-content/50"
                onClick={dismiss}
              >
                关闭
              </button>
            </div>
          </Match>
        </Switch>
      </div>
    </Show>
  );
}

export default UpdateBanner;
