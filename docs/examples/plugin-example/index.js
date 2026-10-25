// Example NoteCraft plugin entry point.
// Plugins run in a restricted host context and must declare permissions.
export function activate(api) {
  api.registerCommand("example.hello", () => {
    api.notify("Hello from Example Plugin");
  });
}
export function deactivate() {}
