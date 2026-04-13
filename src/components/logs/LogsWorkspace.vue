<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { DirEntry, FavoriteItem, HighlightedLine, HostSessionTab } from "../../types/app";
import LogSidebar from "./LogSidebar.vue";
import LogViewer from "./LogViewer.vue";

const activeSessionId = defineModel<string | null>("activeSessionId", { required: true });
const currentPath = defineModel<string>("currentPath", { required: true });
const content = defineModel<string>("content", { required: true });
const terminalTabs = defineModel<import("../../types/app").TerminalTab[]>("terminalTabs", { required: true });
const activeTerminalTabId = defineModel<string | null>("activeTerminalTabId", { required: true });
const isAutoScroll = defineModel<boolean>("isAutoScroll", { required: true });
const logViewerRef = defineModel<HTMLElement | null>("logViewerRef", { required: true });
const isDraggingOverSidebar = defineModel<boolean>("isDraggingOverSidebar", { required: true });

const isSidebarCollapsed = ref(false);

onMounted(() => {
  isSidebarCollapsed.value = localStorage.getItem("logcat.sidebarCollapsed") === "1";
});

watch(isSidebarCollapsed, (collapsed) => {
  localStorage.setItem("logcat.sidebarCollapsed", collapsed ? "1" : "0");
});

defineProps<{
  sessionId: string | null;
  hostSessionTabs: HostSessionTab[];
  showFavorites: boolean;
  entries: DirEntry[];
  currentHostFavorites: FavoriteItem[];
  selectedFile: string | null;
  tailToken: string | null;
  highlightedLines: HighlightedLine[];
  isFavorite: (path: string) => boolean;
  transferProgress: { fileName: string; transferred: number; total: number } | null;
}>();

const emit = defineEmits<{
  (event: "toggle-favorites"): void;
  (event: "disconnect"): void;
  (event: "disconnect-session", sessionId: string): void;
  (event: "select-hosts-tab"): void;
  (event: "refresh"): void;
  (event: "up"): void;
  (event: "open-entry", entry: DirEntry): void;
  (event: "open-favorite", item: FavoriteItem): void;
  (event: "toggle-favorite", entry: DirEntry): void;
  (event: "download-file", entry: DirEntry): void;
  (event: "edit-entry", entry: DirEntry): void;
  (event: "rename-entry", entry: DirEntry): void;
  (event: "change-mode", entry: DirEntry): void;
  (event: "delete-entry", entry: DirEntry): void;
  (event: "create-file"): void;
  (event: "create-dir"): void;
  (event: "clear"): void;
  (event: "stop"): void;
  (event: "start"): void;
  (event: "write-terminal", tabId: string, data: string): void;
  (event: "cd-terminal", tabId: string, path: string): void;
  (event: "resize-terminal", tabId: string, cols: number, rows: number): void;
  (event: "start-terminal"): void;
  (event: "stop-terminal", tabId: string): void;
}>();
</script>

<template>
  <div class="logs-container" :style="{ '--sidebar-pane-width': isSidebarCollapsed ? '0px' : '260px' }">
    <div class="sidebar-pane" :class="{ collapsed: isSidebarCollapsed }">
      <LogSidebar
        v-model:current-path="currentPath"
        v-model:is-dragging-over="isDraggingOverSidebar"
        :session-id="sessionId"
        :show-favorites="showFavorites"
        :entries="entries"
        :current-host-favorites="currentHostFavorites"
        :selected-file="selectedFile"
        :is-favorite="isFavorite"
        :transfer-progress="transferProgress"
        @toggle-favorites="emit('toggle-favorites')"
        @disconnect="emit('disconnect')"
        @select-hosts-tab="emit('select-hosts-tab')"
        @refresh="emit('refresh')"
        @up="emit('up')"
        @open-entry="emit('open-entry', $event)"
        @open-favorite="emit('open-favorite', $event)"
        @toggle-favorite="emit('toggle-favorite', $event)"
        @download-file="emit('download-file', $event)"
        @edit-entry="emit('edit-entry', $event)"
        @rename-entry="emit('rename-entry', $event)"
        @change-mode="emit('change-mode', $event)"
        @delete-entry="emit('delete-entry', $event)"
        @create-file="emit('create-file')"
        @create-dir="emit('create-dir')"
        @cd-terminal="activeTerminalTabId && emit('cd-terminal', activeTerminalTabId, $event)"
      />
    </div>

    <button
      class="sidebar-floating-toggle icon-btn"
      :class="{ collapsed: isSidebarCollapsed }"
      :title="isSidebarCollapsed ? 'Show File Explorer' : 'Hide File Explorer'"
      @click="isSidebarCollapsed = !isSidebarCollapsed"
    >
      <span class="sidebar-toggle-visual" aria-hidden="true">
        <svg v-if="isSidebarCollapsed" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" fill="none" stroke-width="2">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
        <svg v-else viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" fill="none" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </span>
    </button>

    <LogViewer
      v-model:content="content"
      v-model:active-session-id="activeSessionId"
      v-model:terminal-tabs="terminalTabs"
      v-model:active-terminal-tab-id="activeTerminalTabId"
      v-model:is-auto-scroll="isAutoScroll"
      v-model:log-viewer-ref="logViewerRef"
      :session-id="sessionId"
      :host-session-tabs="hostSessionTabs"
      :selected-file="selectedFile"
      :tail-token="tailToken"
      :highlighted-lines="highlightedLines"
      @clear="emit('clear')"
      @stop="emit('stop')"
      @start="emit('start')"
      @write-terminal="(tabId, data) => emit('write-terminal', tabId, data)"
      @resize-terminal="(tabId, cols, rows) => emit('resize-terminal', tabId, cols, rows)"
      @start-terminal="emit('start-terminal')"
      @stop-terminal="(tabId) => emit('stop-terminal', tabId)"
      @disconnect-session="(id) => emit('disconnect-session', id)"
    />
  </div>
</template>
