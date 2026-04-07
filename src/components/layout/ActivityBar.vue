<script setup lang="ts">
import type { AppTab } from "../../types/app";

defineProps<{
  activeTab: AppTab;
}>();

const emit = defineEmits<{
  (event: "update:activeTab", value: AppTab): void;
}>();
</script>

<template>
  <nav class="activity-bar">
    <div
      class="activity-item"
      :class="{ active: activeTab === 'hosts' }"
      @click="emit('update:activeTab', 'hosts')"
    >
      <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
        <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
        <line x1="6" y1="6" x2="6.01" y2="6"></line>
        <line x1="6" y1="18" x2="6.01" y2="18"></line>
      </svg>
      <span>Hosts</span>
    </div>
    <div
      class="activity-item"
      :class="{ active: activeTab === 'logs' }"
      @click="emit('update:activeTab', 'logs')"
    >
      <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
        <polyline points="14 2 14 8 20 8"></polyline>
      </svg>
      <span>Logs</span>
    </div>
  </nav>
</template>
