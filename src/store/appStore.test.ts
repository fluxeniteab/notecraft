import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "./appStore";

describe("appStore", () => {
  beforeEach(() => useAppStore.setState({ tabs: [], activeTabId: null }));
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
  it("closes tab", () => {
    useAppStore.getState().openTab("a.md");
    const id = useAppStore.getState().tabs[0].id;
    useAppStore.getState().closeTab(id);
    expect(useAppStore.getState().tabs.length).toBe(0);
  });
  it("toggles panel", () => {
    const before = useAppStore.getState().panels.leftSidebar;
    useAppStore.getState().togglePanel("leftSidebar");
    expect(useAppStore.getState().panels.leftSidebar).toBe(!before);
  });
});
