<script setup lang="ts">
import type { HostProfile } from "../../types/app";

defineProps<{
  savedHosts: HostProfile[];
  currentConnectedHostId: string | null;
  currentConnectingHostId: string | null;
}>();

const emit = defineEmits<{
  (event: "connect", host: HostProfile): void;
  (event: "edit", host: HostProfile): void;
  (event: "delete", id: string): void;
  (event: "add"): void;
}>();
</script>

<template>
  <div class="hosts-dashboard">
    <div class="dashboard-header">
      <h2>Saved Hosts</h2>
    </div>
    <div class="host-grid">
      <div
        v-for="host in savedHosts"
        :key="host.id"
        class="host-card"
        :class="{ connected: currentConnectedHostId === host.id, connecting: currentConnectingHostId === host.id }"
        @click="emit('connect', host)"
      >
        <div v-if="currentConnectedHostId === host.id" class="card-status">Connected</div>
        <div v-else-if="currentConnectingHostId === host.id" class="card-status pending">Connecting</div>
        <div class="card-icon">
          <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
            <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
          </svg>
        </div>
        <div class="card-info">
          <div class="card-name">{{ host.name }}</div>
          <div class="card-addr">{{ host.username }}@{{ host.host }}</div>
          <div class="card-meta">{{ host.authType === "key" ? "SSH Key" : "Password" }}</div>
        </div>
        <div class="card-actions">
          <button class="icon-btn" @click.stop="emit('edit', host)">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
            </svg>
          </button>
          <button class="icon-btn danger" @click.stop="emit('delete', host.id)">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"></polyline>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            </svg>
          </button>
        </div>
        <div v-if="currentConnectingHostId === host.id" class="host-card-loader" aria-hidden="true"></div>
      </div>

      <div class="host-card add-card" @click="emit('add')">
        <div class="add-icon">+</div>
        <div class="add-text">Add New Host</div>
      </div>
    </div>
  </div>
</template>
