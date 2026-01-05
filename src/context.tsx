/**
 * React context for game pack sandboxed APIs.
 *
 * The host application provides this context to pack components,
 * giving them access to cache and other sandboxed APIs without
 * directly accessing Electron internals.
 */

import { createContext, useContext } from "react";
import type { PackContext, PackCacheAPI } from "./types";

// Default no-op cache (used when context not provided)
const noopCache: PackCacheAPI = {
  read: async () => null,
  write: async () => ({ success: false, error: "Context not provided" }),
  exists: async () => false,
  getSize: async () => ({ size: 0, fileCount: 0 }),
  clear: async () => ({ success: false, error: "Context not provided" }),
};

// Default context value
const defaultContext: PackContext = {
  packId: "",
  gameId: 0,
  cache: noopCache,
};

/**
 * React context for pack APIs.
 * Host application provides this via PackContextProvider.
 */
export const PackContextReact = createContext<PackContext>(defaultContext);

/**
 * Hook to access the pack context.
 * Use this in pack components to access cache and other APIs.
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { cache, gameId } = usePackContext();
 *
 *   const loadData = async () => {
 *     const cached = await cache.read('my-data.json');
 *     if (!cached) {
 *       const data = await fetchFromNetwork();
 *       await cache.write('my-data.json', data);
 *     }
 *   };
 * }
 * ```
 */
export function usePackContext(): PackContext {
  return useContext(PackContextReact);
}

/**
 * Hook to access just the cache API.
 * Convenience wrapper around usePackContext().
 */
export function usePackCache(): PackCacheAPI {
  return usePackContext().cache;
}
