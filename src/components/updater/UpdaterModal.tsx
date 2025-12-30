import React, { useEffect, useState } from "react";
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

function UpdaterModal({ isOpen, onClose }: UpdaterModalProps) {
  const [status, setStatus] = useState<UpdaterStatus>(UpdaterStatus.Checking);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);

  const currentVersion = "0.6.0"; // Should be imported from package.json or env

  useEffect(() => {
    if (isOpen) {
      handleCheckUpdate();
    }
  }, [isOpen]);

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
      // Simulate progress updates
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

      // Show success message
      setTimeout(() => {
        onClose();
      }, 2000);
    } catch (e) {
      setStatus(UpdaterStatus.Failed);
      setErrorMessage(`下载失败: ${e}`);
    }
  };

  const handleRetry = () => {
    handleCheckUpdate();
  };

  if (!isOpen) return null;

  return (
    <div className="modal modal-open">
      <div className="modal-box max-w-md">
        {/* Checking Status */}
        {status === UpdaterStatus.Checking && (
          <div className="flex flex-col items-center justify-center py-4">
            <img
              src="/transparent_logo.png"
              alt="Logo"
              className="w-20 h-20 mb-4 drop-shadow-lg"
            />
            <h3 className="text-xl font-bold gradient-text animate-pulse">
              正在检查更新...
            </h3>
            <progress className="progress progress-primary w-full h-2 mt-6" />
            <button
              type="button"
              className="btn btn-ghost text-base-content/70 hover:text-error mt-4"
              onClick={onClose}
            >
              取消
            </button>
          </div>
        )}

        {/* Available Update */}
        {status === UpdaterStatus.Available && updateInfo && (
          <div className="flex flex-col items-center">
            <img
              src="/transparent_logo.png"
              alt="Logo"
              className="w-20 h-20 mb-4 drop-shadow-lg"
            />
            <h3 className="text-2xl font-bold gradient-text mb-2">
              发现新版本!
            </h3>
            <div className="badge badge-primary badge-soft">
              v{updateInfo.version}
            </div>

            <div className="bg-base-200/50 rounded-lg p-4 my-6 text-sm space-y-2 w-full">
              <div className="flex justify-between">
                <span className="text-base-content/60">当前版本</span>
                <span className="font-mono">v{currentVersion}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-base-content/60">新版本</span>
                <span className="font-mono">v{updateInfo.version}</span>
              </div>
            </div>

            {updateInfo.notes && (
              <div className="bg-base-200/30 rounded-lg p-4 mb-4 text-sm w-full max-h-32 overflow-y-auto">
                <p className="text-base-content/70 whitespace-pre-wrap">
                  {updateInfo.notes}
                </p>
              </div>
            )}

            <div className="flex justify-end gap-3 w-full mt-4">
              <button
                type="button"
                className="btn btn-ghost text-base-content/70 hover:bg-base-200"
                onClick={onClose}
              >
                稍后
              </button>
              <button
                type="button"
                className="btn btn-primary btn-soft shadow-lg hover:scale-105 transition-transform"
                onClick={handleDownload}
              >
                立即下载
              </button>
            </div>
          </div>
        )}

        {/* Downloading */}
        {status === UpdaterStatus.Downloading && (
          <div className="flex flex-col items-center">
            <img
              src="/transparent_logo.png"
              alt="Logo"
              className="w-20 h-20 mb-4 drop-shadow-lg"
            />
            <h3 className="text-xl font-bold gradient-text">
              {downloadProgress < 100
                ? `正在下载更新... ${downloadProgress}%`
                : "下载完成"}
            </h3>

            <div className="w-full space-y-2 my-6">
              <progress
                className="progress progress-primary w-full h-3"
                value={downloadProgress}
                max="100"
              />
              <div className="flex justify-between text-xs text-base-content/50 px-1">
                <span>{downloadProgress}%</span>
              </div>

              {downloadProgress === 100 && (
                <div className="flex flex-col items-center gap-4 py-4">
                  <div className="text-success text-5xl mb-2 animate-bounce">
                    ✓
                  </div>
                  <p className="text-lg font-medium">下载完成</p>
                  <p className="text-sm text-base-content/60">
                    准备就绪，请重启应用以完成更新
                  </p>
                </div>
              )}
            </div>

            {downloadProgress < 100 && (
              <button
                type="button"
                className="btn btn-ghost text-base-content/70 hover:text-error"
                onClick={onClose}
              >
                取消下载
              </button>
            )}
          </div>
        )}

        {/* Up to Date */}
        {status === UpdaterStatus.UpToDate && (
          <div className="flex flex-col items-center text-center py-4">
            <img
              src="/transparent_logo.png"
              alt="Logo"
              className="w-20 h-20 mb-4 drop-shadow-lg"
            />
            <h3 className="text-2xl font-bold gradient-text mb-2">
              已是最新版本
            </h3>
            <p className="text-base-content/60 mb-8">
              BibCiTeX v{currentVersion} 目前是最新的
            </p>
            <button
              type="button"
              className="btn btn-primary btn-soft w-full"
              onClick={onClose}
            >
              太棒了
            </button>
          </div>
        )}

        {/* Failed */}
        {status === UpdaterStatus.Failed && (
          <div className="flex flex-col items-center text-center border-t-4 border-error rounded-t-lg">
            <img
              src="/transparent_logo.png"
              alt="Logo"
              className="w-20 h-20 mt-4 mb-4 grayscale opacity-50"
            />
            <h3 className="text-xl font-bold text-error mb-4">检查更新失败</h3>
            <div className="bg-error/10 text-error p-4 rounded-lg mb-8 text-sm text-left w-full">
              {errorMessage || "未知错误"}
            </div>

            <div className="flex justify-end gap-3 w-full">
              <button
                type="button"
                className="btn btn-ghost hover:bg-base-200"
                onClick={onClose}
              >
                关闭
              </button>
              <button
                type="button"
                className="btn btn-error btn-soft shadow-lg"
                onClick={handleRetry}
              >
                重试
              </button>
            </div>
          </div>
        )}
      </div>
      <div className="modal-backdrop" onClick={onClose} />
    </div>
  );
}

export default UpdaterModal;
