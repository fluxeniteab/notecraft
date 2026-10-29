import { describe, it, expect, beforeEach } from "vitest";
import { getFavorites, isFavorite, toggleFavorite, removeFavorite } from "./favorites";

describe("favorites", () => {
  beforeEach(() => window.localStorage.clear());

  it("starts empty", () => {
    expect(getFavorites()).toEqual([]);
  });

  it("pins and unpins a note", () => {
    toggleFavorite("notes/a.md");
    expect(isFavorite("notes/a.md")).toBe(true);
    toggleFavorite("notes/a.md");
    expect(isFavorite("notes/a.md")).toBe(false);
  });

  it("keeps the newest pin first", () => {
    toggleFavorite("a.md");
    toggleFavorite("b.md");
    expect(getFavorites()).toEqual(["b.md", "a.md"]);
  });

  it("removes a favorite explicitly", () => {
    toggleFavorite("a.md");
    toggleFavorite("b.md");
    removeFavorite("a.md");
    expect(getFavorites()).toEqual(["b.md"]);
  });

  it("ignores corrupt stored values", () => {
    window.localStorage.setItem("notecraft:favorites:v1", "{not json");
    expect(getFavorites()).toEqual([]);
  });
});
