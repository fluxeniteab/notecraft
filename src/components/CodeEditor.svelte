<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import type { Tab } from "../lib/state";
  import { updateContent, updateTab } from "../lib/state";
  import { editorExtensions, setActiveEditor } from "../lib/editor";
  import { livePreview } from "../lib/livePreview";
  import { editorStates, liveCompartment } from "../lib/editorCache";
  import { capabilitiesFor } from "../lib/documentTypes";

  export let tab: Tab;
  export let active = false;

  let host: HTMLDivElement;
  let view: EditorView | null = null;
  let applying = false;
  let mode = tab.mode;

  $: code = capabilitiesFor(tab.path).stats !== "prose";

  onMount(() => {
    const cached = editorStates.get(tab.id);
    const state =
      cached ??
      EditorState.create({
        doc: tab.content,
        extensions: [
          editorExtensions(tab.path),
          liveCompartment.of(tab.mode === "live" ? livePreview : []),
          EditorView.updateListener.of((update) => {
            if (applying) return;
            if (update.docChanged) {
              updateContent(tab.id, update.state.doc.toString());
            }
            if (update.selectionSet || update.docChanged) {
              const head = update.state.selection.main.head;
              const line = update.state.doc.lineAt(head);
              updateTab(tab.id, { cursor: { line: line.number, col: head - line.from + 1 } });
            }
          }),
        ],
      });

    view = new EditorView({ state, parent: host });
    // A cached state can carry the other mode's compartment value; apply the
    // current tab's mode explicitly after attaching.
    view.dispatch({ effects: liveCompartment.reconfigure(tab.mode === "live" ? livePreview : []) });
    setActiveEditor(view);
    if (active) requestAnimationFrame(() => view?.focus());
  });

  onDestroy(() => {
    if (!view) return;
    editorStates.set(tab.id, view.state);
    if (document.activeElement && view.dom.contains(document.activeElement)) setActiveEditor(null);
    view.destroy();
    view = null;
  });

  $: if (active && view) setActiveEditor(view);

  $: if (view && tab.mode !== mode) {
    mode = tab.mode;
    view.dispatch({ effects: liveCompartment.reconfigure(tab.mode === "live" ? livePreview : []) });
  }

  // Only pull text back from the store when it really diverges (external reload,
  // conflict resolution). Normal typing already arrived from this editor.
  $: if (view && !applying && tab.content !== view.state.doc.toString()) {
    applying = true;
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: tab.content } });
    applying = false;
  }
</script>

<div class="editor-mount" class:code bind:this={host}></div>
