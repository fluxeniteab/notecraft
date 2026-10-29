import type { FileEntry } from "./types";

export function normalizeRel(path: string): string {
  // Vault-relative paths always use forward slashes. Collapse duplicate
  // separators and `.` segments so `./notes//a.md` and `notes/a.md` are the
  // same tab instead of two.
  return path
    .replace(/\\/g, "/")
    .split("/")
    .filter((segment) => segment && segment !== ".")
    .join("/");
}

export function baseName(path: string): string {
  const clean = normalizeRel(path).replace(/\/+$/, "");
  const i = clean.lastIndexOf("/");
  return i === -1 ? clean : clean.slice(i + 1);
}

export function dirName(path: string): string {
  const clean = normalizeRel(path).replace(/\/+$/, "");
  const i = clean.lastIndexOf("/");
  return i === -1 ? "" : clean.slice(0, i);
}

export function stem(path: string): string {
  const name = baseName(path);
  return name.replace(/\.(md|markdown|mdx|json)$/i, "");
}

export function isMarkdown(path: string): boolean {
  return /\.(md|markdown)$/i.test(path);
}

/** Files the document engine can render as a note (Markdown, MDX, JSON). */
export function isDocument(path: string): boolean {
  return /\.(md|markdown|mdx|json)$/i.test(path);
}

export function joinRel(dir: string, name: string): string {
  const d = normalizeRel(dir).replace(/\/+$/, "");
  const n = name.replace(/^\/+/, "");
  return d ? `${d}/${n}` : n;
}

export function ensureMarkdown(name: string): string {
  const trimmed = name.trim().replace(/[\\/]+/g, "-");
  if (!trimmed) return "";
  return isMarkdown(trimmed) ? trimmed : `${trimmed}.md`;
}

export function uniqueName(name: string, existing: FileEntry[], dir: string): string {
  const wanted = ensureMarkdown(name);
  const inDir = new Set(
    existing
      .filter((f) => dirName(f.path) === normalizeRel(dir))
      .map((f) => f.name.toLowerCase())
  );
  if (!inDir.has(wanted.toLowerCase())) return wanted;
  const dot = wanted.lastIndexOf(".");
  const base = dot > 0 ? wanted.slice(0, dot) : wanted;
  const ext = dot > 0 ? wanted.slice(dot) : "";
  let i = 2;
  while (inDir.has(`${base} ${i}${ext}`.toLowerCase())) i += 1;
  return `${base} ${i}${ext}`;
}

export function resolveNotePath(target: string, files: FileEntry[], from?: string): string | null {
  let raw = target.trim();
  if (!raw) return null;
  // Strip alias and heading fragments from wiki links.
  raw = raw.split("|")[0].split("#")[0].trim();
  if (!raw) return null;
  raw = decodeURIComponent(raw);
  const wantedWithExt = isMarkdown(raw) ? normalizeRel(raw) : `${normalizeRel(raw)}.md`;
  const lower = wantedWithExt.toLowerCase();
  const exact = files.find((f) => !f.is_dir && f.path.toLowerCase() === lower);
  if (exact) return exact.path;
  // Relative to the current note's folder first, then anywhere in the vault.
  const fromDir = from ? dirName(from) : "";
  if (fromDir) {
    const relative = files.find(
      (f) => !f.is_dir && f.path.toLowerCase() === `${fromDir}/${lower}`.toLowerCase()
    );
    if (relative) return relative.path;
  }
  const wantedStem = stem(wantedWithExt).toLowerCase();
  const byStem = files
    .filter((f) => !f.is_dir && stem(f.path).toLowerCase() === wantedStem)
    .sort((a, b) => a.path.length - b.path.length);
  return byStem[0]?.path ?? null;
}

export function countWords(text: string): number {
  const trimmed = text.trim();
  return trimmed ? trimmed.split(/\s+/).length : 0;
}

export function formatRelativeTime(secondsOrIso: number | string): string {
  const ms =
    typeof secondsOrIso === "string"
      ? Date.parse(secondsOrIso)
      : secondsOrIso > 1e12
        ? secondsOrIso
        : secondsOrIso * 1000;
  if (!Number.isFinite(ms)) return "";
  const diff = Date.now() - ms;
  if (diff < 0) return "just now";
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  return new Date(ms).toLocaleDateString();
}

export function highlightParts(text: string, query: string): Array<{ text: string; match: boolean }> {
  const q = query.trim();
  if (!q) return [{ text, match: false }];
  const lower = text.toLowerCase();
  const needle = q.toLowerCase();
  const parts: Array<{ text: string; match: boolean }> = [];
  let i = 0;
  while (i < text.length) {
    const at = lower.indexOf(needle, i);
    if (at === -1) {
      parts.push({ text: text.slice(i), match: false });
      break;
    }
    if (at > i) parts.push({ text: text.slice(i, at), match: false });
    parts.push({ text: text.slice(at, at + needle.length), match: true });
    i = at + needle.length;
  }
  return parts;
}
