<script setup lang="ts">
import type { Window } from "@tauri-apps/api/window";

defineProps<{
  appWindow: Window;
}>();
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-logo">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 17L10 11 4 5"></path>
        <line x1="12" y1="19" x2="20" y2="19"></line>
      </svg>
      <span>DogShell</span>
    </div>
    <div class="titlebar-actions">
      <div class="titlebar-button" @click="appWindow.minimize()">
        <svg viewBox="0 0 10 10" width="10" height="10" stroke="currentColor">
          <line x1="1" y1="5" x2="9" y2="5"></line>
        </svg>
      </div>
      <div class="titlebar-button" @click="appWindow.toggleMaximize()">
        <svg viewBox="0 0 10 10" width="10" height="10" stroke="currentColor" fill="none">
          <rect x="1.5" y="1.5" width="7" height="7"></rect>
        </svg>
      </div>
      <div class="titlebar-button close" @click="appWindow.close()">
        <svg viewBox="0 0 10 10" width="10" height="10" stroke="currentColor">
          <line x1="1.5" y1="1.5" x2="8.5" y2="8.5"></line>
          <line x1="8.5" y1="1.5" x2="1.5" y2="8.5"></line>
        </svg>
      </div>
    </div>
  </div>
</template>
