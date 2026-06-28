import fs from "./fs.native";
import path from "./path.native";
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

export async function loadPluginsFromDir(
  dir: string,
): Promise<PluginInput[]> {
  const dirPath = path.resolve(dir);
  const entries: string[] = await fs.readdir(dirPath);
  const plugins: PluginInput[] = [];
  for (const name of entries) {
    if (!name.endsWith(".ts") && !name.endsWith(".js")) continue;
    const filePath = `${dirPath}/${name}`;
    try {
      const plugin = await loadPluginFromFile(filePath);
      plugins.push(plugin);
    } catch {
      // skip invalid plugins
    }
  }
  return plugins;
}
