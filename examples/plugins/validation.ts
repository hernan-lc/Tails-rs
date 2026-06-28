import {
  IPlugin,
  PluginInput,
  PluginMetadata,
} from "./types.ts";

export class PluginValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "PluginValidationError";
  }
}

function isValidMetadata(value: unknown): boolean {
  if (!value || typeof value !== "object") return false;
  const m = value as Record<string, unknown>;
  return (
    typeof m.name === "string" &&
    m.name.length > 0 &&
    typeof m.version === "string" &&
    m.version.length > 0
  );
}

function validateHooks(name: string, obj: Record<string, unknown>): void {
  const hooks = ["setup", "onLoad", "onEnable", "onDisable", "onUnload"];
  for (const hook of hooks) {
    if (hook in obj && typeof obj[hook] !== "function") {
      throw new PluginValidationError(
        `Plugin "${name}": "${hook}" must be a function`,
      );
    }
  }
}

function validateMetadata(obj: Record<string, unknown>): void {
  if (!obj.metadata || typeof obj.metadata !== "object") {
    throw new PluginValidationError("Plugin must have a metadata object");
  }
  if (!isValidMetadata(obj.metadata)) {
    throw new PluginValidationError(
      "Plugin metadata must have non-empty 'name' and 'version' strings",
    );
  }
}

export function validatePlugin(plugin: PluginInput): void {
  if (typeof plugin === "function") {
    const proto = (plugin as { prototype?: Record<string, unknown> }).prototype;
    if (proto && typeof proto === "object" && "metadata" in proto) {
      validateMetadata(proto as Record<string, unknown>);
      const metadata = proto.metadata as PluginMetadata;
      validateHooks(metadata.name, proto as Record<string, unknown>);
      return;
    }

    try {
      const instance = new (plugin as new () => IPlugin)();
      if (typeof instance === "object" && instance !== null && "metadata" in instance) {
        const p = instance as unknown as Record<string, unknown>;
        validateMetadata(p);
        const metadata = p.metadata as PluginMetadata;
        validateHooks(metadata.name, p);
        return;
      }
    } catch {
      // instantiation failed, fall through to error
    }

    throw new PluginValidationError(
      "Class plugin must have metadata in prototype or instance",
    );
  }

  if (typeof plugin === "object" && plugin !== null) {
    const p = plugin as Record<string, unknown>;
    validateMetadata(p);
    const metadata = p.metadata as PluginMetadata;
    validateHooks(metadata.name, p);
    return;
  }

  throw new PluginValidationError(
    "Plugin must be a class instance or a plain object with metadata",
  );
}
