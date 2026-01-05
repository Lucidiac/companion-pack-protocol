/**
 * Types for building Clip Companion game packs.
 *
 * These types define the interface between game packs and the main application.
 */

import type { ComponentType } from "react";

/**
 * Base match type that all game packs extend.
 * Contains only fields common to all games.
 *
 * Game-specific data is stored in the `details` field as an opaque JSON object.
 * Each pack defines its own details interface and is responsible for parsing it.
 */
export interface BaseMatch<TDetails = Record<string, unknown>> {
  /** Unique match identifier */
  id: string;
  /** Pack UUID (stable internal identifier) */
  packId?: string;
  /** Game ID this match belongs to (for backwards compatibility) */
  gameId: number;
  /** ISO timestamp of when the match was played */
  playedAt: string;
  /** Match duration in seconds */
  durationSecs: number;
  /** Match result */
  result: "win" | "loss" | "remake";
  /** ISO timestamp of when this record was created */
  createdAt: string;
  /**
   * Game-specific match details.
   * Each pack defines its own details schema and is responsible for rendering it.
   * The main application treats this as an opaque JSON blob.
   */
  details: TDetails;
}

/**
 * Props for match card components.
 */
export interface MatchCardProps<TMatch = BaseMatch> {
  /** The match data to display */
  match: TMatch;
  /** Whether this card is currently selected */
  isSelected?: boolean;
  /** Click handler for selecting this match */
  onClick?: () => void;
}

/**
 * Props for live match card components.
 */
export interface LiveMatchCardProps<TLiveMatch = unknown> {
  /** The live match data to display */
  match: TLiveMatch;
}

/**
 * Resource loading state for a game pack.
 */
export interface ResourceState {
  /** Whether resources are currently being loaded */
  loading: boolean;
  /** Whether resources have been successfully loaded and are ready to use */
  ready: boolean;
  /** Optional loading progress (0-100) */
  progress?: number;
  /** Optional description of what's currently loading */
  currentItem?: string;
  /** Cached data version (e.g., DDragon version for League) */
  version?: string | null;
}

/**
 * Resource management API for game packs.
 * Enables lazy loading of game-specific assets like icons, data, etc.
 */
export interface GamePackResources {
  /** Check if resources are ready (already loaded/cached) */
  isReady: () => boolean;

  /** Initialize resources (lazy load). Safe to call multiple times. */
  init: () => Promise<void>;

  /** Get current loading state */
  getState: () => ResourceState;

  /** Subscribe to state changes. Returns unsubscribe function. */
  onStateChange: (callback: (state: ResourceState) => void) => () => void;
}

/**
 * Cache API provided to game packs by the host application.
 * Packs use this sandboxed interface instead of directly accessing storage.
 * The host automatically namespaces cache by game ID.
 */
export interface PackCacheAPI {
  /**
   * Read a cached value.
   * @param key - Cache key (e.g., "champions.json", "images/Ahri.png")
   * @returns Cached value or null if not found
   */
  read: <T = unknown>(key: string) => Promise<T | null>;

  /**
   * Write a value to cache.
   * @param key - Cache key
   * @param data - Data to cache (will be JSON serialized)
   */
  write: (key: string, data: unknown) => Promise<{ success: boolean; error?: string }>;

  /**
   * Check if a key exists in cache.
   * @param key - Cache key
   */
  exists: (key: string) => Promise<boolean>;

  /**
   * Get the total size of this pack's cache.
   * @returns Size in bytes and file count
   */
  getSize: () => Promise<{ size: number; fileCount: number }>;

  /**
   * Clear all cached data for this pack.
   */
  clear: () => Promise<{ success: boolean; error?: string }>;
}

/**
 * Context provided to game packs by the host application.
 * This is the sandboxed environment packs operate within.
 */
export interface PackContext {
  /** Pack UUID (stable internal identifier) */
  packId: string;

  /** Game ID this pack is registered for (for backwards compatibility) */
  gameId: number;

  /** Cache API for storing game-specific data */
  cache: PackCacheAPI;
}

/**
 * Utility functions that packs can export for use by generic components.
 * These enable the core app's viewers to display game-specific content.
 */
export interface GamePackUtilities {
  /**
   * Get URL for a game-specific icon/asset.
   * @param type - Asset type (e.g., "champion", "spell", "item")
   * @param id - Asset identifier (e.g., champion name, spell ID)
   * @returns URL to the asset image
   */
  getAssetUrl?: (type: string, id: string) => string;

  /**
   * Human-readable names for game event types.
   * Used by Timeline component to display event markers.
   */
  eventDisplayNames?: Record<string, string>;
}

/**
 * Interface that each game pack must implement.
 * Provides components and utilities specific to that game.
 */
export interface GamePack<
  TMatch extends BaseMatch = BaseMatch,
  TLiveMatch = unknown,
> {
  /** Pack UUID (stable internal identifier) */
  packId: string;

  /** Static game ID (for backwards compatibility) */
  gameId: number;

  /** URL-safe slug (e.g., "league", "valorant") */
  slug: string;

  /** Component for rendering a match card in the match list */
  MatchCard: ComponentType<MatchCardProps<TMatch>>;

  /** Component for rendering a live match card (optional) */
  LiveMatchCard?: ComponentType<LiveMatchCardProps<TLiveMatch>>;

  /**
   * Resource management for lazy loading game assets.
   * Call resources.init() when the first match card for this game mounts.
   */
  resources?: GamePackResources;

  /**
   * Utility functions for game-specific asset URLs and display names.
   * Used by generic components to display game-specific content.
   */
  utilities?: GamePackUtilities;

  /** Component showing asset loading status (optional) */
  AssetsStatus?: ComponentType;

  /** Type guard to check if a match belongs to this game */
  isMatch: (match: BaseMatch<unknown>) => match is TMatch;

  /**
   * Initialize game assets.
   * @deprecated Use resources.init() instead
   */
  initAssets?: () => Promise<void>;
}

/**
 * Runtime game pack interface.
 * Uses unknown for flexibility when loading packs dynamically.
 */
export interface RuntimeGamePack {
  packId: string;
  gameId: number;
  slug: string;
  MatchCard: ComponentType<{
    match: unknown;
    isSelected?: boolean;
    onClick?: () => void;
  }>;
  LiveMatchCard?: ComponentType<{ match: unknown }>;
  resources?: GamePackResources;
  utilities?: GamePackUtilities;
  AssetsStatus?: ComponentType;
  isMatch: (match: BaseMatch<unknown>) => boolean;
}

/**
 * Game definition from the games registry.
 */
export interface GameDefinition {
  /** Unique game ID */
  id: number;
  /** URL-safe slug */
  slug: string;
  /** Full game name */
  name: string;
  /** Short name for UI */
  shortName: string;
  /** Whether this game is enabled */
  enabled: boolean;
}

// ============================================
// Game Event Types (for triggers/clips)
// ============================================

/**
 * A game event that can trigger clip recording.
 */
export interface GameEvent {
  /** Event type key (e.g., "ChampionKill", "TurretDestroyed") */
  eventKey: string;
  /** When the event occurred (game time in seconds) */
  eventTime: number;
  /** Additional event data (game-specific) */
  eventData?: Record<string, unknown>;
  /** Seconds to capture before the event */
  preSecs?: number;
  /** Seconds to capture after the event */
  postSecs?: number;
}

/**
 * Current game connection/status.
 */
export interface GameStatus {
  /** Whether connected to the game client */
  connected: boolean;
  /** Current game phase (e.g., "Lobby", "InProgress", "EndOfGame") */
  phase: string;
  /** Whether actively in a game */
  isInGame: boolean;
}

/**
 * Match data returned when a game session ends.
 */
export interface MatchData {
  /** Pack UUID (stable internal identifier) */
  packId?: string;
  /** Game slug (e.g., "league") */
  gameSlug: string;
  /** Game ID (for backwards compatibility) */
  gameId: number;
  /** Match result */
  result: "win" | "loss" | "remake" | "unknown";
  /** Match duration in seconds */
  duration: number;
  /** Game-specific match details */
  details: Record<string, unknown>;
}
