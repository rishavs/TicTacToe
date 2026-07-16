---
status: complete
created: 2026-07-16
tags:
- mapgen
- ui
- rendering
- cleanup
created_at: 2026-07-16T08:45:05.434882Z
updated_at: 2026-07-16T09:54:08.440285100Z
completed_at: 2026-07-16T09:54:08.440285100Z
transitions:
- status: planned
  at: 2026-07-16T09:54:07.882928400Z
- status: in-progress
  at: 2026-07-16T09:54:08.147743700Z
- status: complete
  at: 2026-07-16T09:54:08.440285100Z
---
# Always-on Map Viewer Biomes and Lighting

Remove the map viewer controls for Biomes and Light because both render modes should always be enabled.

## Current Status

Complete. The active code has removed the controls and always renders with biome colors plus lighting.

## Implemented

- Removed the Biomes checkbox.
- Removed the Light checkbox.
- Removed the Render label from the mapgen panel.
- Removed `biomesOn` and `lightingOn` scene state.
- Simplified tile coloring to always use biome colors.
- Always applies `tile.Light`.
- Rebuilt `bin/stoneheart.exe`.

## Problem

The map viewer exposes Biomes and Light checkboxes, but the intended default and desired permanent behavior is to render biome colors with lighting enabled. The controls add panel clutter without supporting a useful workflow.

## Scope

- Remove the Biomes checkbox from the mapgen viewer panel.
- Remove the Light checkbox from the mapgen viewer panel.
- Remove `biomesOn` and `lightingOn` scene state if no longer needed.
- Always render biome colors.
- Always apply tile lighting.
- Update docs/specs that describe Biomes and Light as active viewer toggles.

## Non-Goals

- Do not reintroduce Edge or Fill modes.
- Do not add new debug render modes in this change.
- Do not change the underlying biome or lighting generation algorithms.

## Acceptance Criteria

- The mapgen panel no longer shows Render, Biomes, or Light controls.
- The map always renders with biome colors and lighting.
- `go test ./...` and `go vet ./...` pass with repo-local `GOCACHE`.
- `bin/stoneheart.exe` is rebuilt so the normal launch path reflects the change.
