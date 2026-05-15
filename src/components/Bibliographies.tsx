import { createSignal, For, onMount, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { useApp } from "../context/AppContext.tsx";
import {
  loadBibliography,
  loadSettings,
  openFile,
  removeBibliography,
} from "../tauri.ts";
import {
  ADD_ICON,
  DELETE_ICON,
  DETAILS_ICON,
  ERROR_ICON,
  TRANSPARENT_LOGO,
} from "../constants/icons.ts";

interface BibliographiesProps {
  onOpenModal: () => void;
}

function Bibliographies(props: BibliographiesProps) {
  const { settings, updateSettings, setCurrentReferences, setCurrentBibName } =
    useApp();
  const navigate = useNavigate();
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const [isFadingOut, setIsFadingOut] = createSignal(false);
  const [progress, setProgress] = createSignal(100);

  onMount(async () => {
    try {
      const loadedSettings = await loadSettings();
      updateSettings(loadedSettings);
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  });

  const bibliographyList = () =>
    Object.entries(settings().bibliographies).map(
      ([name, info]) => ({
        name,
        ...info,
      }),
    );

  const handleSelect = async (name: string, path: string) => {
    try {
      const refs = await loadBibliography(path);
      setCurrentReferences(refs);
      setCurrentBibName(name);
      navigate("/detail");
    } catch (e) {
      setErrorMessage(`加载失败: ${e}`);
      startErrorTimer();
    }
  };

  const handleDelete = async (
    name: string,
    event: MouseEvent,
  ) => {
    event.stopPropagation();
    try {
      await removeBibliography(name);
      const loadedSettings = await loadSettings();
      updateSettings(loadedSettings);
    } catch (e) {
      setErrorMessage(`删除失败: ${e}`);
      startErrorTimer();
    }
  };

  const handleOpenFile = async (path: string, event: MouseEvent) => {
    event.stopPropagation();
    try {
      await openFile(path);
    } catch (e) {
      setErrorMessage(`打开文件失败: ${e}`);
      startErrorTimer();
    }
  };

  const startErrorTimer = () => {
    setProgress(100);
    const timer = setInterval(() => {
      setProgress((prev) => {
        const newProgress = prev - 1;
        if (newProgress <= 0) {
          clearInterval(timer);
          setIsFadingOut(true);
          setTimeout(() => {
            setErrorMessage(null);
            setIsFadingOut(false);
            setProgress(100);
          }, 300);
          return 0;
        }
        return newProgress;
      });
    }, 20);
  };

  const abbrPath = (path: string, maxLen: number = 35): string => {
    if (path.length <= maxLen) return path;
    const parts = path.split("/");
    if (parts.length <= 2) return "..." + path.slice(-maxLen + 3);
    return parts[0] + "/.../" + parts.slice(-2).join("/");
  };

  const formatDate = (dateStr: string): string => {
    try {
      return new Date(dateStr).toLocaleString("zh-CN", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    } catch {
      return dateStr;
    }
  };

  return (
    <div class="relative container mx-auto p-6">
      <div class="flex items-center justify-between mb-8">
        <div>
          <h2 class="text-3xl font-bold gradient-text">文献库</h2>
          <p class="text-base-content/60 text-sm mt-1">管理你的文献库</p>
        </div>
        <div class="flex gap-2">
          <button
            type="button"
            class="btn btn-modern gap-2"
            onClick={props.onOpenModal}
          >
            <img src={ADD_ICON} alt="Add" class="h-4 w-4" />
            新建文献库
          </button>
        </div>
      </div>

      {/* Bibliography Grid */}
      <div class="w-full">
        <Show
          when={bibliographyList().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-64 text-base-content/50">
              <img
                src={TRANSPARENT_LOGO}
                alt="No bibliographies"
                class="w-16 h-16 mb-4 opacity-50"
              />
              <p class="text-lg">未找到文献库</p>
              <p class="text-sm">点击 + 按钮添加一个文献库</p>
            </div>
          }
        >
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 p-4">
            <For each={bibliographyList()}>
              {(bib) => {
                const isExist = true;
                return (
                  <div
                    class="card-modern card-shine group relative overflow-hidden flex flex-col h-full min-h-50 transition-all duration-500 hover:-translate-y-2 hover:shadow-primary/10 border-white/5 cursor-pointer"
                    onClick={() => handleSelect(bib.name, bib.path)}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) =>
                      e.key === "Enter" && handleSelect(bib.name, bib.path)}
                  >
                    {/* Decorative Background Elements */}
                    <div class="absolute -top-20 -right-20 w-40 h-40 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-all duration-700 animate-blob">
                    </div>
                    <div class="absolute -bottom-20 -left-20 w-40 h-40 bg-secondary/5 rounded-full blur-3xl group-hover:bg-secondary/10 transition-all duration-700 animate-blob animation-delay-2000">
                    </div>
                    <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-full h-full bg-linear-to-br from-white/0 via-white/5 to-white/0 opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none">
                    </div>

                    <div class="card-body p-6 flex-1 relative z-10 backdrop-blur-[2px]">
                      {/* Header */}
                      <div class="flex items-start justify-between mb-4">
                        <div class="flex items-center gap-3 overflow-hidden">
                          <div class="w-10 h-10 rounded-xl bg-linear-to-br from-primary/10 to-secondary/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-500 shadow-inner border border-white/10">
                            <img
                              src={TRANSPARENT_LOGO}
                              alt="Bibliography"
                              class="w-6 h-6"
                            />
                          </div>
                          <div>
                            <h3
                              class="text-xl font-bold gradient-text truncate leading-tight"
                              title={bib.name}
                            >
                              {bib.name}
                            </h3>
                            <Show
                              when={isExist}
                              fallback={
                                <div class="flex items-center gap-1 mt-1">
                                  <div class="w-1.5 h-1.5 rounded-full bg-error animate-pulse">
                                  </div>
                                  <span class="text-[10px] uppercase tracking-wider font-bold text-error/80">
                                    缺失
                                  </span>
                                </div>
                              }
                            >
                              <div class="flex items-center gap-1 mt-1">
                                <div class="w-1.5 h-1.5 rounded-full bg-success animate-pulse">
                                </div>
                                <span class="text-[10px] uppercase tracking-wider font-bold text-success/80">
                                  可用
                                </span>
                              </div>
                            </Show>
                          </div>
                        </div>
                      </div>

                      {/* Content */}
                      <div class="flex-1 pl-1">
                        <Show
                          when={bib.description}
                          fallback={
                            <p class="text-base-content/30 text-sm mb-6 italic font-light">
                              暂无描述
                            </p>
                          }
                        >
                          <p
                            class="text-base-content/70 text-sm mb-6 line-clamp-2 font-light leading-relaxed"
                            title={bib.description}
                          >
                            {bib.description}
                          </p>
                        </Show>

                        <div class="flex flex-col gap-3 text-xs text-base-content/60">
                          <div class="flex items-center gap-2 group/link">
                            <img
                              src={DETAILS_ICON}
                              alt="文件"
                              class="w-3 h-3 opacity-50 group-hover/link:opacity-100 transition-opacity"
                            />
                            <button
                              type="button"
                              class="link link-hover truncate hover:text-primary transition-colors font-mono bg-base-200/50 px-2 py-1 rounded-md w-full text-left border border-transparent hover:border-primary/20 hover:bg-primary/5"
                              onClick={(e) => handleOpenFile(bib.path, e)}
                              title={bib.path}
                            >
                              {abbrPath(bib.path)}
                            </button>
                          </div>
                          <div class="flex items-center gap-2">
                            <span class="text-xs opacity-50">更新于</span>
                            <span class="font-mono opacity-80">
                              {formatDate(bib.updated_at)}
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* Actions (visible on hover) */}
                      <div class="flex justify-end gap-2 mt-4 opacity-0 group-hover:opacity-100 translate-y-2 group-hover:translate-y-0 transition-all duration-300">
                        <button
                          type="button"
                          class="btn btn-sm btn-circle btn-ghost text-error hover:bg-error/10 tooltip tooltip-left shadow-sm border border-transparent hover:border-error/20"
                          data-tip="删除"
                          aria-label="删除"
                          onClick={(e) => handleDelete(bib.name, e)}
                        >
                          <img
                            src={DELETE_ICON}
                            alt="Delete"
                            class="h-3.5 w-3.5 opacity-70"
                          />
                        </button>
                        <button
                          type="button"
                          class="btn btn-sm btn-primary shadow-lg shadow-primary/30 hover:shadow-primary/50 border-none bg-linear-to-r from-primary to-secondary text-white gap-2 px-4 rounded-full"
                          onClick={() => handleSelect(bib.name, bib.path)}
                        >
                          <span>打开</span>
                          <span class="group-hover:translate-x-1 transition-transform text-lg">
                            Open
                          </span>
                        </button>
                      </div>
                    </div>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>
      </div>

      {/* Error Alert */}
      <Show when={errorMessage()}>
        <div
          class={`absolute top-2 right-2 w-1/3 z-50 ${
            isFadingOut() ? "animate-fade-out" : "animate-fade-in"
          }`}
        >
          <div
            role="alert"
            class="alert alert-error shadow-lg backdrop-blur-md bg-error/10 border-error/20 flex justify-between items-center"
          >
            <div class="flex items-center gap-2">
              <img
                src={ERROR_ICON}
                alt="错误"
                class="h-5 w-5"
              />
              <span class="font-medium">{errorMessage()}</span>
            </div>
            <div
              class="radial-progress text-error text-xs"
              style={{
                "--value": progress(),
                "--size": "1.2rem",
                "--thickness": "2px",
              }}
              role="progressbar"
              aria-valuenow={progress()}
            >
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

export default Bibliographies;
