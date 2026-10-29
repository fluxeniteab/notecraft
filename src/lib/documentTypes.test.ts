import { describe, expect, it } from "vitest";
import {
  capabilitiesFor,
  documentKind,
  documentTemplate,
  ensureDocumentName,
  extensionOf,
  hasDocumentExtension,
  isProse,
} from "./documentTypes";
import { isDocument, stem, uniqueName } from "./files";

describe("documentKind", () => {
  it("detects every supported extension case-insensitively", () => {
    expect(documentKind("notes/a.MD")).toBe("markdown");
    expect(documentKind("notes/a.mdx")).toBe("mdx");
    expect(documentKind("data/a.JSON")).toBe("json");
    expect(documentKind("conf/a.yml")).toBe("yaml");
    expect(documentKind("site/a.css")).toBe("css");
    expect(documentKind("src/a.tsx")).toBe("typescript");
    expect(documentKind("src/a.jsx")).toBe("javascript");
    expect(documentKind("notes/a.txt")).toBe("plain-text");
  });

  it("treats unknown extensions as plain text, not Markdown", () => {
    expect(documentKind("image.png")).toBe("plain-text");
    expect(documentKind("README")).toBe("plain-text");
    expect(hasDocumentExtension("image.png")).toBe(false);
  });

  it("exposes capabilities per format", () => {
    expect(capabilitiesFor("a.json").preview).toBe("json");
    expect(capabilitiesFor("a.json").stats).toBe("data");
    expect(capabilitiesFor("a.json").hasLinks).toBe(false);
    expect(capabilitiesFor("a.md").stats).toBe("prose");
    expect(capabilitiesFor("a.css").preview).toBe("source");
  });

  it("only prose formats treat wiki links as meaningful", () => {
    expect(isProse("a.md")).toBe(true);
    expect(isProse("a.mdx")).toBe(true);
    expect(isProse("a.json")).toBe(false);
  });
});

describe("file naming", () => {
  it("keeps an explicit supported extension", () => {
    expect(ensureDocumentName("page.mdx")).toBe("page.mdx");
    expect(ensureDocumentName("data.json")).toBe("data.json");
    expect(ensureDocumentName("conf.yml")).toBe("conf.yml");
  });

  it("adds .md only when no supported extension is given", () => {
    expect(ensureDocumentName("Ideas")).toBe("Ideas.md");
    expect(ensureDocumentName("Ideas", "json")).toBe("Ideas.json");
  });

  it("does not double-append extensions when making names unique", () => {
    const files = [
      { path: "page.mdx", name: "page.mdx", is_dir: false, size: 0, modified: 0, extension: "mdx" },
    ];
    expect(uniqueName("page.mdx", files, "")).toBe("page 2.mdx");
    expect(uniqueName("page.md", files, "")).toBe("page.md");
  });

  it("strips any supported extension when computing a stem", () => {
    expect(stem("notes/page.mdx")).toBe("page");
    expect(stem("data/report.json")).toBe("report");
    expect(stem("src/main.tsx")).toBe("main");
    expect(stem("no-extension")).toBe("no-extension");
  });

  it("recognises documents that are not Markdown", () => {
    expect(isDocument("a.json")).toBe(true);
    expect(isDocument("a.yml")).toBe(true);
    expect(isDocument("a.png")).toBe(false);
  });

  it("reads the extension safely from nested and windows-style paths", () => {
    expect(extensionOf("a/b/c.TXT")).toBe("txt");
    expect(extensionOf("a\\b\\c.json")).toBe("json");
    expect(extensionOf("noext")).toBe("");
  });
});

describe("documentTemplate", () => {
  it("produces valid JSON for a new data file", () => {
    const text = documentTemplate("json", "report");
    expect(() => JSON.parse(text)).not.toThrow();
    expect(JSON.parse(text).title).toBe("report");
  });

  it("produces a heading for Markdown and MDX", () => {
    expect(documentTemplate("markdown", "Ideas")).toBe("# Ideas\n\n");
    expect(documentTemplate("mdx", "Page")).toBe("# Page\n\n");
  });

  it("escapes the stem inside JSON strings", () => {
    const text = documentTemplate("json", 'a"b');
    expect(() => JSON.parse(text)).not.toThrow();
  });
});
