import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "./appStore";
import { normalizeTheme } from "../lib/theme";

const reset = () =>
  useAppStore.setState({
    tabs: [],
    activeTabId: null,
    selectedFile: null,
    saveStatus: {},
    tabStats: {},
    cursors: {},
    recentNotes: [],
  });

describe("appStore tabs", () => {
  beforeEach(reset);

  it("opens a tab", () => {
    useAppStore.getState().openTab("a.md", "a");
    expect(useAppStore.getState().tabs.length).toBe(1);
    expect(useAppStore.getState().activeTabId).not.toBeNull();
  });

  it("does not duplicate tabs", () => {
    useAppStore.getState().openTab("a.md");
    useAppStore.getState().openTab("a.md");
    expect(useAppStore.getState().tabs.length).toBe(1);
  });

  it("normalizes paths before opening", () => {
    useAppStore.getState().openTab("./notes//a.md");
    expect(useAppStore.getState().tabs[0].path).toBe("notes/a.md");
  });

  it("closes tab and activates a neighbour", () => {
    useAppStore.getState().openTab("a.md");
    useAppStore.getState().openTab("b.md");
    useAppStore.getState().openTab("c.md");
    const [a, b, c] = useAppStore.getState().tabs;
    useAppStore.getState().setActiveTab(b.id);
    useAppStore.getState().closeTab(b.id);
    expect(useAppStore.getState().tabs.map((t) => t.path)).toEqual(["a.md", "c.md"]);
    expect(useAppStore.getState().activeTabId).toBe(c.id);
    expect(a.id).not.toBe(c.id);
  });

  it("clears per-tab state when a tab closes", () => {
    useAppStore.getState().openTab("a.md");
    const id = useAppStore.getState().tabs[0].id;
    useAppStore.getState().setSaveStatus(id, "dirty");
    useAppStore.getState().setTabStats(id, { words: 4, chars: 20 });
    useAppStore.getState().setCursor(id, { line: 2, col: 3 });
    useAppStore.getState().closeTab(id);
    const state = useAppStore.getState();
    expect(state.saveStatus[id]).toBeUndefined();
    expect(state.tabStats[id]).toBeUndefined();
    expect(state.cursors[id]).toBeUndefined();
  });

  it("keeps save status independent per tab", () => {
    useAppStore.getState().openTab("a.md");
    useAppStore.getState().openTab("b.md");
    const [a, b] = useAppStore.getState().tabs;
    useAppStore.getState().setSaveStatus(a.id, "saved");
    useAppStore.getState().setSaveStatus(b.id, "error");
    expect(useAppStore.getState().saveStatus[a.id]).toBe("saved");
    expect(useAppStore.getState().saveStatus[b.id]).toBe("error");
  });

  it("closes every tab under a folder", () => {
    ["notes/a.md", "notes/deep/b.md", "other.md"].forEach((path) =>
      useAppStore.getState().openTab(path)
    );
    useAppStore.getState().closeTabsUnder("notes");
    expect(useAppStore.getState().tabs.map((t) => t.path)).toEqual(["other.md"]);
  });

  it("remaps a renamed folder without touching prefix siblings", () => {
    ["notes/a.md", "notes/deep/b.md", "notes2/c.md"].forEach((path) =>
      useAppStore.getState().openTab(path)
    );
    useAppStore.getState().remapPaths("notes", "archive");
    expect(useAppStore.getState().tabs.map((t) => t.path)).toEqual([
      "archive/a.md",
      "archive/deep/b.md",
      "notes2/c.md",
    ]);
  });

  it("remaps the selected file", () => {
    useAppStore.getState().openTab("notes/a.md");
    useAppStore.getState().remapPaths("notes/a.md", "archive/a.md");
    expect(useAppStore.getState().selectedFile).toBe("archive/a.md");
  });

  it("reorders tabs", () => {
    ["a.md", "b.md", "c.md"].forEach((path) => useAppStore.getState().openTab(path));
    useAppStore.getState().reorderTabs(0, 2);
    expect(useAppStore.getState().tabs.map((t) => t.path)).toEqual(["b.md", "c.md", "a.md"]);
  });
});

describe("theme normalization", () => {
  it("treats the removed system theme as dark", () => {
    expect(normalizeTheme("system")).toBe("dark");
  });
  it("keeps light and normalizes unknown values to dark", () => {
    expect(normalizeTheme("light")).toBe("light");
    expect(normalizeTheme(undefined)).toBe("dark");
    expect(normalizeTheme("solarized")).toBe("dark");
  });
});
