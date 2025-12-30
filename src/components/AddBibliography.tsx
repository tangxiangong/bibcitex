import React, { useCallback, useMemo, useState } from "react";
import { useApp } from "../context/AppContext.tsx";
import { addBibliography, loadSettings, selectBibFile } from "../tauri.ts";

interface AddBibliographyProps {
  show: boolean;
  onClose: () => void;
}

function AddBibliography({ show, onClose }: AddBibliographyProps) {
  const { settings, updateSettings } = useApp();
  const [name, setName] = useState("");
  const [path, setPath] = useState<string | null>(null);
  const [description, setDescription] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const existNames = useMemo(
    () => Object.keys(settings.bibliographies),
    [settings.bibliographies],
  );

  const nameIsValid = useMemo(
    () => name.trim() !== "" && !existNames.includes(name),
    [name, existNames],
  );

  const saveAvailable = useMemo(
    () => path !== null && nameIsValid,
    [path, nameIsValid],
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
    if (!path || !nameIsValid) return;

    try {
      await addBibliography(name, path, description || undefined);
      const loadedSettings = await loadSettings();
      updateSettings(loadedSettings);
      handleClose();
    } catch (e) {
      setErrorMessage(`保存失败: ${e}`);
    }
  };

  const handleClose = useCallback(() => {
    onClose();
    setName("");
    setPath(null);
    setDescription("");
    setErrorMessage(null);
  }, [onClose]);

  const abbrPath = (p: string, maxLen: number = 40): string => {
    if (p.length <= maxLen) return p;
    const parts = p.split("/");
    if (parts.length <= 2) return "..." + p.slice(-maxLen + 3);
    return parts[0] + "/.../" + parts.slice(-2).join("/");
  };

  if (!show) return null;

  return (
    <div className="modal modal-open backdrop-blur-sm">
      <div className="modal-box w-1/2 max-w-2xl glass-panel shadow-2xl">
        <h3 className="text-2xl font-bold mb-6 gradient-text">新增文献库</h3>

        {/* Name Input */}
        <div className="form-control w-full mb-4">
          <label className="label" htmlFor="bib-name">
            <span className="label-text font-medium">文献库名称</span>
          </label>
          <label className="input input-bordered flex items-center gap-2 focus-within:input-primary transition-colors">
            <input
              id="bib-name"
              className="grow"
              type="text"
              placeholder="输入名称"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
            {name.trim() !== "" && (
              nameIsValid
                ? (
                  <img
                    src="/assets/icons/ok.svg"
                    alt="Valid"
                    className="h-5 w-5 text-success"
                  />
                )
                : (
                  <img
                    src="/assets/icons/cancel.svg"
                    alt="Invalid"
                    className="h-5 w-5 text-error"
                  />
                )
            )}
          </label>
          {name.trim() !== "" && !nameIsValid && (
            <div className="label">
              <span className="label-text-alt text-error">名称已存在</span>
            </div>
          )}
        </div>

        {/* File Path */}
        <div className="form-control w-full mb-4">
          <label className="label" htmlFor="bib-path">
            <span className="label-text font-medium">文件路径</span>
          </label>
          <div className="join w-full">
            <input
              id="bib-path"
              className="input input-bordered join-item grow focus:outline-none cursor-default bg-base-200/50"
              type="text"
              readOnly
              value={path ? abbrPath(path) : "请选择 .bib 文件"}
              title={path || ""}
            />
            <button
              type="button"
              className="btn btn-primary join-item"
              onClick={handleSelectFile}
            >
              选择文件
            </button>
          </div>
        </div>

        {/* Description */}
        <div className="form-control w-full mb-6">
          <label className="label" htmlFor="bib-desc">
            <span className="label-text font-medium">描述（可选）</span>
          </label>
          <textarea
            id="bib-desc"
            className="textarea textarea-bordered"
            placeholder="添加描述..."
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>

        {/* Error Message */}
        {errorMessage && (
          <div className="alert alert-error mb-4">
            <img
              src="/assets/icons/error.svg"
              alt="Error"
              className="h-5 w-5"
            />
            <span>{errorMessage}</span>
          </div>
        )}

        {/* Actions */}
        <div className="modal-action">
          <button type="button" className="btn btn-ghost" onClick={handleClose}>
            取消
          </button>
          <button
            type="button"
            className="btn btn-primary"
            disabled={!saveAvailable}
            onClick={handleSave}
          >
            保存
          </button>
        </div>
      </div>
      <div
        className="modal-backdrop"
        onClick={handleClose}
        role="button"
        tabIndex={-1}
        aria-label="Close modal"
        onKeyDown={(e) => e.key === "Escape" && handleClose()}
      />
    </div>
  );
}

export default AddBibliography;
