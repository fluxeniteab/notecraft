import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";

vi.mock("./ipc", () => ({
  api: {
    readFile: vi.fn(),
    writeFileIfUnchanged: vi.fn(),
    writeFile: vi.fn(),
    jsonAnalyze: vi.fn(),
  },
}));

import { api } from "./ipc";
import {
  activeTabId,
  closeTab,
  openTab,
  saveTab,
  tabs,
  updateContent,
} from "./state";

const readFile = vi.mocked(api.readFile);
const writeIfUnchanged = vi.mocked(api.writeFileIfUnchanged);
const writeFile = vi.mocked(api.writeFile);

async function reset(): Promise<void> {
  for (const tab of get(tabs)) {
    await closeTab(tab.id);
  }
  tabs.set([]);
  activeTabId.set(null);
  localStorage.clear();
}

describe("document tabs", () => {
  beforeEach(async () => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    readFile.mockResolvedValue("# Hello\n");
    writeIfUnchanged.mockResolvedValue(undefined);
    writeFile.mockResolvedValue(undefined);
    await reset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("opens an MDX file without assuming Markdown and keeps its source", async () => {
    readFile.mockResolvedValue("<Hello name='x' />\n");
    await openTab("notes/page.mdx");
    const tab = get(tabs)[0];
    expect(tab.path).toBe("notes/page.mdx");
    expect(tab.content).toContain("<Hello");
    expect(tab.mode).toBe("source");
    expect(tab.loading).toBe(false);
    expect(tab.base).toBe(tab.content);
  });

  it("marks edits dirty and saves with the last known disk contents", async () => {
    await openTab("note.md");
    const tab = get(tabs)[0];
    updateContent(tab.id, "# Hello again\n");
    expect(get(tabs)[0].dirty).toBe(true);
    expect(get(tabs)[0].status).toBe("dirty");

    await saveTab(tab.id);
    expect(writeIfUnchanged).toHaveBeenCalledWith("note.md", "# Hello again\n", "# Hello\n");
    expect(get(tabs)[0].dirty).toBe(false);
    expect(get(tabs)[0].base).toBe("# Hello again\n");
    expect(get(tabs)[0].status).toBe("saved");
  });

  it("surfaces a conflict without overwriting, then allows an explicit overwrite", async () => {
    await openTab("note.md");
    const tab = get(tabs)[0];
    updateContent(tab.id, "mine\n");
    writeIfUnchanged.mockRejectedValueOnce(new Error("the file changed on disk"));

    await saveTab(tab.id);
    expect(get(tabs)[0].status).toBe("error");
    expect(get(tabs)[0].dirty).toBe(true);
    expect(writeFile).not.toHaveBeenCalled();

    await saveTab(tab.id, true);
    expect(writeFile).toHaveBeenCalledWith("note.md", "mine\n");
    expect(get(tabs)[0].status).toBe("saved");
    expect(get(tabs)[0].dirty).toBe(false);
  });

  it("flushes a dirty tab before closing it", async () => {
    await openTab("draft.md");
    const tab = get(tabs)[0];
    updateContent(tab.id, "new text");
    const closed = await closeTab(tab.id);
    expect(closed).toBe(true);
    expect(writeIfUnchanged).toHaveBeenCalledWith("draft.md", "new text", "# Hello\n");
    expect(get(tabs)).toHaveLength(0);
  });
});
