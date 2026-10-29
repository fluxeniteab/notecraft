import { Compartment, type EditorState } from "@codemirror/state";

/** The live-preview extension slot must survive remounts along with the state. */
export const liveCompartment = new Compartment();

/**
 * One saved EditorState per tab. The editor component is unmounted when a tab
 * is not visible, but undo history, selection and scroll-independent state must
 * survive that: keeping the states at module scope makes switching tabs lossless.
 */
export const editorStates = new Map<string, EditorState>();

export function forgetEditorState(tabId: string): void {
  editorStates.delete(tabId);
}
