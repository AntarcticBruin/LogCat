export type AppTab = "hosts" | "logs";

export type Auth =
  | { type: "password"; password: string }
  | { type: "key"; key_path: string; passphrase?: string | null };

export type HostProfile = {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
} & (
  | {
      authType?: "password";
      password?: string;
    }
  | {
      authType: "key";
      keyPath: string;
      passphrase?: string;
    }
);

export type ConnectOptions = {
  host: string;
  port: number;
  username: string;
  auth: Auth;
  keepalive_ms?: number;
};

export type DirEntry = {
  name: string;
  path: string;
  kind: "file" | "dir" | "symlink" | "other";
  is_symlink: boolean;
  is_text: boolean;
  mode?: number | null;
  size?: number | null;
  mtime?: number | null;
};

export type FavoriteItem = {
  id: string;
  hostId: string;
  hostName: string;
  name: string;
  path: string;
  kind: "file" | "dir";
  is_symlink?: boolean;
};

export type TailEvent = {
  token: string;
  session_id: string;
  path: string;
  chunk: string;
};

export type HostSession = {
  sessionId: string;
  hostId: string;
  profile: HostProfile;
  currentPath: string;
  entries: DirEntry[];
  selectedFile: string | null;
  tailToken: string | null;
  content: string;
  highlightedLines: HighlightedLine[];
  terminalTabs: TerminalTab[];
  activeTerminalTabId: string | null;
  transferProgress: {
    fileName: string;
    transferred: number;
    total: number;
  } | null;
  loading: boolean;
};

export type HostSessionTab = {
  sessionId: string;
  label: string;
};

export type TerminalTab = {
  id: string;
  token: string | null;
  content: string;
  name: string;
  isStarting: boolean;
};

export type TerminalEvent = {
  token: string;
  session_id: string;
  chunk: string;
};

export type TransferProgressEvent = {
  session_id: string;
  file_name: string;
  transferred: number;
  total: number;
};

export type HighlightTone =
  | "default"
  | "error"
  | "warn"
  | "info"
  | "debug"
  | "trace"
  | "success"
  | "muted";

export type HighlightSegment = {
  text: string;
  tone: HighlightTone;
  isMatch?: boolean;
};

export type HighlightedLine = {
  tone: HighlightTone;
  segments: HighlightSegment[];
};
