<script setup lang="ts">
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { ref } from "vue";
import { nextTick, onBeforeUnmount, onMounted, watch } from "vue";
import type { HighlightedLine } from "../../types/app";
import { readText } from "@tauri-apps/plugin-clipboard-manager";

const content = defineModel<string>("content", { required: true });
const terminalContent = defineModel<string>("terminalContent", { required: true });
const isAutoScroll = defineModel<boolean>("isAutoScroll", { required: true });
const logViewerRef = defineModel<HTMLElement | null>("logViewerRef", { required: true });
const terminalViewerRef = defineModel<HTMLElement | null>("terminalViewerRef", { required: true });

const props = defineProps<{
  sessionId: string | null;
  selectedFile: string | null;
  tailToken: string | null;
  terminalToken: string | null;
  highlightedLines: HighlightedLine[];
}>();

const emit = defineEmits<{
  (event: "clear"): void;
  (event: "stop"): void;
  (event: "start"): void;
  (event: "write-terminal", data: string): void;
  (event: "resize-terminal", payload: { cols: number; rows: number }): void;
}>();

let resizeObserver: ResizeObserver | null = null;
let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let lastRenderedTerminalLength = 0;
let disposeDataHandler: { dispose(): void } | null = null;
const localLogViewerRef = ref<HTMLElement | null>(null);
const localTerminalViewerRef = ref<HTMLElement | null>(null);
const terminalTheme = {
  background: "#141414",
  foreground: "#d4d4d4",
  cursor: "#79c0ff",
  cursorAccent: "#141414",
  selectionBackground: "rgba(121, 192, 255, 0.20)",
  selectionForeground: "#ffffff",
  scrollbarSliderBackground: "rgba(255, 255, 255, 0.14)",
  scrollbarSliderHoverBackground: "rgba(255, 255, 255, 0.22)",
  scrollbarSliderActiveBackground: "rgba(255, 255, 255, 0.28)",
  overviewRulerBorder: "transparent",
  black: "#1e1e1e",
  red: "#f14c4c",
  green: "#89d185",
  yellow: "#d7ba7d",
  blue: "#4fc1ff",
  magenta: "#c586c0",
  cyan: "#4ec9b0",
  white: "#d4d4d4",
  brightBlack: "#6a6a6a",
  brightRed: "#ff6a6a",
  brightGreen: "#b5cea8",
  brightYellow: "#f0d58c",
  brightBlue: "#9cdcfe",
  brightMagenta: "#d7a7d1",
  brightCyan: "#7fe7d2",
  brightWhite: "#ffffff",
} as const;

function focusTerminal() {
  if (!props.selectedFile && props.sessionId && props.terminalToken) {
    terminal?.focus();
  }
}

function terminalEmptyState() {
  if (!props.sessionId) {
    return {
      title: "No Active Session",
      text: "Select a host on the left to open a persistent SSH terminal. File previews will appear above it without closing the shell.",
    };
  }

  if (!props.terminalToken) {
    return {
      title: "Preparing Terminal",
      text: "The SSH connection is established. Initializing the interactive shell and syncing terminal size.",
    };
  }

  return null;
}

function syncTerminalResize() {
  if (!terminal || !fitAddon || !props.terminalToken) return;

  fitAddon.fit();
  emit("resize-terminal", {
    cols: Math.max(20, terminal.cols),
    rows: Math.max(8, terminal.rows),
  });
}

function resetTerminalView() {
  if (!terminal) return;

  terminal.reset();
  lastRenderedTerminalLength = 0;

  if (terminalContent.value) {
    terminal.write(terminalContent.value);
    lastRenderedTerminalLength = terminalContent.value.length;
  }
}

function syncTerminalOutput() {
  if (!terminal) return;

  if (terminalContent.value.length < lastRenderedTerminalLength) {
    resetTerminalView();
    return;
  }

  const chunk = terminalContent.value.slice(lastRenderedTerminalLength);
  if (!chunk) return;

  terminal.write(chunk);
  lastRenderedTerminalLength = terminalContent.value.length;
}

async function handleTerminalContextMenu() {
  if (props.selectedFile || !terminal || !props.terminalToken) return;
  
  try {
    const text = await readText();
    if (text) {
      emit("write-terminal", text);
    }
  } catch (error) {
    console.error("Failed to read clipboard:", error);
  }
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
  terminalContent,
  () => {
    syncTerminalOutput();
  },
  { flush: "post" },
);

watch(
  () => props.selectedFile,
  (selectedFile) => {
    if (selectedFile) {
      terminal?.blur();
      void nextTick(() => {
        if (localLogViewerRef.value) {
          localLogViewerRef.value.scrollTop = localLogViewerRef.value.scrollHeight;
        }
      });
      return;
    }

    void nextTick(() => focusTerminal());
  },
);

watch(
  () => [props.sessionId, props.terminalToken, props.selectedFile] as const,
  ([sessionId, terminalToken, selectedFile]) => {
    if (!terminal) return;

    const isInteractive = Boolean(sessionId && terminalToken && !selectedFile);
    terminal.options.cursorBlink = isInteractive;
    terminal.options.theme = {
      ...terminalTheme,
      cursor: isInteractive ? terminalTheme.cursor : "transparent",
    };

    if (!isInteractive) {
      terminal.blur();
    }
  },
  { flush: "post" },
);

watch(
  () => props.terminalToken,
  () => {
    resetTerminalView();
    void nextTick(() => {
      syncTerminalResize();
      focusTerminal();
    });
  },
);

onMounted(() => {
  terminalViewerRef.value = localTerminalViewerRef.value;

  terminal = new Terminal({
    cursorBlink: false,
    fontFamily: "Consolas, 'Cascadia Mono', 'SFMono-Regular', monospace",
    fontSize: 13,
    lineHeight: 1.2,
    fontWeight: "500",
    fontWeightBold: "800",
    scrollback: 5000,
    convertEol: false,
    allowProposedApi: false,
    minimumContrastRatio: 1.2,
    drawBoldTextInBrightColors: true,
    theme: terminalTheme,
    overviewRuler: {
      width: 0,
    },
  });

  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);

  if (localTerminalViewerRef.value) {
    terminal.open(localTerminalViewerRef.value);
    syncTerminalOutput();
    focusTerminal();
    syncTerminalResize();

    resizeObserver = new ResizeObserver(() => syncTerminalResize());
    resizeObserver.observe(localTerminalViewerRef.value);
  }

  disposeDataHandler = terminal.onData((data) => {
    if (!props.selectedFile) {
      emit("write-terminal", data);
    }
  });
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  disposeDataHandler?.dispose();
  terminal?.dispose();
});
</script>

<template>
  <div class="log-content">
    <div class="toolbar">
      <div class="tab-title">
        {{ selectedFile ? selectedFile.split("/").pop() : "SSH Terminal" }}
      </div>
      <div class="actions">
        <template v-if="selectedFile">
          <label class="toggle-check">
            <input v-model="isAutoScroll" type="checkbox" />
            <span class="toggle-box" aria-hidden="true"></span>
            <span>Auto Scroll</span>
          </label>
          <button class="btn btn-sm btn-outline" @click="emit('clear')">Clear</button>
          <button v-if="tailToken" class="btn btn-sm btn-danger" @click="emit('stop')">Stop</button>
          <button v-if="!tailToken" class="btn btn-sm btn-success" @click="emit('start')">Start</button>
        </template>
        <div v-else class="terminal-badge" :class="{ active: terminalToken }">
          {{ terminalToken ? "Terminal Ready" : "Initializing" }}
        </div>
      </div>
    </div>

    <div class="viewer-stack">
      <div
        class="terminal-viewer"
        :class="{ inactive: !sessionId || !terminalToken }"
        @click="focusTerminal"
        @contextmenu="handleTerminalContextMenu"
      >
        <div ref="localTerminalViewerRef" class="terminal-host"></div>
      </div>
      <div v-if="terminalEmptyState()" class="terminal-empty-state">
        <div class="terminal-empty-card">
          <div class="terminal-empty-kicker">Integrated Terminal</div>
          <div class="terminal-empty-title">{{ terminalEmptyState()?.title }}</div>
          <div class="terminal-empty-text">{{ terminalEmptyState()?.text }}</div>
        </div>
      </div>

      <div v-if="selectedFile" ref="localLogViewerRef" class="log-viewer log-viewer-overlay">
        <div v-if="content" class="log-lines">
          <div
            v-for="(line, lineIndex) in highlightedLines"
            :key="lineIndex"
            class="log-line"
            :class="`tone-${line.tone}`"
          >
            <span
              v-for="(segment, segmentIndex) in line.segments"
              :key="`${lineIndex}-${segmentIndex}`"
              class="log-segment"
              :class="`tone-${segment.tone}`"
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
