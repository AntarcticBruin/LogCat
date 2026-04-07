import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AppTab,
  ConnectOptions,
  DirEntry,
  FavoriteItem,
  HighlightedLine,
  HostProfile,
  TailEvent,
} from "../types/app";
import { highlightLogLine } from "../utils/logHighlight";

function normalizeHostProfile(profile: HostProfile): HostProfile {
  if (profile.authType === "key") {
    return {
      ...profile,
      passphrase: profile.passphrase || undefined,
    };
  }

  return {
    ...profile,
    authType: "password",
    password: profile.password || "",
  };
}

export function useLogCatApp() {
  const appWindow = getCurrentWindow();

  const activeTab = ref<AppTab>("hosts");
  const isModalOpen = ref(false);
  const savedHosts = ref<HostProfile[]>([]);
  const selectedHostId = ref<string | null>(null);

  const hostName = ref("");
  const host = ref("");
  const port = ref(22);
  const username = ref("root");
  const authType = ref<"password" | "key">("password");
  const password = ref("");
  const keyPath = ref("");
  const passphrase = ref("");

  const sessionId = ref<string | null>(null);
  const currentConnectedHostId = ref<string | null>(null);
  const currentConnectingHostId = ref<string | null>(null);

  const entries = ref<DirEntry[]>([]);
  const currentPath = ref("/");
  const selectedFile = ref<string | null>(null);
  const favorites = ref<FavoriteItem[]>([]);
  const showFavorites = ref(false);
  const tailToken = ref<string | null>(null);
  const content = ref("");
  const loading = ref(false);
  const isConnecting = ref(false);
  const errorMsg = ref("");

  const logViewer = ref<HTMLElement | null>(null);
  const isAutoScroll = ref(true);
  const favoritePaths = computed(() => {
    const paths = new Set<string>();
    const hostId = currentConnectedHostId.value;
    if (!hostId) {
      return paths;
    }

    for (const item of favorites.value) {
      if (item.hostId === hostId) {
        paths.add(item.path);
      }
    }

    return paths;
  });
  const highlightedLines = ref<HighlightedLine[]>([]);
  const MAX_LINES = 5000;
  const directoryCache = new Map<string, DirEntry[]>();
  let latestDirectoryRequestId = 0;
  let hasPendingHighlightedLine = false;
  let pendingHighlightedChunk = "";

  let unlisten: (() => void) | null = null;

  function visibleEntries(list: DirEntry[]) {
    return list.filter((entry) => entry.kind === "dir" || (entry.kind === "file" && entry.is_text));
  }

  function directoryCacheKey(targetSessionId: string, path: string) {
    return `${targetSessionId}:${path}`;
  }

  function getCachedDirectoryEntries(targetSessionId: string, path: string) {
    return directoryCache.get(directoryCacheKey(targetSessionId, path));
  }

  function setCachedDirectoryEntries(targetSessionId: string, path: string, list: DirEntry[]) {
    directoryCache.set(directoryCacheKey(targetSessionId, path), visibleEntries(list));
  }

  function clearDirectoryState() {
    directoryCache.clear();
    latestDirectoryRequestId += 1;
    entries.value = [];
  }

  onMounted(() => {
    const storedHosts = localStorage.getItem("logcat_hosts");
    if (storedHosts) {
      savedHosts.value = (JSON.parse(storedHosts) as HostProfile[]).map(normalizeHostProfile);
    }

    const storedFavorites = localStorage.getItem("logcat_favorites");
    if (storedFavorites) {
      favorites.value = JSON.parse(storedFavorites) as FavoriteItem[];
    }
  });

  watch(
    savedHosts,
    (newValue) => {
      localStorage.setItem("logcat_hosts", JSON.stringify(newValue));
    },
    { deep: true },
  );

  watch(
    favorites,
    (newValue) => {
      localStorage.setItem("logcat_favorites", JSON.stringify(newValue));
    },
    { deep: true },
  );

  const currentHostFavorites = computed(() =>
    favorites.value.filter((item) => item.hostId === currentConnectedHostId.value),
  );

  function resetHighlightedLines() {
    highlightedLines.value = [];
    hasPendingHighlightedLine = false;
    pendingHighlightedChunk = "";
  }

  function appendHighlightedChunk(chunk: string) {
    const merged = pendingHighlightedChunk + chunk;
    const parts = merged.split("\n");
    const endsWithNewline = merged.endsWith("\n");
    const nextPending = endsWithNewline ? null : parts.pop() ?? "";

    if (hasPendingHighlightedLine) {
      highlightedLines.value.pop();
      hasPendingHighlightedLine = false;
    }

    const newLines = parts.map(highlightLogLine);
    highlightedLines.value.push(...newLines);

    if (nextPending !== null) {
      pendingHighlightedChunk = nextPending;
      highlightedLines.value.push(highlightLogLine(nextPending));
      hasPendingHighlightedLine = true;
    } else {
      pendingHighlightedChunk = "";
    }

    // 维持最大行数限制，防止内存暴涨
    if (highlightedLines.value.length > MAX_LINES) {
      highlightedLines.value.splice(0, highlightedLines.value.length - MAX_LINES);
    }
  }

  function resetHostForm() {
    selectedHostId.value = null;
    hostName.value = "";
    host.value = "";
    port.value = 22;
    username.value = "root";
    authType.value = "password";
    password.value = "";
    keyPath.value = "";
    passphrase.value = "";
  }

  function openAddModal() {
    resetHostForm();
    isModalOpen.value = true;
  }

  function openEditModal(profile: HostProfile) {
    selectedHostId.value = profile.id;
    hostName.value = profile.name;
    host.value = profile.host;
    port.value = profile.port;
    username.value = profile.username;
    authType.value = profile.authType === "key" ? "key" : "password";
    password.value = profile.authType === "key" ? "" : profile.password || "";
    keyPath.value = profile.authType === "key" ? profile.keyPath : "";
    passphrase.value = profile.authType === "key" ? profile.passphrase || "" : "";
    isModalOpen.value = true;
  }

  function closeModal() {
    isModalOpen.value = false;
  }

  function saveHost() {
    if (!host.value || !username.value) return;
    if (authType.value === "key" && !keyPath.value) return;

    const baseProfile = {
      id: selectedHostId.value || crypto.randomUUID(),
      name: hostName.value || host.value,
      host: host.value,
      port: port.value,
      username: username.value,
    };

    const newProfile: HostProfile =
      authType.value === "key"
        ? {
            ...baseProfile,
            authType: "key",
            keyPath: keyPath.value,
            passphrase: passphrase.value || undefined,
          }
        : {
            ...baseProfile,
            authType: "password",
            password: password.value,
          };

    const index = savedHosts.value.findIndex((item) => item.id === newProfile.id);
    if (index >= 0) {
      savedHosts.value[index] = newProfile;
    } else {
      savedHosts.value.push(newProfile);
    }

    closeModal();
  }

  function deleteHost(id: string) {
    savedHosts.value = savedHosts.value.filter((item) => item.id !== id);
  }

  async function connectToHost(profile: HostProfile) {
    selectedHostId.value = profile.id;
    currentConnectingHostId.value = profile.id;
    host.value = profile.host;
    port.value = profile.port;
    username.value = profile.username;
    authType.value = profile.authType === "key" ? "key" : "password";
    password.value = profile.authType === "key" ? "" : profile.password || "";
    keyPath.value = profile.authType === "key" ? profile.keyPath : "";
    passphrase.value = profile.authType === "key" ? profile.passphrase || "" : "";

    loading.value = true;
    isConnecting.value = true;
    errorMsg.value = "";
    activeTab.value = "logs";

    try {
      const options: ConnectOptions = {
        host: host.value,
        port: port.value,
        username: username.value,
        auth:
          authType.value === "key"
            ? {
                type: "key",
                key_path: keyPath.value,
                passphrase: passphrase.value || null,
              }
            : {
                type: "password",
                password: password.value,
              },
        keepalive_ms: 15000,
      };

      const result = await invoke<{ session_id: string }>("connect_ssh", { opts: options });

      if (sessionId.value) {
        await disconnect();
      }

      sessionId.value = result.session_id;
      currentConnectedHostId.value = profile.id;
      currentPath.value = "/";
      showFavorites.value = false;
      clearDirectoryState();
      await refresh();

      if (!unlisten) {
        unlisten = await listen<TailEvent>("tail_data", (event) => {
          if (event.payload?.token === tailToken.value) {
            content.value += event.payload.chunk;
            appendHighlightedChunk(event.payload.chunk);
            if (isAutoScroll.value) {
              scrollToBottom();
            }
          }
        });
      }
    } catch (error) {
      errorMsg.value = String(error);
      activeTab.value = "hosts";
    } finally {
      currentConnectingHostId.value = null;
      isConnecting.value = false;
      loading.value = false;
    }
  }

  async function disconnect() {
    if (!sessionId.value) return;

    await stopTail();
    try {
      await invoke("disconnect_ssh", { sessionId: sessionId.value });
    } catch (error) {
      console.error(error);
    }

    sessionId.value = null;
    currentConnectedHostId.value = null;
    clearDirectoryState();
    content.value = "";
    resetHighlightedLines();
    selectedFile.value = null;
    showFavorites.value = false;
  }

  async function refresh(options?: { force?: boolean; path?: string }) {
    const activeSessionId = sessionId.value;
    if (!activeSessionId) return;

    const targetPath = options?.path ?? currentPath.value;
    const requestId = ++latestDirectoryRequestId;
    const cachedEntries = !options?.force ? getCachedDirectoryEntries(activeSessionId, targetPath) : undefined;

    // 性能优化：如果缓存命中，立即同步更新 UI，实现“秒开”体验
    if (cachedEntries) {
      entries.value = cachedEntries;
    } else {
      // 只有在完全没有数据时才显示加载状态，减少闪烁
      loading.value = true;
    }

    try {
      const list = await invoke<DirEntry[]>("list_dir", {
        sessionId: activeSessionId,
        path: targetPath,
      });
      const processedEntries = visibleEntries(list);
      setCachedDirectoryEntries(activeSessionId, targetPath, list);

      if (
        requestId === latestDirectoryRequestId
        && sessionId.value === activeSessionId
        && currentPath.value === targetPath
      ) {
        entries.value = processedEntries;
      }
    } catch (error) {
      if (requestId === latestDirectoryRequestId) {
        errorMsg.value = `Failed to list dir: ${error}`;
      }
    } finally {
      if (requestId === latestDirectoryRequestId) {
        loading.value = false;
      }
    }
  }

  function openDirectory(path: string) {
    if (!sessionId.value) return;

    showFavorites.value = false;
    selectedFile.value = null;
    currentPath.value = path;

    const cachedEntries = getCachedDirectoryEntries(sessionId.value, path);
    if (cachedEntries) {
      entries.value = cachedEntries;
    }

    void refresh({ path });
  }

  function enter(entry: DirEntry) {
    if (entry.kind === "dir") {
      openDirectory(entry.path);
      return;
    }

    if (entry.kind === "file" && entry.is_text) {
      showFavorites.value = false;
      selectedFile.value = entry.path;
      void startTail();
    }
  }

  async function startTail() {
    if (!sessionId.value || !selectedFile.value) return;

    await stopTail();
    content.value = "";
    resetHighlightedLines();

    try {
      tailToken.value = await invoke<string>("start_tail", {
        sessionId: sessionId.value,
        path: selectedFile.value,
        lines: 200,
      });
    } catch (error) {
      errorMsg.value = `Failed to tail: ${error}`;
    }
  }

  async function stopTail() {
    if (!sessionId.value || !tailToken.value) return;

    try {
      await invoke("stop_tail", { sessionId: sessionId.value, token: tailToken.value });
    } catch (error) {
      console.error(error);
    }

    tailToken.value = null;
  }

  function up() {
    const path = currentPath.value;
    if (path === "/") return;

    const parent = path.endsWith("/") ? path.slice(0, -1) : path;
    const index = parent.lastIndexOf("/");
    openDirectory(index <= 0 ? "/" : parent.slice(0, index));
  }

  function dirname(path: string) {
    if (path === "/") return "/";
    const normalized = path.endsWith("/") ? path.slice(0, -1) : path;
    const index = normalized.lastIndexOf("/");
    return index <= 0 ? "/" : normalized.slice(0, index);
  }

  function isFavorite(path: string) {
    return favoritePaths.value.has(path);
  }

  function toggleFavorite(entry: DirEntry) {
    if (!currentConnectedHostId.value || (entry.kind !== "dir" && entry.kind !== "file")) return;

    const index = favorites.value.findIndex(
      (item) => item.hostId === currentConnectedHostId.value && item.path === entry.path,
    );

    if (index >= 0) {
      favorites.value.splice(index, 1);
      return;
    }

    const currentHostName =
      savedHosts.value.find((item) => item.id === currentConnectedHostId.value)?.name || "Current Host";

    favorites.value.push({
      id: crypto.randomUUID(),
      hostId: currentConnectedHostId.value,
      hostName: currentHostName,
      name: entry.name,
      path: entry.path,
      kind: entry.kind,
    });
  }

  function toggleFavoritesPanel() {
    showFavorites.value = !showFavorites.value;
  }

  async function openFavorite(item: FavoriteItem) {
    if (!sessionId.value) return;

    showFavorites.value = false;

    if (item.kind === "dir") {
      openDirectory(item.path);
      return;
    }

    currentPath.value = dirname(item.path);
    selectedFile.value = item.path;
    const cachedEntries = getCachedDirectoryEntries(sessionId.value, currentPath.value);
    if (cachedEntries) {
      entries.value = cachedEntries;
    }
    await refresh({ path: currentPath.value });
    await startTail();
  }

  function scrollToBottom() {
    void nextTick(() => {
      if (logViewer.value) {
        logViewer.value.scrollTop = logViewer.value.scrollHeight;
      }
    });
  }

  function clearError() {
    errorMsg.value = "";
  }

  function clearContent() {
    content.value = "";
    resetHighlightedLines();
  }

  async function pickPrivateKeyPath() {
    try {
      const defaultPath = await invoke<string>("default_ssh_key_dir");
      const selected = await open({
        defaultPath,
        directory: false,
        multiple: false,
      });

      if (typeof selected === "string") {
        keyPath.value = selected;
      }
    } catch (error) {
      errorMsg.value = `Failed to pick key: ${error}`;
    }
  }

  onBeforeUnmount(() => {
    if (unlisten) {
      unlisten();
    }
  });

  return {
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
    favorites,
    showFavorites,
    tailToken,
    content,
    loading,
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
  };
}
