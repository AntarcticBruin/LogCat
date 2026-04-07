<script setup lang="ts">
import ActivityBar from "./components/layout/ActivityBar.vue";
import TitleBar from "./components/layout/TitleBar.vue";
import HostModal from "./components/hosts/HostModal.vue";
import HostsDashboard from "./components/hosts/HostsDashboard.vue";
import LogsWorkspace from "./components/logs/LogsWorkspace.vue";
import { useLogCatApp } from "./composables/useLogCatApp";
import "./styles/app.css";

const {
  appWindow,
  activeTab,
  isModalOpen,
  savedHosts,
  selectedHostId,
  hostName,
  host,
  port,
  username,
  authType,
  password,
  keyPath,
  passphrase,
  sessionId,
  currentConnectedHostId,
  currentConnectingHostId,
  entries,
  currentPath,
  selectedFile,
  showFavorites,
  tailToken,
  content,
  isConnecting,
  errorMsg,
  logViewer,
  isAutoScroll,
  currentHostFavorites,
  highlightedLines,
  openAddModal,
  openEditModal,
  closeModal,
  saveHost,
  deleteHost,
  connectToHost,
  disconnect,
  refresh,
  enter,
  startTail,
  stopTail,
  up,
  isFavorite,
  toggleFavorite,
  toggleFavoritesPanel,
  openFavorite,
  clearError,
  clearContent,
  pickPrivateKeyPath,
} = useLogCatApp();
</script>

<template>
  <div class="app-container">
    <TitleBar :app-window="appWindow" />

    <div class="app-body">
      <ActivityBar v-model:active-tab="activeTab" />

      <main class="main-content">
        <HostsDashboard
          v-if="activeTab === 'hosts'"
          :saved-hosts="savedHosts"
          :current-connected-host-id="currentConnectedHostId"
          :current-connecting-host-id="currentConnectingHostId"
          @connect="connectToHost"
          @edit="openEditModal"
          @delete="deleteHost"
          @add="openAddModal"
        />

        <LogsWorkspace
          v-else
          v-model:current-path="currentPath"
          v-model:content="content"
          v-model:is-auto-scroll="isAutoScroll"
          v-model:log-viewer-ref="logViewer"
          :session-id="sessionId"
          :show-favorites="showFavorites"
          :entries="entries"
          :current-host-favorites="currentHostFavorites"
          :selected-file="selectedFile"
          :tail-token="tailToken"
          :highlighted-lines="highlightedLines"
          :is-favorite="isFavorite"
          @toggle-favorites="toggleFavoritesPanel"
          @disconnect="disconnect"
          @select-hosts-tab="activeTab = 'hosts'"
          @refresh="refresh"
          @up="up"
          @open-entry="enter"
          @open-favorite="openFavorite"
          @toggle-favorite="toggleFavorite"
          @clear="clearContent"
          @stop="stopTail"
          @start="startTail"
        />
      </main>
    </div>

    <HostModal
      v-model:selected-host-id="selectedHostId"
      v-model:host-name="hostName"
      v-model:host="host"
      v-model:port="port"
      v-model:username="username"
      v-model:auth-type="authType"
      v-model:password="password"
      v-model:key-path="keyPath"
      v-model:passphrase="passphrase"
      :is-open="isModalOpen"
      @close="closeModal"
      @save="saveHost"
      @pick-key-path="pickPrivateKeyPath"
    />

    <div v-if="errorMsg" class="error-banner">
      {{ errorMsg }}
      <button @click="clearError()">×</button>
    </div>

    <div v-if="isConnecting" class="loading-overlay">
      <div class="loading-panel">
        <div class="loading-spinner" aria-hidden="true"></div>
        <div class="loading-title">Connecting</div>
        <div class="loading-text">Establishing SSH session and loading the initial workspace.</div>
      </div>
    </div>
  </div>
</template>
