import type { JSX } from "nsh";
import { windowSize } from "nsh";
import { ResizablePanel, ResizablePanelGroup } from "nsh/base";
import { workbench } from "../theme/metrics";
import { Surface } from "./Surface";

/**
 * Where the auxiliary panes go at the current window width: docked beside
 * the list, or (below 900 / 1280pt) behind a toolbar popover, as the macOS
 * workbench does. Reactive in the window size.
 */
export function paneLayout(sidebarShown: boolean, inspectorShown: boolean) {
  const width = windowSize().width;
  return {
    sidebarDocked: sidebarShown && width >= workbench.compactSidebarBelow,
    inspectorDocked: inspectorShown && width >= workbench.compactInspectorBelow,
    sidebarCompact: width < workbench.compactSidebarBelow,
    inspectorCompact: width < workbench.compactInspectorBelow,
  };
}

export interface SplitPanesProps {
  /** The user wants the sidebar (toolbar toggle); it docks only when wide enough. */
  sidebarShown: boolean;
  inspectorShown: boolean;
  /** The widths the user dragged to (220 / 350 at first). */
  sidebarWidth?: number;
  inspectorWidth?: number;
  onSidebarResize?: (width: number) => void;
  onInspectorResize?: (width: number) => void;
  sidebar: JSX.Element;
  content: JSX.Element;
  inspector: JSX.Element;
}

/**
 * The workbench's three resizable columns: sidebar 180–300 (220), list at
 * least 330, inspector 280–520 (350). Side panes keep their width as the
 * window grows; the list takes the difference.
 */
export function SplitPanes(props: SplitPanesProps) {
  const layout = () => paneLayout(props.sidebarShown, props.inspectorShown);
  return (
    <ResizablePanelGroup
      class="flex-1"
      handleClass="w-px bg-separator"
    >
      <ResizablePanel
        fixed
        visible={layout().sidebarDocked}
        size={props.sidebarWidth ?? workbench.sidebar.ideal}
        minSize={workbench.sidebar.min}
        maxSize={workbench.sidebar.max}
        onResize={({ size }) => props.onSidebarResize?.(size)}
      >
        <Surface variant="sidebar" class="flex-1">
          {props.sidebar}
        </Surface>
      </ResizablePanel>
      <ResizablePanel minSize={workbench.list.min}>
        <Surface variant="pane" class="flex-1">
          {props.content}
        </Surface>
      </ResizablePanel>
      <ResizablePanel
        fixed
        visible={layout().inspectorDocked}
        size={props.inspectorWidth ?? workbench.inspector.ideal}
        minSize={workbench.inspector.min}
        maxSize={workbench.inspector.max}
        onResize={({ size }) => props.onInspectorResize?.(size)}
      >
        <Surface variant="sidebar" class="flex-1">
          {props.inspector}
        </Surface>
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}
