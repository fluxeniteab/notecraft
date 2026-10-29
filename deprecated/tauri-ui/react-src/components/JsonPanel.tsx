import { useCallback, useEffect, useState } from "react";
import {
  AlertTriangle,
  Braces,
  Check,
  ChevronRight,
  Copy,
  Minimize2,
  Search,
  Wand2,
} from "lucide-react";
import { toast } from "sonner";
import { api } from "../lib/tauri";
import type { JsonTreeNode } from "../lib/types";

interface Analysis {
  valid: boolean;
  error: { message: string; line: number; column: number } | null;
  root: JsonTreeNode | null;
  keys: number;
  nodes: number;
  depth: number;
  truncated: boolean;
}

/**
 * Structured JSON view for a JSON tab.
 *
 * The editor source stays authoritative: formatting and minifying are explicit
 * actions that write a new document into the editor, and invalid input is
 * reported rather than rewritten.
 */
export default function JsonPanel({
  content,
  onApply,
}: {
  content: string;
  onApply: (next: string) => void;
}) {
  const [analysis, setAnalysis] = useState<Analysis | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState<"format" | "minify" | null>(null);

  useEffect(() => {
    let cancelled = false;
    const timer = window.setTimeout(() => {
      api
        .jsonAnalyze(content)
        .then((result) => {
          if (!cancelled) {
            setAnalysis(result as Analysis);
            setFailed(null);
          }
        })
        .catch((error) => {
          if (!cancelled) setFailed(String(error));
        });
    }, 220);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [content]);

  const apply = useCallback(
    async (mode: "format" | "minify") => {
      setBusy(mode);
      try {
        const next =
          mode === "format"
            ? await api.jsonFormat(content, 2)
            : await api.jsonMinify(content);
        onApply(next);
        toast.success(mode === "format" ? "Formatted JSON" : "Minified JSON");
      } catch (error) {
        toast.error("Invalid JSON — nothing was changed", {
          description: String(error).replace(/^Error:\s*/, ""),
        });
      } finally {
        setBusy(null);
      }
    },
    [content, onApply]
  );

  const needle = query.trim().toLowerCase();

  return (
    <div className="nc-json-panel">
      <div className="nc-json-toolbar">
        <span className="nc-json-badge">
          <Braces size={11} />
          JSON
        </span>
        {analysis?.valid && (
          <span className="nc-json-stat">
            {analysis.keys} keys · {analysis.nodes} values · depth {analysis.depth}
          </span>
        )}
        {analysis && !analysis.valid && (
          <span className="nc-json-invalid">
            <AlertTriangle size={11} />
            Invalid
          </span>
        )}
        <div className="flex-1" />
        <label className="nc-json-find">
          <Search size={11} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Filter keys and values"
            aria-label="Filter JSON keys and values"
          />
        </label>
        <button
          type="button"
          className="nc-btn nc-btn-ghost nc-btn-xs"
          onClick={() => void apply("format")}
          disabled={busy !== null || !analysis?.valid}
          title="Pretty-print with 2-space indentation"
        >
          <Wand2 size={12} /> {busy === "format" ? "Formatting…" : "Format"}
        </button>
        <button
          type="button"
          className="nc-btn nc-btn-ghost nc-btn-xs"
          onClick={() => void apply("minify")}
          disabled={busy !== null || !analysis?.valid}
          title="Collapse to a single line"
        >
          <Minimize2 size={12} /> {busy === "minify" ? "Minifying…" : "Minify"}
        </button>
      </div>

      {failed && (
        <div className="nc-json-error">
          <strong>Could not analyse this document</strong>
          <span>{failed}</span>
        </div>
      )}

      {analysis?.error && (
        <div className="nc-json-error" role="alert">
          <strong>
            Invalid JSON at line {analysis.error.line}, column {analysis.error.column}
          </strong>
          <span>{analysis.error.message}</span>
        </div>
      )}

      {analysis?.valid && analysis.root && (
        <div className="nc-json-tree" role="tree" aria-label="JSON structure">
          <JsonBranch node={analysis.root} needle={needle} depth={0} />
        </div>
      )}

      {analysis?.truncated && (
        <p className="nc-json-note">
          The tree stops at the display limit. The document itself is unchanged.
        </p>
      )}
    </div>
  );
}

function matches(node: JsonTreeNode, needle: string): boolean {
  if (!needle) return true;
  return (
    node.key.toLowerCase().includes(needle) ||
    node.preview.toLowerCase().includes(needle) ||
    node.children.some((child) => matches(child, needle))
  );
}

function JsonBranch({
  node,
  needle,
  depth,
}: {
  node: JsonTreeNode;
  needle: string;
  depth: number;
}) {
  const [open, setOpen] = useState(depth < 2);
  const [copied, setCopied] = useState(false);

  if (!matches(node, needle)) return null;

  const label = node.key === "" ? "root" : node.key;
  const isContainer = node.kind === "object" || node.kind === "array";

  const copyPointer = async () => {
    try {
      await navigator.clipboard.writeText(node.pointer || "/");
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      toast.error("Could not copy the JSON Pointer");
    }
  };

  if (!isContainer) {
    return (
      <div className="nc-json-row" role="treeitem" style={{ paddingLeft: depth * 14 }}>
        <span className="nc-json-leafdot" />
        <span className="nc-json-key">{label}</span>
        <span className="nc-json-type">{node.kind}</span>
        <span className="nc-json-value">{node.preview}</span>
        <button
          type="button"
          className="nc-json-copy"
          onClick={() => void copyPointer()}
          title={`Copy JSON Pointer ${node.pointer || "/"}`}
          aria-label={`Copy JSON Pointer for ${label}`}
        >
          {copied ? <Check size={10} /> : <Copy size={10} />}
        </button>
      </div>
    );
  }

  const forcedOpen = needle.length > 0;
  const expanded = forcedOpen || open;

  return (
    <div className="nc-json-group" role="treeitem" aria-expanded={expanded}>
      <div className="nc-json-row" style={{ paddingLeft: depth * 14 }}>
        <button
          type="button"
          className="nc-json-twisty"
          onClick={() => setOpen((value) => !value)}
          aria-label={expanded ? `Collapse ${label}` : `Expand ${label}`}
        >
          <ChevronRight size={11} data-open={expanded} />
        </button>
        <span className="nc-json-key">{label}</span>
        <span className="nc-json-type">{node.kind}</span>
        <span className="nc-json-count">{node.children.length}</span>
        <button
          type="button"
          className="nc-json-copy"
          onClick={() => void copyPointer()}
          title={`Copy JSON Pointer ${node.pointer || "/"}`}
          aria-label={`Copy JSON Pointer for ${label}`}
        >
          {copied ? <Check size={10} /> : <Copy size={10} />}
        </button>
      </div>
      {expanded &&
        node.children.map((child) => (
          <JsonBranch
            key={child.pointer || child.key}
            node={child}
            needle={needle}
            depth={depth + 1}
          />
        ))}
    </div>
  );
}
