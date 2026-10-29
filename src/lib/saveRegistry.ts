type Flusher = () => Promise<void>;

const flushers = new Map<string, Flusher>();

export function registerFlusher(tabId: string, flush: Flusher): () => void {
  flushers.set(tabId, flush);
  return () => {
    if (flushers.get(tabId) === flush) flushers.delete(tabId);
  };
}

export async function flushTab(tabId: string): Promise<void> {
  const flush = flushers.get(tabId);
  if (!flush) return;
  try {
    await flush();
  } catch {
    // Save failures are surfaced by the editor pane itself.
  }
}

export async function flushAll(): Promise<void> {
  await Promise.allSettled([...flushers.values()].map((flush) => flush()));
}
