import fs from "fs/promises";
import path from "path";
import { PluginInput } from "./types.ts";
import { validatePlugin } from "./validation.ts";

export async function loadPluginFromFile(
  pluginPath: string,
): Promise<PluginInput> {
  const mod = await import(pluginPath);
  const plugin = mod.default ?? mod.plugin ?? mod;
  validatePlugin(plugin as PluginInput);
  return plugin as PluginInput;
}

export async function loadPluginsFromDir(dir: string): Promise<PluginInput[]> {
  const dirPath: string = path.resolve(dir);
  const entries: string[] = await fs.readdir(dirPath); // Now this works perfectly with await!
  //console.log("dirPath", dirPath, entries);
  const plugins: PluginInput[] = [];
  for (let i = 0; i < entries.length; i++) {
    const name: string = entries[i];
    if (!name.endsWith(".ts") && !name.endsWith(".js")) continue;
    const filePath: string = `${dirPath}/${name}`;
    try {
      const plugin: PluginInput = await loadPluginFromFile(filePath);
      plugins.push(plugin);
    } catch {
      // skip invalid plugins
    }
  }
  return plugins;
}
