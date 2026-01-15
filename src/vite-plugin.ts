/**
 * Vite plugin for building Clip Companion game packs.
 *
 * This plugin configures Vite to build a pack's frontend as an IIFE bundle
 * that can be dynamically loaded by the main application in a sandboxed iframe.
 */

import type { Plugin, UserConfig } from "vite";

/**
 * Options for the companion pack Vite plugin.
 */
export interface CompanionPackOptions {
  /** Unique pack identifier (e.g., "league", "valorant") */
  packId: string;
  /** Global variable name for the pack (e.g., "LeaguePack") */
  packName: string;
  /** Entry file path (defaults to "index.ts") */
  entry?: string;
  /** Output directory (defaults to "dist") */
  outDir?: string;
}

/**
 * Vite plugin that configures the build for a companion game pack.
 *
 * This plugin:
 * - Builds the frontend as an IIFE bundle
 * - Externalizes React (provided by host app)
 * - Adds self-registration code to `window.__COMPANION_PACKS__`
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineConfig } from "vite";
 * import react from "@vitejs/plugin-react";
 * import { companionPack } from "@companion/gamepack-runtime/vite";
 *
 * export default defineConfig({
 *   plugins: [
 *     react(),
 *     companionPack({
 *       packId: "league",
 *       packName: "LeaguePack",
 *     }),
 *   ],
 * });
 * ```
 */
export function companionPack(options: CompanionPackOptions): Plugin {
  const { packId, packName, entry = "index.ts", outDir = "dist" } = options;

  return {
    name: "companion-pack",

    config: (): UserConfig => ({
      build: {
        lib: {
          entry,
          name: packName,
          fileName: () => "frontend.js",
          formats: ["iife"],
        },
        rollupOptions: {
          external: ["react", "react-dom"],
          output: {
            globals: {
              react: "React",
              "react-dom": "ReactDOM",
            },
            // Self-registration footer
            footer: `
if (typeof window !== 'undefined') {
  if (!window.__COMPANION_PACKS__) window.__COMPANION_PACKS__ = {};
  window.__COMPANION_PACKS__['${packId}'] = ${packName}.default;
}
            `.trim(),
          },
        },
        outDir,
        minify: true,
        sourcemap: false,
      },
    }),
  };
}

// Also export as default for convenience
export default companionPack;
