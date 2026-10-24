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
  theme: string;
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
  data_dir?: string | null;
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
