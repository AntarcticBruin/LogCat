<script setup lang="ts">
import type { DirEntry, FavoriteItem, HighlightedLine } from "../../types/app";
import LogSidebar from "./LogSidebar.vue";
import LogViewer from "./LogViewer.vue";

const currentPath = defineModel<string>("currentPath", { required: true });
const content = defineModel<string>("content", { required: true });
const terminalContent = defineModel<string>("terminalContent", { required: true });
const isAutoScroll = defineModel<boolean>("isAutoScroll", { required: true });
const logViewerRef = defineModel<HTMLElement | null>("logViewerRef", { required: true });
const terminalViewerRef = defineModel<HTMLElement | null>("terminalViewerRef", { required: true });
const isDraggingOverSidebar = defineModel<boolean>("isDraggingOverSidebar", { required: true });

defineProps<{
  sessionId: string | null;
  showFavorites: boolean;
  entries: DirEntry[];
  currentHostFavorites: FavoriteItem[];
  selectedFile: string | null;
  tailToken: string | null;
  terminalToken: string | null;
  highlightedLines: HighlightedLine[];
  isFavorite: (path: string) => boolean;
  transferProgress: { fileName: string; transferred: number; total: number } | null;
}>();

const emit = defineEmits<{
  (event: "toggle-favorites"): void;
  (event: "disconnect"): void;
  (event: "select-hosts-tab"): void;
  (event: "refresh"): void;
  (event: "up"): void;
  (event: "open-entry", entry: DirEntry): void;
  (event: "open-favorite", item: FavoriteItem): void;
  (event: "toggle-favorite", entry: DirEntry): void;
  (event: "download-file", entry: DirEntry): void;
  (event: "clear"): void;
  (event: "stop"): void;
  (event: "start"): void;
  (event: "write-terminal", data: string): void;
  (event: "cd-terminal", path: string): void;
  (event: "resize-terminal", cols: number, rows: number): void;
}>();
</script>

<template>
  <div class="logs-container">
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
      @cd-terminal="emit('cd-terminal', $event)"
    />

    <LogViewer
      v-model:content="content"
      v-model:terminal-content="terminalContent"
      v-model:is-auto-scroll="isAutoScroll"
      v-model:log-viewer-ref="logViewerRef"
      v-model:terminal-viewer-ref="terminalViewerRef"
      :session-id="sessionId"
      :selected-file="selectedFile"
      :tail-token="tailToken"
      :terminal-token="terminalToken"
      :highlighted-lines="highlightedLines"
      @clear="emit('clear')"
      @stop="emit('stop')"
      @start="emit('start')"
      @write-terminal="emit('write-terminal', $event)"
      @resize-terminal="emit('resize-terminal', $event.cols, $event.rows)"
    />
  </div>
</template>
