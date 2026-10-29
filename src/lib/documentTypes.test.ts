import { describe, expect, it } from "vitest";
import { capabilitiesFor, documentKind, ensureDocumentName, hasDocumentExtension } from "./documentTypes";
import { renamedFileName, uniqueName } from "./files";

const entries = [
  { path: "notes.mdx", name: "notes.mdx", is_dir: false, size: 10, modified: 0, extension: "mdx" },
  { path: "data.json", name: "data.json", is_dir: false, size: 10, modified: 0, extension: "json" },
  { path: "plain.txt", name: "plain.txt", is_dir: false, size: 10, modified: 0, extension: "txt" },
];

describe("document registry", () => {
  it("maps supported extensions to their own kinds", () => {
    expect(documentKind("a.mdx")).toBe("mdx");
    expect(documentKind("a.json")).toBe("json");
    expect(documentKind("a.yaml")).toBe("yaml");
    expect(documentKind("a.ts")).toBe("typescript");
  });

  it("never treats an unknown file as Markdown", () => {
    expect(documentKind("archive.xyz")).toBe("plain-text");
    expect(capabilitiesFor("archive.xyz").hasLinks).toBe(false);
  });

  it("keeps an explicit extension when creating a file", () => {
    expect(ensureDocumentName("page.mdx")).toBe("page.mdx");
    expect(ensureDocumentName("schema.json")).toBe("schema.json");
    expect(ensureDocumentName("config.yaml", "yaml")).toBe("config.yaml");
  });

  it("appends the fallback extension only when none is present", () => {
    expect(ensureDocumentName("note", "markdown")).toBe("note.md");
    expect(ensureDocumentName("data", "json")).toBe("data.json");
  });

  it("recognises the supported extension set", () => {
    expect(hasDocumentExtension("x.mdx")).toBe(true);
    expect(hasDocumentExtension("x.xyz")).toBe(false);
  });
});

describe("renaming", () => {
  it("keeps the original extension when none is typed", () => {
    expect(renamedFileName("Project brief", "docs/brief.mdx")).toBe("Project brief.mdx");
    expect(renamedFileName("schema", "data/schema.json")).toBe("schema.json");
  });

  it("preserves an explicit extension, including unknown formats", () => {
    expect(renamedFileName("page.mdx", "page.mdx")).toBe("page.mdx");
    expect(renamedFileName("archive.zip", "notes.txt")).toBe("archive.zip");
    expect(renamedFileName("diagram.v2", "diagram.txt")).toBe("diagram.v2");
  });

  it("does not invent an extension for an extensionless file", () => {
    expect(renamedFileName("LICENSE", "LICENSE")).toBe("LICENSE");
  });
});

describe("unique names", () => {
  it("preserves a typed non-Markdown extension", () => {
    expect(uniqueName("page.mdx", entries, "")).toBe("page.mdx");
    expect(uniqueName("notes.mdx", entries, "")).toBe("notes 2.mdx");
  });

  it("keeps JSON files JSON and text files text when de-duplicating", () => {
    expect(uniqueName("data.json", entries, "")).toBe("data 2.json");
    expect(uniqueName("plain.txt", entries, "")).toBe("plain 2.txt");
  });
});
