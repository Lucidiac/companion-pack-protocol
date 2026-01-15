import { Plugin } from 'vite';

/**
 * Vite plugin for building Clip Companion game packs.
 *
 * This plugin configures Vite to build a pack's frontend as an IIFE bundle
 * that can be dynamically loaded by the main application in a sandboxed iframe.
 */

/**
 * Options for the companion pack Vite plugin.
 */
interface CompanionPackOptions {
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
declare function companionPack(options: CompanionPackOptions): Plugin;

export { type CompanionPackOptions, companionPack, companionPack as default };
