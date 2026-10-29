import { describe, expect, it } from "vitest";
import { buildTree, filterTree, flattenFiles } from "./tree";
import type { FileEntry } from "./types";

function file(path: string): FileEntry {
  const name = path.slice(path.lastIndexOf("/") + 1);
  const extension = name.includes(".") ? name.slice(name.lastIndexOf(".") + 1) : "";
  return { path, name, is_dir: false, size: 1, modified: 0, extension };
}

describe("buildTree", () => {
  it("synthesises folders that are not in the listing", () => {
    const tree = buildTree([file("journal/2026/day.md"), file("inbox.md")]);
    const journal = tree.find((node) => node.path === "journal");
    expect(journal?.isDir).toBe(true);
    expect(journal?.children[0].path).toBe("journal/2026");
    expect(journal?.children[0].children[0].path).toBe("journal/2026/day.md");
  });

  it("puts folders first and sorts names naturally", () => {
    const tree = buildTree([file("note 10.md"), file("note 2.md"), file("folder/a.md")]);
    expect(tree[0].path).toBe("folder");
    expect(tree.slice(1).map((node) => node.name)).toEqual(["note 2.md", "note 10.md"]);
  });

  it("filters to matching files and keeps their parent folders", () => {
    const tree = buildTree([file("a/alpha.md"), file("a/beta.md"), file("b/gamma.md")]);
    const filtered = filterTree(tree, "alp");
    expect(flattenFiles(filtered).map((node) => node.path)).toEqual(["a/alpha.md"]);
    expect(filtered[0].path).toBe("a");
  });

  it("lists every file once", () => {
    const tree = buildTree([file("a.md"), file("b/c.mdx"), file("b/d.json")]);
    expect(flattenFiles(tree).map((node) => node.path).sort()).toEqual(["a.md", "b/c.mdx", "b/d.json"]);
  });
});
