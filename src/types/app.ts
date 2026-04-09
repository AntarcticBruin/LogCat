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
  is_text: boolean;
  size?: number | null;
};

export type FavoriteItem = {
  id: string;
  hostId: string;
  hostName: string;
  name: string;
  path: string;
  kind: "file" | "dir";
};

export type TailEvent = {
  token: string;
  session_id: string;
  path: string;
  chunk: string;
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
};

export type HighlightedLine = {
  tone: HighlightTone;
  segments: HighlightSegment[];
};
