export type Theme = "dark" | "light";

export interface FileEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  modified: number;
  extension: string;
}

export interface OutlineItem {
  level: number;
  text: string;
  line: number;
  id: string;
}

export interface LinkRef {
  target: string;
  alias?: string | null;
  heading?: string | null;
  block?: string | null;
  line: number;
  kind: string;
}

export interface TaskRef {
  line: number;
  text: string;
  done: boolean;
  indent: number;
  due?: string | null;
  priority?: string | null;
}

export interface ParsedNote {
  frontmatter: { raw: string; data: Record<string, unknown> };
  body: string;
  outline: OutlineItem[];
  links: LinkRef[];
  tags: string[];
  tasks: TaskRef[];
  word_count: number;
  char_count: number;
}

export interface Settings {
  theme: Theme;
  accent: string;
  editor_font_size: number;
  editor_line_height: number;
  editor_font_family: string;
  reading_width: number;
  autosave_ms: number;
  snapshot_interval_min: number;
  snapshot_retention: number;
  excluded_folders: string[];
  hidden_files: boolean;
  reduced_motion: boolean;
  high_contrast: boolean;
  telemetry_opt_in: boolean;
  semantic_search_enabled: boolean;
  daily_note_folder: string;
  daily_note_template: string | null;
  capture_note: string;
  explorer_sort: "name" | "modified" | "size";
  explorer_group_dirs: boolean;
  shortcuts: Record<string, string>;
  writing_goal_words: number;
  sync_folder?: string | null;
  data_dir?: string | null;
}

export interface CollectionConfig {
  id: string;
  name: string;
  config: {
    view?: "table" | "list" | "kanban" | "calendar";
    folder?: string;
    tag?: string;
    property?: string;
    sort?: string;
  };
}

export interface SyncFileStatus {
  path: string;
  status: "in_sync" | "local_newer" | "remote_newer" | "conflict" | "local_only" | "remote_only";
}

export interface TaskItem {
  id: number | null;
  note_path: string;
  line: number;
  text: string;
  done: boolean;
  due: string | null;
  priority: string | null;
}

export interface SearchResult {
  path: string;
  score: number;
  snippet: string;
  title: string;
}

export interface GraphNode {
  id: string;
  label: string;
  x?: number;
  y?: number;
  size?: number;
  color?: string;
  tags?: string[];
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  kind?: string;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface Snapshot {
  id: string;
  note_path: string;
  created_at: string;
  hash: string;
  size: number;
  label?: string | null;
}

export interface Workspace {
  id: string;
  name: string;
  data: string;
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  api_version: string;
  entry: string;
  permissions: string[];
}
