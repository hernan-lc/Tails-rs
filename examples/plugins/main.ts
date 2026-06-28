import { PluginManager } from "./plugin_manager.ts";
import { loadPluginFromFile, loadPluginsFromDir } from "./loader.ts";
import { PluginInput } from "./types.ts";

// --- Example: Register a plugin from a plain object ---
const helloPlugin: PluginInput = {
  metadata: { name: "hello", version: "1.0.0" },
  setup() {
    console.log("[hello] setup called");
  },
  onLoad() {
    console.log("[hello] loaded");
  },
  onEnable() {
    console.log("[hello] enabled");
  },
  onDisable() {
    console.log("[hello] disabled");
  },
  onUnload() {
    console.log("[hello] unloaded");
  },
};

const manager = new PluginManager();

// Register the plugin
manager.register(helloPlugin);
console.log("Registered plugins:", manager.getPlugins());

// Initialize (calls setup + onLoad for each plugin)
await manager.init();

// Shutdown (calls onDisable + onUnload for each plugin)
await manager.shutdown();
console.log("All plugins shut down.");

// --- Example: Load plugins from a directory ---
// const plugins = await loadPluginsFromDir("./plugins/list");
// for (const plugin of plugins) {
//   manager.register(plugin);
// }
// await manager.init();
