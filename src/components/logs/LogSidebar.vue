<script setup lang="ts">
import type { DirEntry, FavoriteItem } from "../../types/app";

defineProps<{
  sessionId: string | null;
  showFavorites: boolean;
  currentPath: string;
  entries: DirEntry[];
  currentHostFavorites: FavoriteItem[];
  selectedFile: string | null;
  isFavorite: (path: string) => boolean;
  transferProgress: { fileName: string; transferred: number; total: number } | null;
}>();

const emit = defineEmits<{
  (event: "toggle-favorites"): void;
  (event: "disconnect"): void;
  (event: "select-hosts-tab"): void;
  (event: "update:currentPath", value: string): void;
  (event: "refresh"): void;
  (event: "up"): void;
  (event: "open-entry", entry: DirEntry): void;
  (event: "open-favorite", item: FavoriteItem): void;
  (event: "toggle-favorite", entry: DirEntry): void;
  (event: "download-file", entry: DirEntry): void;
  (event: "cd-terminal", path: string): void;
}>();

const asFavoriteEntry = (item: FavoriteItem): DirEntry => ({
  name: item.name,
  path: item.path,
  kind: item.kind,
  is_text: item.kind === "file",
});

const isDraggingOver = defineModel<boolean>("isDraggingOver", { default: false });
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="brand">File Explorer</span>
      <div class="sidebar-actions">
        <button
          v-if="sessionId"
          class="icon-btn sidebar-favorites-btn"
          :class="{ active: showFavorites }"
          :title="showFavorites ? 'Show current folder' : 'Show favorites'"
          @click="emit('toggle-favorites')"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" stroke="currentColor" stroke-width="1.5">
            <path d="M12 2.75l2.86 5.8 6.4.93-4.63 4.51 1.09 6.37L12 17.36l-5.72 3 1.1-6.37L2.75 9.48l6.39-.93L12 2.75z"></path>
          </svg>
        </button>
        <button v-if="sessionId" class="btn btn-sm btn-danger" @click="emit('disconnect')">Disconnect</button>
      </div>
    </div>

    <div v-if="!sessionId" class="no-conn-state">
      <p>No active connection.</p>
      <button class="btn btn-primary" @click="emit('select-hosts-tab')">Select a Host</button>
    </div>

    <div
      v-else
      class="file-explorer"
      :class="{ 'dragging-over': isDraggingOver }"
    >
      <div class="path-bar">
        <button class="icon-btn" @click="emit('up')" title="Up one level">
          <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" fill="none" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <input
          class="path-input"
          :value="currentPath"
          @input="emit('update:currentPath', ($event.target as HTMLInputElement).value)"
          @keyup.enter="emit('refresh')"
        />
        <button class="icon-btn" title="Open in Terminal" @click="emit('cd-terminal', currentPath)">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 17 10 11 4 5"></polyline>
            <line x1="12" y1="19" x2="20" y2="19"></line>
          </svg>
        </button>
      </div>

      <div v-if="showFavorites" class="favorites-panel">
        <div class="favorites-panel-header">
          <span>Favorites</span>
          <span class="favorites-count">{{ currentHostFavorites.length }}</span>
        </div>
        <div v-if="currentHostFavorites.length" class="file-list">
          <div
            v-for="item in currentHostFavorites"
            :key="item.id"
            class="file-item favorite-item"
            :class="{ selected: selectedFile === item.path }"
            @click="emit('open-favorite', item)"
          >
            <svg v-if="item.kind === 'dir'" viewBox="0 0 24 24" width="14" height="14" fill="#dcb67a">
              <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"></path>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="#519aba" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
              <polyline points="14 2 14 8 20 8"></polyline>
            </svg>
            <div class="favorite-meta">
              <span class="file-name">{{ item.name }}</span>
              <span class="favorite-path">{{ item.path }}</span>
            </div>
            <button
              class="icon-btn favorite-toggle active"
              title="Remove favorite"
              @click.stop="emit('toggle-favorite', asFavoriteEntry(item))"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" stroke="currentColor" stroke-width="1.5">
                <path d="M12 2.75l2.86 5.8 6.4.93-4.63 4.51 1.09 6.37L12 17.36l-5.72 3 1.1-6.37L2.75 9.48l6.39-.93L12 2.75z"></path>
              </svg>
            </button>
          </div>
        </div>
        <div v-else class="favorites-empty">No favorites on this host yet.</div>
      </div>

      <div v-else class="file-list">
        <div
          v-for="entry in entries"
          :key="entry.path"
          class="file-item"
          :class="{ selected: selectedFile === entry.path }"
          @click="emit('open-entry', entry)"
        >
          <svg v-if="entry.kind === 'dir'" viewBox="0 0 24 24" width="14" height="14" fill="#dcb67a">
            <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"></path>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="#519aba" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
            <polyline points="14 2 14 8 20 8"></polyline>
          </svg>
          <span class="file-name">{{ entry.name }}</span>
          
          <button
            v-if="entry.kind === 'file'"
            class="icon-btn download-btn"
            title="Download file"
            @click.stop="emit('download-file', entry)"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
          </button>

          <button
            v-if="entry.kind === 'dir'"
            class="icon-btn terminal-btn"
            title="Open in Terminal"
            @click.stop="emit('cd-terminal', entry.path)"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="4 17 10 11 4 5"></polyline>
              <line x1="12" y1="19" x2="20" y2="19"></line>
            </svg>
          </button>

          <button
            class="icon-btn favorite-toggle"
            :class="{ active: isFavorite(entry.path) }"
            :title="isFavorite(entry.path) ? 'Remove favorite' : 'Add favorite'"
            @click.stop="emit('toggle-favorite', entry)"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" stroke="currentColor" stroke-width="1.5">
              <path d="M12 2.75l2.86 5.8 6.4.93-4.63 4.51 1.09 6.37L12 17.36l-5.72 3 1.1-6.37L2.75 9.48l6.39-.93L12 2.75z"></path>
            </svg>
          </button>
        </div>
      </div>
      
      <div v-if="transferProgress" class="transfer-progress">
        <div class="progress-info">
          <span class="progress-filename">{{ transferProgress.fileName }}</span>
          <span class="progress-percentage" v-if="transferProgress.total > 0">
            {{ Math.round((transferProgress.transferred / transferProgress.total) * 100) }}%
          </span>
          <span class="progress-percentage" v-else>
            {{ (transferProgress.transferred / 1024 / 1024).toFixed(1) }} MB
          </span>
        </div>
        <div class="progress-bar-container">
          <div 
            class="progress-bar" 
            :style="{ 
              width: transferProgress.total > 0 ? `${(transferProgress.transferred / transferProgress.total) * 100}%` : '100%',
              animation: transferProgress.total === 0 ? 'indeterminate-progress 1.5s infinite linear' : 'none',
              transformOrigin: 'left'
            }"
          ></div>
        </div>
      </div>
    </div>
  </aside>
</template>
