import { PluginManager } from "./plugin_manager";
import { loadPluginsFromDir } from "./loader";
const manager = new PluginManager();

// Load all plugins from ./plugins directory
const plugins = await loadPluginsFromDir("./examples/plugins/plugins");
//console.log("plugins", plugins);
for (const plugin of plugins) {
  manager.register(plugin);
}

await manager.init();

// Full access to loaded plugins
const counter = manager.getPlugin("counter") as {
  increment(): void;
  getCount(): number;
};
counter?.increment();
counter?.increment();
console.log(`Counter: ${counter?.getCount()}`);

const time = manager.getPlugin("time") as {
  now(): string;
};
console.log(`Time: ${time?.now()}`);

await manager.shutdown();
