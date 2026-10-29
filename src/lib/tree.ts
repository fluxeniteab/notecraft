import type { FileEntry } from "./types";
import { normalizeRel } from "./files";

export interface TreeNode {
  path: string;
  name: string;
  isDir: boolean;
  extension: string;
  modified: number;
  children: TreeNode[];
}

type SortMode = "name" | "modified" | "size";

function compare(a: TreeNode, b: TreeNode, sort: SortMode, groupDirs: boolean): number {
  if (groupDirs && a.isDir !== b.isDir) return a.isDir ? -1 : 1;
  if (sort === "modified") return a.modified - b.modified || a.name.localeCompare(b.name);
  return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
}

/**
 * The backend returns a flat list. Directories are created on demand so a vault
 * with only files at `a/b/c.md` still shows every intermediate folder.
 */
export function buildTree(entries: FileEntry[], sort: SortMode = "name", groupDirs = true): TreeNode[] {
  const root: TreeNode = {
    path: "",
    name: "",
    isDir: true,
    extension: "",
    modified: 0,
    children: [],
  };
  const dirs = new Map<string, TreeNode>([["", root]]);

  const ensureDir = (path: string): TreeNode => {
    const existing = dirs.get(path);
    if (existing) return existing;
    const parentPath = path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
    const parent = ensureDir(parentPath);
    const node: TreeNode = {
      path,
      name: path.slice(path.lastIndexOf("/") + 1),
      isDir: true,
      extension: "",
      modified: 0,
      children: [],
    };
    parent.children.push(node);
    dirs.set(path, node);
    return node;
  };

  for (const entry of entries) {
    const path = normalizeRel(entry.path);
    if (!path) continue;
    if (entry.is_dir) {
      const node = ensureDir(path);
      node.modified = entry.modified;
      continue;
    }
    const parentPath = path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
    const parent = ensureDir(parentPath);
    parent.children.push({
      path,
      name: entry.name || path.slice(path.lastIndexOf("/") + 1),
      isDir: false,
      extension: entry.extension || "",
      modified: entry.modified,
      children: [],
    });
  }

  const sortNode = (node: TreeNode): void => {
    node.children.sort((a, b) => compare(a, b, sort, groupDirs));
    for (const child of node.children) if (child.isDir) sortNode(child);
  };
  sortNode(root);
  return root.children;
}

export function filterTree(nodes: TreeNode[], query: string): TreeNode[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return nodes;
  const walk = (node: TreeNode): TreeNode | null => {
    if (!node.isDir) return node.name.toLowerCase().includes(needle) || node.path.toLowerCase().includes(needle) ? node : null;
    const children = node.children.map(walk).filter((child): child is TreeNode => !!child);
    if (children.length || node.name.toLowerCase().includes(needle)) return { ...node, children };
    return null;
  };
  return nodes.map(walk).filter((node): node is TreeNode => !!node);
}

export function flattenFiles(nodes: TreeNode[]): TreeNode[] {
  const out: TreeNode[] = [];
  const walk = (node: TreeNode): void => {
    if (node.isDir) node.children.forEach(walk);
    else out.push(node);
  };
  nodes.forEach(walk);
  return out;
}
