import { Show } from "solid-js";
import ChunksComp from "../ChunksComp.tsx";
import { useApp } from "../../context/AppContext.tsx";
import { copyToClipboard, openFile, openUrl } from "../../tauri.ts";
import IconButton from "../ui/IconButton.tsx";
import SvgIcon from "../ui/SvgIcon.tsx";
import {
  getChunkText,
  getReferenceTitleText,
  getReferenceTypeKey,
  getReferenceVenue,
  TYPE_STYLES,
} from "../reference/semantic.ts";
import { MetadataChip, MetadataRow } from "./ReferenceMetadata.tsx";

export default function ReferenceDetailPane() {
  const { selectedReference } = useApp();
  const reference = () => selectedReference();
  const typeStyle = () =>
    reference() ? TYPE_STYLES[getReferenceTypeKey(reference()!.type_)] : null;
  const doiUrl = () => {
    const doi = reference()?.doi;
    if (!doi) return null;
    return doi.startsWith("http") ? doi : `https://doi.org/${doi}`;
  };
  const venue = () => (reference() ? getReferenceVenue(reference()!) : "");
  const pagesText = () => {
    const selected = reference();
    if (!selected) return undefined;
    if (selected.pages) {
      return `${selected.pages.start}-${selected.pages.end}`;
    }
    return selected.book_pages;
  };

  const handleCopyCiteKey = async () => {
    const citeKey = reference()?.cite_key;
    if (citeKey) await copyToClipboard(citeKey);
  };

  return (
    <aside class="flex h-full min-w-80 max-w-[26rem] basis-96 flex-col border-l border-base-300 bg-base-100">
      <div class="flex h-14 shrink-0 items-center justify-between border-b border-base-300 px-4">
        <div class="flex min-w-0 items-center gap-2">
          <SvgIcon name="info" class="h-4 w-4 shrink-0 text-base-content/70" aria-hidden />
          <h2 class="truncate text-sm font-semibold">文献详情</h2>
        </div>
      </div>

      <Show
        when={reference()}
        fallback={
          <div class="flex min-h-0 flex-1 items-center justify-center px-6 text-center text-sm text-base-content/55">
            选择一条文献查看字段和操作
          </div>
        }
      >
        {(selected) => (
          <div class="min-h-0 flex-1 overflow-y-auto">
            <div class="border-b border-base-300 p-4">
              <div class="mb-3 flex items-center gap-2">
                <span class={typeStyle()?.badgeClass}>
                  <SvgIcon name={typeStyle()!.icon} class="h-3.5 w-3.5" aria-hidden />
                  {typeStyle()?.label}
                </span>
                <MetadataChip kind="citeKey" text={selected().cite_key} />
              </div>
              <h3 class="text-base font-semibold leading-6">
                <Show
                  when={selected().title?.length}
                  fallback={<span>{getReferenceTitleText(selected())}</span>}
                >
                  <ChunksComp
                    chunks={selected().title ?? []}
                    citeKey={selected().cite_key}
                  />
                </Show>
              </h3>
              <div class="mt-4 flex flex-wrap gap-2">
                <IconButton
                  icon="copy"
                  label="复制引用键"
                  size="sm"
                  variant="primary"
                  onClick={handleCopyCiteKey}
                />
                <Show when={doiUrl()}>
                  {(url) => (
                    <IconButton
                      icon="link"
                      label="打开 DOI"
                      size="sm"
                      onClick={() => openUrl(url())}
                    />
                  )}
                </Show>
                <Show when={selected().url}>
                  {(url) => (
                    <IconButton
                      icon="externalLink"
                      label="打开 URL"
                      size="sm"
                      onClick={() => openUrl(url())}
                    />
                  )}
                </Show>
                <Show when={selected().file}>
                  {(file) => (
                    <IconButton
                      icon="folderOpen"
                      label="打开文件"
                      size="sm"
                      onClick={() => openFile(file())}
                    />
                  )}
                </Show>
              </div>
            </div>

            <section>
              <MetadataRow kind="author" label="作者" value={selected().author?.join(", ")} />
              <MetadataRow kind="year" label="年份" value={selected().year} />
              <MetadataRow kind="venue" label="来源" value={venue()} />
              <MetadataRow kind="note" label="出版方" value={selected().publisher?.join(", ")} />
              <MetadataRow kind="note" label="页码" value={pagesText()} />
              <MetadataRow kind="link" label="DOI" value={selected().doi} />
              <MetadataRow kind="link" label="URL" value={selected().url} />
              <MetadataRow kind="note" label="文件" value={selected().file} />
              <Show when={selected().abstract_?.length}>
                <MetadataRow kind="note" label="摘要">
                  <div class="leading-6 text-base-content/80">
                    <ChunksComp
                      chunks={selected().abstract_ ?? []}
                      citeKey={selected().cite_key}
                    />
                  </div>
                </MetadataRow>
              </Show>
              <MetadataRow kind="note" label="备注" value={getChunkText(selected().note)} />
            </section>
          </div>
        )}
      </Show>
    </aside>
  );
}
