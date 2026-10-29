import { useEffect, useState } from "react";

const STORAGE_KEY = "notecraft:favorites:v1";
const EVENT = "notecraft:favorites-changed";
const LIMIT = 40;

function read(): string[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed)
      ? parsed.filter((value): value is string => typeof value === "string")
      : [];
  } catch {
    return [];
  }
}

function write(next: string[]): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next.slice(0, LIMIT)));
  } catch {
    /* storage can be unavailable in private/embedded contexts */
  }
  window.dispatchEvent(new CustomEvent(EVENT));
}

export function getFavorites(): string[] {
  return read();
}

export function isFavorite(path: string): boolean {
  return read().includes(path);
}

export function toggleFavorite(path: string): string[] {
  const current = read();
  const next = current.includes(path)
    ? current.filter((item) => item !== path)
    : [path, ...current];
  write(next);
  return next;
}

export function removeFavorite(path: string): string[] {
  const next = read().filter((item) => item !== path);
  write(next);
  return next;
}

/** Favourites are per-machine UI state, so they live in localStorage. */
export function useFavorites(): string[] {
  const [favorites, setFavorites] = useState<string[]>(read);
  useEffect(() => {
    const refresh = () => setFavorites(read());
    window.addEventListener(EVENT, refresh);
    window.addEventListener("storage", refresh);
    return () => {
      window.removeEventListener(EVENT, refresh);
      window.removeEventListener("storage", refresh);
    };
  }, []);
  return favorites;
}
