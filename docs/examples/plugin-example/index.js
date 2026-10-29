// Example NoteCraft plugin entry point.
//
// The module is loaded as an ES module and receives one context object:
//   { pluginId, version, api, toast, store }
// `api` is the same typed bridge the app uses, `toast` shows notifications and
// `store` is the app store (use getState() to read it).
export async function activate({ pluginId, version, api, toast, store }) {
  toast.success(`Example plugin ${version} loaded`, { description: pluginId });

  // Count notes once and expose the result on the document for the stylesheet.
  try {
    const files = await api.listAllFiles();
    const notes = files.filter((file) => !file.is_dir);
    document.documentElement.dataset.exampleNoteCount = String(notes.length);
  } catch (error) {
    console.warn("Example plugin could not list notes", error);
  }

  // Store actions are available too, for example opening a note:
  // store.getState().openTab("welcome.md");
}
