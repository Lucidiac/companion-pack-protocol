/**
 * Sandbox runtime for gamepacks running inside iframes.
 *
 * This module provides the bridge implementation that allows pack code
 * to communicate with the host application via postMessage.
 *
 * This code runs INSIDE the sandboxed iframe.
 */

import type { PackCacheAPI, PackContext } from "./types";
import type { PackToHostMessage, HostToPackMessage, BridgeState } from "./bridge";

// Bridge state (module-scoped, runs in iframe)
const bridgeState: BridgeState & { packId: string } = {
  messageId: 0,
  pending: new Map(),
  initialized: false,
  packId: "",
  gameId: 0,
};

/**
 * Make a bridge call to the host and wait for response.
 */
function bridgeCall<T>(
  type: PackToHostMessage["type"],
  payload: Omit<PackToHostMessage, "type" | "id">
): Promise<T> {
  return new Promise((resolve, reject) => {
    const id = String(++bridgeState.messageId);
    bridgeState.pending.set(id, {
      resolve: resolve as (data: unknown) => void,
      reject,
    });

    const message = { type, id, ...payload } as PackToHostMessage;
    window.parent.postMessage(message, "*");
  });
}

/**
 * Handle incoming messages from the host.
 */
function handleHostMessage(event: MessageEvent): void {
  const msg = event.data as HostToPackMessage;

  if (msg.type === "host:init") {
    bridgeState.initialized = true;
    bridgeState.gameId = msg.gameId;
    return;
  }

  if (msg.type === "host:response") {
    const pending = bridgeState.pending.get(msg.id);
    if (pending) {
      bridgeState.pending.delete(msg.id);
      if (msg.success) {
        pending.resolve(msg.data);
      } else {
        pending.reject(msg.error || "Unknown error");
      }
    }
  }
}

/**
 * Initialize the sandbox runtime.
 * Call this once when the pack iframe loads.
 */
export function initSandboxRuntime(): void {
  if (typeof window === "undefined") return;

  // Listen for host messages
  window.addEventListener("message", handleHostMessage);

  // Notify host that pack is ready
  window.parent.postMessage({ type: "pack:ready" }, "*");
}

/**
 * Create the sandboxed cache API that communicates via postMessage.
 */
function createSandboxedCacheAPI(): PackCacheAPI {
  return {
    read: <T = unknown>(key: string): Promise<T | null> => {
      return bridgeCall<T | null>("pack:cache:read", { key });
    },

    write: (key: string, data: unknown): Promise<{ success: boolean; error?: string }> => {
      return bridgeCall("pack:cache:write", { key, data });
    },

    exists: (key: string): Promise<boolean> => {
      return bridgeCall("pack:cache:exists", { key });
    },

    getSize: (): Promise<{ size: number; fileCount: number }> => {
      return bridgeCall("pack:cache:getSize", {});
    },

    clear: (): Promise<{ success: boolean; error?: string }> => {
      return bridgeCall("pack:cache:clear", {});
    },
  };
}

// Singleton cache API instance
let sandboxedCacheAPI: PackCacheAPI | null = null;

/**
 * Get the sandboxed cache API.
 * Use this in pack components running inside an iframe sandbox.
 */
export function getSandboxedCacheAPI(): PackCacheAPI {
  if (!sandboxedCacheAPI) {
    sandboxedCacheAPI = createSandboxedCacheAPI();
  }
  return sandboxedCacheAPI;
}

/**
 * Get the sandboxed pack context.
 */
export function getSandboxedPackContext(): PackContext {
  return {
    packId: bridgeState.packId,
    gameId: bridgeState.gameId,
    cache: getSandboxedCacheAPI(),
  };
}

/**
 * Hook-like function to get the cache API.
 * Compatible with the usePackCache() interface but works in sandbox.
 *
 * Note: For actual React hook usage, import usePackCache from context.tsx
 * and the host will provide the appropriate implementation.
 */
export function useSandboxedCache(): PackCacheAPI {
  return getSandboxedCacheAPI();
}
