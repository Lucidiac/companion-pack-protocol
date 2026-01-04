# @companion/pack-protocol

TypeScript types and utilities for building Clip Companion game packs.

## Data Model

Game packs emit three types of data to the daemon:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    GAMEPACK OUTPUTS                                 │
├─────────────────────┬─────────────────────┬─────────────────────────┤
│       EVENTS        │     STATISTICS      │       MOMENTS           │
│   (GameEvent)       │   (get_live_data)   │      (Moment)           │
│                     │                     │                         │
│  Discrete           │  Continuous         │  "Something             │
│  occurrences        │  polled state       │  interesting!"          │
│                     │                     │                         │
│  • ChampionKill     │  • KDA              │  • pentakill            │
│  • DragonKill       │  • Gold             │  • baron_steal          │
│  • Ace              │  • CS               │  • big_purchase         │
└─────────────────────┴─────────────────────┴─────────────────────────┘
```

| Concept | Type | Description |
|---------|------|-------------|
| **Event** | `GameEvent` | Discrete occurrence from the game API |
| **Statistics** | JSON via `get_live_data()` | Polled game state |
| **Moment** | `Moment` | Gamepack signals "something interesting!" |
| **Trigger** | (daemon-managed) | User config for recording moments |

**Key principle**: Your gamepack decides what's interesting (moments). The user decides what gets recorded (triggers).

## Installation

```bash
npm install @companion/pack-protocol
# or
pnpm add @companion/pack-protocol
```

## Usage

### Creating a Game Pack

```typescript
import type { GamePack, BaseMatch } from "@companion/pack-protocol";
import { usePackCache } from "@companion/pack-protocol";
import { MatchCard } from "./components/MatchCard";

// Extend BaseMatch with game-specific fields
interface MyGameMatch extends BaseMatch {
  kills: number;
  deaths: number;
  assists: number;
}

// Define your pack
const pack: GamePack<MyGameMatch> = {
  gameId: 99,
  slug: "my-game",
  MatchCard,
  isMatch: (match): match is MyGameMatch => match.gameId === 99,
};

export default pack;
```

### Using the Cache API

```typescript
import { usePackCache } from "@companion/pack-protocol";

function MyComponent() {
  const cache = usePackCache();

  const loadData = async () => {
    // Check cache first
    const cached = await cache.read<MyData>("data.json");
    if (cached) return cached;

    // Fetch and cache
    const data = await fetchFromNetwork();
    await cache.write("data.json", data);
    return data;
  };
}
```

### Vite Configuration

```typescript
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { companionPack } from "@companion/pack-protocol/vite";

export default defineConfig({
  plugins: [
    react(),
    companionPack({
      packId: "my-game",
      packName: "MyGamePack",
    }),
  ],
});
```

## API Reference

### Types

**Core Pack Types**
- `GamePack<T>` - Interface packs must implement
- `BaseMatch` - Base match type all packs extend
- `MatchCardProps<T>` - Props for match card components
- `LiveMatchCardProps<T>` - Props for live match components
- `PackContext` - Context provided to packs
- `PackCacheAPI` - Sandboxed cache interface

**Data Model Types**
- `GameEvent` - Discrete game event (kills, objectives)
- `Moment` - "Something interesting happened!" signal
- `GameStatus` - Game connection status
- `MatchData` - End-of-match summary data (Summary category)

### Hooks

- `usePackContext()` - Access full pack context
- `usePackCache()` - Access cache API

### Sandbox Runtime

For packs running in sandboxed iframes:

- `initSandboxRuntime()` - Initialize the sandbox bridge
- `getSandboxedCacheAPI()` - Get cache API via postMessage
- `useSandboxedCache()` - Hook-like cache access

### Vite Plugin

- `companionPack(options)` - Configure Vite for pack builds

## Security

Game packs run in sandboxed iframes with limited capabilities:

- ✅ Can cache data (namespaced by game ID)
- ✅ Can make fetch requests
- ✅ Can render React components
- ❌ Cannot access host window
- ❌ Cannot access localStorage
- ❌ Cannot access Electron APIs

## Releasing a Gamepack

Game packs use GitHub Actions to automatically build and publish releases. Here's the workflow:

### 1. Set Up GitHub Actions

Create `.github/workflows/release.yml` in your pack repo:

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build-daemon:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: daemon-darwin-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: daemon-darwin-x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: daemon-linux-x64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: daemon-win32-x64.exe

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
        working-directory: daemon
      - run: mv daemon/target/${{ matrix.target }}/release/daemon* ${{ matrix.artifact }}
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: ${{ matrix.artifact }}

  build-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: pnpm install && pnpm build
        working-directory: frontend
      - uses: actions/upload-artifact@v4
        with:
          name: frontend
          path: frontend/dist/frontend.js

  release:
    needs: [build-daemon, build-frontend]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
      - run: |
          mkdir release && find artifacts -type f -exec mv {} release/ \;
          cd release && sha256sum * > checksums.txt
      - uses: softprops/action-gh-release@v1
        with:
          files: release/*
          generate_release_notes: true
```

### 2. Release Workflow

```bash
# Make changes and commit
git add -A && git commit -m "feat: your changes"

# Update version in config.json
# "version": "0.2.0"
git add config.json && git commit -m "chore: bump version to 0.2.0"

# Create and push tag (triggers CI)
git tag v0.2.0
git push && git push --tags

# Update packs-index with new version
cd ~/Projects/packs-index
# Edit index.json: "version": "0.2.0"
git add -A && git commit -m "chore: update pack to v0.2.0"
git push
```

### 3. How Updates Work

1. Main app fetches `packs-index` to get available packs and versions
2. Compares `installed.version` vs `index.version`
3. Shows "Update" button when versions differ
4. Update reinstalls pack from GitHub release artifacts

## License

MIT
