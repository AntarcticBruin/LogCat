<script setup lang="ts">
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { ref, computed } from "vue";
import { nextTick, onBeforeUnmount, onMounted, watch } from "vue";
import type { HighlightedLine, HighlightSegment, HostSessionTab } from "../../types/app";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

const content = defineModel<string>("content", { required: true });
const activeSessionId = defineModel<string | null>("activeSessionId", { required: true });
const terminalTabs = defineModel<import("../../types/app").TerminalTab[]>("terminalTabs", { required: true });
const activeTerminalTabId = defineModel<string | null>("activeTerminalTabId", { required: true });
const isAutoScroll = defineModel<boolean>("isAutoScroll", { required: true });
const logViewerRef = defineModel<HTMLElement | null>("logViewerRef", { required: true });

const props = defineProps<{
  sessionId: string | null;
  hostSessionTabs: HostSessionTab[];
  selectedFile: string | null;
  tailToken: string | null;
  highlightedLines: HighlightedLine[];
}>();

const emit = defineEmits<{
  (event: "clear"): void;
  (event: "stop"): void;
  (event: "start"): void;
  (event: "write-terminal", tabId: string, data: string): void;
  (event: "resize-terminal", tabId: string, cols: number, rows: number): void;
  (event: "start-terminal"): void;
  (event: "stop-terminal", tabId: string): void;
  (event: "disconnect-session", sessionId: string): void;
}>();

type TerminalInstance = {
  terminal: Terminal;
  fitAddon: FitAddon;
  disposeDataHandler: { dispose(): void } | null;
  lastRenderedLength: number;
};

const terminalInstances = new Map<string, TerminalInstance>();
let resizeObserver: ResizeObserver | null = null;
const localLogViewerRef = ref<HTMLElement | null>(null);
const terminalContainers = new Map<string, HTMLElement>();
const observedTerminalElements = new Set<HTMLElement>();

const terminalContextMenu = ref<{
  tabId: string;
  type: "copy" | "paste";
  selection: string;
  x: number;
  y: number;
} | null>(null);

const searchQuery = ref("");
const filterQuery = ref("");
const showSearch = ref(false);
const showFilter = ref(false);
const searchInputRef = ref<HTMLInputElement | null>(null);
const filterInputRef = ref<HTMLInputElement | null>(null);

const displayLines = computed(() => {
  let lines = props.highlightedLines;

  if (filterQuery.value) {
    const lowerFilter = filterQuery.value.toLowerCase();
    lines = lines.filter(line => {
      const fullText = line.segments.map(s => s.text).join("");
      return fullText.toLowerCase().includes(lowerFilter);
    });
  }

  if (searchQuery.value) {
    const lowerSearch = searchQuery.value.toLowerCase();
    lines = lines.map(line => {
      const newSegments: HighlightSegment[] = [];
      for (const seg of line.segments) {
        if (!seg.text.toLowerCase().includes(lowerSearch)) {
          newSegments.push(seg);
          continue;
        }

        let remaining = seg.text;
        while (remaining) {
          const idx = remaining.toLowerCase().indexOf(lowerSearch);
          if (idx === -1) {
            newSegments.push({ text: remaining, tone: seg.tone });
            break;
          }
          if (idx > 0) {
            newSegments.push({ text: remaining.substring(0, idx), tone: seg.tone });
          }
          newSegments.push({
            text: remaining.substring(idx, idx + lowerSearch.length),
            tone: seg.tone,
            isMatch: true
          });
          remaining = remaining.substring(idx + lowerSearch.length);
        }
      }
      return { tone: line.tone, segments: newSegments };
    });
  }

  return lines;
});

function closeSearchOrFilter() {
  showSearch.value = false;
  showFilter.value = false;
  searchQuery.value = "";
  filterQuery.value = "";
  localLogViewerRef.value?.focus();
}

function handleKeydown(e: KeyboardEvent) {
  if (terminalContextMenu.value && e.key === "Escape") {
    e.preventDefault();
    terminalContextMenu.value = null;
    return;
  }
  if (!props.selectedFile) return;

  if (e.ctrlKey && e.key.toLowerCase() === 'f') {
    e.preventDefault();
    if (e.shiftKey) {
      showFilter.value = true;
      showSearch.value = false;
      searchQuery.value = "";
      void nextTick(() => filterInputRef.value?.focus());
    } else {
      showSearch.value = true;
      showFilter.value = false;
      filterQuery.value = "";
      void nextTick(() => searchInputRef.value?.focus());
    }
  } else if (e.key === 'Escape') {
    if (showSearch.value || showFilter.value) {
      e.preventDefault();
      closeSearchOrFilter();
    }
  }
}

const terminalTheme = {
  background: "#1e1e1e",
  foreground: "#d4d4d4",
  cursor: "#79c0ff",
  cursorAccent: "#1e1e1e",
  selectionBackground: "rgba(121, 192, 255, 0.30)",
  selectionForeground: "#ffffff",
  scrollbarSliderBackground: "rgba(255, 255, 255, 0.14)",
  scrollbarSliderHoverBackground: "rgba(255, 255, 255, 0.22)",
  scrollbarSliderActiveBackground: "rgba(255, 255, 255, 0.28)",
  overviewRulerBorder: "transparent",
  black: "#1e1e1e",
  red: "#cd3131",
  green: "#0dbc79",
  yellow: "#e5e510",
  blue: "#2472c8",
  magenta: "#bc3fbc",
  cyan: "#11a8cd",
  white: "#e5e5e5",
  brightBlack: "#666666",
  brightRed: "#f14c4c",
  brightGreen: "#23d18b",
  brightYellow: "#f5f543",
  brightBlue: "#3b8eea",
  brightMagenta: "#d670d6",
  brightCyan: "#29b8db",
  brightWhite: "#e5e5e5",
} as const;

function focusTerminal(tabId: string) {
  if (!props.selectedFile && props.sessionId) {
    const inst = terminalInstances.get(tabId);
    if (inst) inst.terminal.focus();
  }
}

function terminalEmptyState() {
  if (!props.sessionId) {
    return {
      title: "No Active Session",
      text: "Select a host on the left to open a persistent SSH terminal. File previews will appear above it without closing the shell.",
    };
  }

  if (terminalTabs.value.length === 0) {
    return {
      title: "No Terminal Open",
      text: "Click the + button to open a new SSH terminal.",
    };
  }

  return null;
}

function syncTerminalResize(tabId: string) {
  const inst = terminalInstances.get(tabId);
  const tab = terminalTabs.value.find(t => t.id === tabId);
  if (!inst || !tab || !tab.token) return;

  inst.fitAddon.fit();
  emit("resize-terminal", tabId,
    Math.max(20, inst.terminal.cols),
    Math.max(8, inst.terminal.rows)
  );
}

function resetTerminalView(tabId: string) {
  const inst = terminalInstances.get(tabId);
  const tab = terminalTabs.value.find(t => t.id === tabId);
  if (!inst || !tab) return;

  inst.terminal.reset();
  inst.lastRenderedLength = 0;

  if (tab.content) {
    inst.terminal.write(tab.content);
    inst.lastRenderedLength = tab.content.length;
  }
}

function syncTerminalOutput() {
  for (const tab of terminalTabs.value) {
    const inst = terminalInstances.get(tab.id);
    if (!inst) continue;

    if (tab.content.length < inst.lastRenderedLength) {
      resetTerminalView(tab.id);
      continue;
    }

    const chunk = tab.content.slice(inst.lastRenderedLength);
    if (!chunk) continue;

    inst.terminal.write(chunk);
    inst.lastRenderedLength = tab.content.length;
  }
}

const TERMINAL_CONTEXT_MENU_WIDTH = 156;
const TERMINAL_CONTEXT_MENU_ITEM_HEIGHT = 36;
const TERMINAL_CONTEXT_MENU_PADDING = 12;

function positionTerminalContextMenu(x: number, y: number) {
  const menuHeight = TERMINAL_CONTEXT_MENU_ITEM_HEIGHT + TERMINAL_CONTEXT_MENU_PADDING;
  const maxX = window.innerWidth - TERMINAL_CONTEXT_MENU_WIDTH - 8;
  const maxY = window.innerHeight - menuHeight - 8;

  return {
    x: Math.max(8, Math.min(x, maxX)),
    y: Math.max(8, Math.min(y, maxY)),
  };
}

function closeTerminalContextMenu() {
  terminalContextMenu.value = null;
}

function handleGlobalClick() {
  closeTerminalContextMenu();
}

async function handleTerminalCopy() {
  const menu = terminalContextMenu.value;
  if (!menu || menu.type !== "copy") return;
  try {
    await writeText(menu.selection);
    const inst = terminalInstances.get(menu.tabId);
    inst?.terminal.clearSelection();
  } catch (error) {
    console.error("Failed to write clipboard:", error);
  } finally {
    closeTerminalContextMenu();
  }
}

async function handleTerminalPaste() {
  const menu = terminalContextMenu.value;
  if (!menu || menu.type !== "paste") return;
  try {
    const text = await readText();
    if (text && text.length > 0) {
      emit("write-terminal", menu.tabId, text);
      focusTerminal(menu.tabId);
    }
  } catch (error) {
    console.error("Failed to read clipboard:", error);
  } finally {
    closeTerminalContextMenu();
  }
}

async function handleTerminalContextMenu(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();

  if (props.selectedFile || !activeTerminalTabId.value) return;
  const tabId = activeTerminalTabId.value;
  const tab = terminalTabs.value.find(t => t.id === tabId);
  if (!tab || !tab.token) return;

  const inst = terminalInstances.get(tabId);
  const selection = inst?.terminal.getSelection() ?? "";
  const type = selection.trim() ? "copy" : "paste";
  const pos = positionTerminalContextMenu(event.clientX, event.clientY);
  terminalContextMenu.value = {
    tabId,
    type,
    selection,
    ...pos,
  };
}

watch(
  localLogViewerRef,
  (element) => {
    logViewerRef.value = element;
    if (element && props.selectedFile) {
      void nextTick(() => {
        element.scrollTop = element.scrollHeight;
      });
    }
  },
  { flush: "post" },
);

watch(
  () => terminalTabs.value.map(t => t.content),
  () => {
    syncTerminalOutput();
  },
  { flush: "post", deep: true },
);

watch(
  () => props.selectedFile,
  (selectedFile) => {
    if (selectedFile) {
      if (activeTerminalTabId.value) {
        const inst = terminalInstances.get(activeTerminalTabId.value);
        if (inst) inst.terminal.blur();
      }
      void nextTick(() => {
        if (localLogViewerRef.value) {
          localLogViewerRef.value.scrollTop = localLogViewerRef.value.scrollHeight;
        }
      });
      return;
    }

    if (activeTerminalTabId.value) {
      void nextTick(() => focusTerminal(activeTerminalTabId.value!));
    }
  },
);

watch(
  () => {
    const activeTabId = activeTerminalTabId.value;
    const activeTab = activeTabId ? terminalTabs.value.find(t => t.id === activeTabId) : undefined;
    return [props.sessionId, activeTabId, props.selectedFile, activeTab?.token] as const;
  },
  ([sessionId, activeTabId, selectedFile]) => {
    if (!activeTabId) return;
    const inst = terminalInstances.get(activeTabId);
    const tab = terminalTabs.value.find(t => t.id === activeTabId);
    if (!inst || !tab) return;

    const isInteractive = Boolean(sessionId && tab.token && !selectedFile);
    inst.terminal.options.cursorBlink = isInteractive;
    inst.terminal.options.theme = {
      ...terminalTheme,
      cursor: isInteractive ? terminalTheme.cursor : "transparent",
    };

    if (isInteractive) {
      void nextTick(() => focusTerminal(activeTabId));
    } else {
      inst.terminal.blur();
    }
  },
  { flush: "post" },
);

function createTerminal(tabId: string, element: HTMLElement) {
  if (terminalInstances.has(tabId)) return;

  const terminal = new Terminal({
    cursorBlink: false,
    fontFamily: "Consolas, 'Cascadia Mono', 'SFMono-Regular', monospace",
    fontSize: 13,
    lineHeight: 1.2,
    fontWeight: "500",
    fontWeightBold: "800",
    scrollback: 5000,
    convertEol: false,
    allowProposedApi: false,
    minimumContrastRatio: 4.5,
    drawBoldTextInBrightColors: true,
    theme: terminalTheme,
    overviewRuler: {
      width: 0,
    },
  });

  const fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);
  terminal.open(element);

  const disposeDataHandler = terminal.onData((data) => {
    if (!props.selectedFile) {
      emit("write-terminal", tabId, data);
    }
  });

  terminalInstances.set(tabId, {
    terminal,
    fitAddon,
    disposeDataHandler,
    lastRenderedLength: 0,
  });

  const tab = terminalTabs.value.find(t => t.id === tabId);
  const isInteractive = Boolean(props.sessionId && tab?.token && !props.selectedFile);
  terminal.options.cursorBlink = isInteractive;
  terminal.options.theme = {
    ...terminalTheme,
    cursor: isInteractive ? terminalTheme.cursor : "transparent",
  };

  if (!isInteractive) {
    terminal.blur();
  }

  syncTerminalOutput();
  if (activeTerminalTabId.value === tabId) {
    if (isInteractive) {
      void nextTick(() => {
        focusTerminal(tabId);
        requestAnimationFrame(() => focusTerminal(tabId));
      });
    }
    syncTerminalResize(tabId);
  }
}

function destroyTerminal(tabId: string) {
  const inst = terminalInstances.get(tabId);
  if (inst) {
    inst.disposeDataHandler?.dispose();
    inst.terminal.dispose();
    terminalInstances.delete(tabId);
  }
}

function setTerminalContainer(tabId: string, el: unknown) {
  if (el instanceof HTMLElement) {
    terminalContainers.set(tabId, el);
    return;
  }

  terminalContainers.delete(tabId);
}

function syncTerminalContainerMounts(tabs: { id: string }[]) {
  for (const tab of tabs) {
    const el = terminalContainers.get(tab.id);
    if (el && !terminalInstances.has(tab.id)) {
      createTerminal(tab.id, el);
    }
  }
}

function syncResizeObserverTargets(tabs: { id: string }[]) {
  if (!resizeObserver) return;

  for (const el of observedTerminalElements) {
    resizeObserver.unobserve(el);
  }
  observedTerminalElements.clear();

  for (const tab of tabs) {
    const el = terminalContainers.get(tab.id);
    if (el) {
      resizeObserver.observe(el);
      observedTerminalElements.add(el);
    }
  }
}

watch(
  () => terminalTabs.value,
  (newTabs) => {
    const newIds = new Set(newTabs.map(t => t.id));
    for (const id of terminalInstances.keys()) {
      if (!newIds.has(id)) {
        destroyTerminal(id);
      }
    }

    void nextTick(() => {
      syncTerminalContainerMounts(newTabs);
      syncResizeObserverTargets(newTabs);
    });
  },
  { deep: true, immediate: true }
);

watch(
  activeTerminalTabId,
  (newId, oldId) => {
    if (oldId) {
      const oldInst = terminalInstances.get(oldId);
      oldInst?.terminal.blur();
    }
    if (newId) {
      void nextTick(() => {
        syncTerminalResize(newId);
        focusTerminal(newId);
      });
    }
  }
);

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
  window.addEventListener("click", handleGlobalClick);
  window.addEventListener("blur", handleGlobalClick);

  resizeObserver = new ResizeObserver(() => {
    if (activeTerminalTabId.value) {
      syncTerminalResize(activeTerminalTabId.value);
    }
  });

  void nextTick(() => {
    syncTerminalContainerMounts(terminalTabs.value);
    syncResizeObserverTargets(terminalTabs.value);
  });
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown);
  window.removeEventListener("click", handleGlobalClick);
  window.removeEventListener("blur", handleGlobalClick);
  resizeObserver?.disconnect();
  for (const tabId of terminalInstances.keys()) {
    destroyTerminal(tabId);
  }
});
</script>

<template>
  <div class="log-content">
    <div class="toolbar">
      <div class="tab-title">
        <template v-if="selectedFile">
          {{ selectedFile.split("/").pop() }}
        </template>
        <template v-else>
          <div v-if="hostSessionTabs.length > 0" class="host-tabs-inline">
            <div
              v-for="session in hostSessionTabs"
              :key="session.sessionId"
              class="host-tab-inline"
              :class="{ active: activeSessionId === session.sessionId }"
              @click="activeSessionId = session.sessionId"
            >
              <span>{{ session.label }}</span>
              <button class="close-tab-btn" @click.stop="emit('disconnect-session', session.sessionId)">×</button>
            </div>
          </div>
          <span v-else>SSH Terminal</span>
        </template>
      </div>
      <div class="actions">
        <template v-if="selectedFile">
          <div v-if="showSearch" class="search-bar">
            <input
              ref="searchInputRef"
              v-model="searchQuery"
              type="text"
              placeholder="Search... (Esc to close)"
              @keydown.esc="closeSearchOrFilter"
            />
            <button class="icon-btn" @click="closeSearchOrFilter">×</button>
          </div>
          <div v-if="showFilter" class="search-bar filter-bar">
            <input
              ref="filterInputRef"
              v-model="filterQuery"
              type="text"
              placeholder="Filter... (Esc to close)"
              @keydown.esc="closeSearchOrFilter"
            />
            <button class="icon-btn" @click="closeSearchOrFilter">×</button>
          </div>

          <label class="toggle-check" v-show="!showSearch && !showFilter">
            <input v-model="isAutoScroll" type="checkbox" />
            <span class="toggle-box" aria-hidden="true"></span>
            <span>Auto Scroll</span>
          </label>
          <button v-show="!showSearch && !showFilter" class="btn btn-sm btn-outline" @click="emit('clear')">Clear</button>
          <button v-if="tailToken" v-show="!showSearch && !showFilter" class="btn btn-sm btn-danger" @click="emit('stop')">Stop</button>
          <button v-if="!tailToken" v-show="!showSearch && !showFilter" class="btn btn-sm btn-success" @click="emit('start')">Start</button>
        </template>
        <div v-else class="terminal-badge" :class="{ active: hostSessionTabs.length > 0 }">
          {{ hostSessionTabs.length > 0 ? `${hostSessionTabs.length} Terminal(s)` : "No Session" }}
        </div>
      </div>
    </div>

    <div class="viewer-stack">
      <div
        v-show="!selectedFile"
        class="terminal-tabs-header"
        v-if="terminalTabs.length > 0"
      >
        <div
          v-for="tab in terminalTabs"
          :key="tab.id"
          class="terminal-tab"
          :class="{ active: activeTerminalTabId === tab.id }"
          @click="activeTerminalTabId = tab.id"
        >
          <span>{{ tab.name }}</span>
          <button class="close-tab-btn" @click.stop="emit('stop-terminal', tab.id)">×</button>
        </div>
        <button class="add-tab-btn" @click="emit('start-terminal')">+</button>
      </div>

      <div
        v-for="tab in terminalTabs"
        :key="tab.id"
        class="terminal-viewer"
        v-show="activeTerminalTabId === tab.id"
        :class="{ inactive: !sessionId || !tab.token }"
        @click="focusTerminal(tab.id)"
        @contextmenu="handleTerminalContextMenu"
      >
        <div :ref="(el) => setTerminalContainer(tab.id, el)" :data-tab-id="tab.id" class="terminal-host"></div>
      </div>
      
      <teleport to="body">
        <div
          v-if="terminalContextMenu"
          class="terminal-context-menu"
          :style="{ left: `${terminalContextMenu.x}px`, top: `${terminalContextMenu.y}px` }"
          @click.stop
        >
          <button
            v-if="terminalContextMenu.type === 'copy'"
            class="terminal-context-menu-item"
            @click="handleTerminalCopy"
          >
            Copy
          </button>
          <button
            v-else
            class="terminal-context-menu-item"
            @click="handleTerminalPaste"
          >
            Paste
          </button>
        </div>
      </teleport>

      <div v-if="terminalEmptyState()" class="terminal-empty-state">
        <div class="terminal-empty-card">
          <div class="terminal-empty-kicker">Integrated Terminal</div>
          <div class="terminal-empty-title">{{ terminalEmptyState()?.title }}</div>
          <div class="terminal-empty-text">{{ terminalEmptyState()?.text }}</div>
          <button v-if="sessionId" class="btn btn-primary" style="margin-top: 1rem" @click="emit('start-terminal')">Start Terminal</button>
        </div>
      </div>

      <div v-if="selectedFile" ref="localLogViewerRef" class="log-viewer log-viewer-overlay" tabindex="-1">
        <div v-if="content" class="log-lines">
          <div
            v-for="(line, lineIndex) in displayLines"
            :key="lineIndex"
            class="log-line"
            :class="`tone-${line.tone}`"
          >
            <span
              v-for="(segment, segmentIndex) in line.segments"
              :key="`${lineIndex}-${segmentIndex}`"
              class="log-segment"
              :class="[`tone-${segment.tone}`, { 'search-match': segment.isMatch }]"
            >
              {{ segment.text }}
            </span>
          </div>
        </div>
        <div v-else class="empty-viewer">Select Start to load this file, or click the file again to close it.</div>
      </div>
    </div>
  </div>
</template>
