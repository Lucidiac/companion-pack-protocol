/**
 * @companion/pack-protocol
 *
 * Types and utilities for building Clip Companion game packs.
 *
 * This package provides:
 * - Type definitions for pack interfaces
 * - React context and hooks for sandboxed APIs
 * - PostMessage bridge for iframe sandbox communication
 * - Vite plugin for building pack bundles
 *
 * @example
 * ```ts
 * import type { GamePack, BaseMatch } from "@companion/pack-protocol";
 * import { usePackCache } from "@companion/pack-protocol";
 * import { MatchCard } from "./MatchCard";
 *
 * interface MyGameMatch extends BaseMatch {
 *   // game-specific fields
 * }
 *
 * const pack: GamePack<MyGameMatch> = {
 *   gameId: 99,
 *   slug: "my-game",
 *   MatchCard,
 *   isMatch: (match): match is MyGameMatch => match.gameId === 99,
 * };
 *
 * export default pack;
 * ```
 */

// Export all types
export type {
  // Core match types (new architecture)
  CoreMatchData,
  MatchWithDetails,
  // Legacy match types (backwards compatibility)
  BaseMatch,
  // Component props
  MatchCardProps,
  LiveMatchCardProps,
  // Resource management
  ResourceState,
  GamePackResources,
  GamePackUtilities,
  // Pack context
  PackCacheAPI,
  PackContext,
  // Pack definitions
  GamePack,
  RuntimeGamePack,
  GameDefinition,
  // Events and status
  GameEvent,
  GameStatus,
  MatchData,
} from "./types";

// Export React context and hooks for pack sandboxed APIs
export { PackContextReact, usePackContext, usePackCache } from "./context";

// Export bridge types for host/pack communication
export type {
  PackToHostMessage,
  HostToPackMessage,
  PackReadyMessage,
  PackCacheReadMessage,
  PackCacheWriteMessage,
  PackCacheExistsMessage,
  PackCacheGetSizeMessage,
  PackCacheClearMessage,
  HostInitMessage,
  HostResponseMessage,
  HostRenderMessage,
  PendingRequest,
  BridgeState,
} from "./bridge";

// Export sandbox runtime for packs running in iframes
export {
  initSandboxRuntime,
  getSandboxedCacheAPI,
  getSandboxedPackContext,
  useSandboxedCache,
} from "./sandbox-runtime";

// Re-export the Vite plugin for convenience
// Users can also import directly from "@companion/pack-protocol/vite"
export { companionPack, type CompanionPackOptions } from "./vite-plugin";
