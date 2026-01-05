import {
  companionPack
} from "./chunk-LM3L3LVG.js";

// src/context.tsx
import { createContext, useContext } from "react";
var noopCache = {
  read: async () => null,
  write: async () => ({ success: false, error: "Context not provided" }),
  exists: async () => false,
  getSize: async () => ({ size: 0, fileCount: 0 }),
  clear: async () => ({ success: false, error: "Context not provided" })
};
var defaultContext = {
  packId: "",
  gameId: 0,
  cache: noopCache
};
var PackContextReact = createContext(defaultContext);
function usePackContext() {
  return useContext(PackContextReact);
}
function usePackCache() {
  return usePackContext().cache;
}

// src/sandbox-runtime.ts
var bridgeState = {
  messageId: 0,
  pending: /* @__PURE__ */ new Map(),
  initialized: false,
  packId: "",
  gameId: 0
};
function bridgeCall(type, payload) {
  return new Promise((resolve, reject) => {
    const id = String(++bridgeState.messageId);
    bridgeState.pending.set(id, {
      resolve,
      reject
    });
    const message = { type, id, ...payload };
    window.parent.postMessage(message, "*");
  });
}
function handleHostMessage(event) {
  const msg = event.data;
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
function initSandboxRuntime() {
  if (typeof window === "undefined") return;
  window.addEventListener("message", handleHostMessage);
  window.parent.postMessage({ type: "pack:ready" }, "*");
}
function createSandboxedCacheAPI() {
  return {
    read: (key) => {
      return bridgeCall("pack:cache:read", { key });
    },
    write: (key, data) => {
      return bridgeCall("pack:cache:write", { key, data });
    },
    exists: (key) => {
      return bridgeCall("pack:cache:exists", { key });
    },
    getSize: () => {
      return bridgeCall("pack:cache:getSize", {});
    },
    clear: () => {
      return bridgeCall("pack:cache:clear", {});
    }
  };
}
var sandboxedCacheAPI = null;
function getSandboxedCacheAPI() {
  if (!sandboxedCacheAPI) {
    sandboxedCacheAPI = createSandboxedCacheAPI();
  }
  return sandboxedCacheAPI;
}
function getSandboxedPackContext() {
  return {
    packId: bridgeState.packId,
    gameId: bridgeState.gameId,
    cache: getSandboxedCacheAPI()
  };
}
function useSandboxedCache() {
  return getSandboxedCacheAPI();
}
export {
  PackContextReact,
  companionPack,
  getSandboxedCacheAPI,
  getSandboxedPackContext,
  initSandboxRuntime,
  usePackCache,
  usePackContext,
  useSandboxedCache
};
