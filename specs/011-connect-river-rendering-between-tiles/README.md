---
status: complete
created: 2026-07-16
tags:
- mapgen
- rendering
- ui
created_at: 2026-07-16T10:32:18.698244900Z
updated_at: 2026-07-16T10:36:27.291640900Z
completed_at: 2026-07-16T10:36:27.291640900Z
transitions:
- status: planned
  at: 2026-07-16T10:32:45.493163900Z
- status: in-progress
  at: 2026-07-16T10:32:45.826116800Z
- status: complete
  at: 2026-07-16T10:36:27.291640900Z
---
# Connect River Rendering Between Tiles

Make river visuals continuous by drawing connector rectangles from each river tile toward connected river/water neighbors.

## Current Status

Complete. River rendering now draws each river tile as a center body plus connector rectangles along actual upstream/downstream flow relationships and lake endpoints.

## Implemented

- Kept hydrology generation unchanged.
- Added `MapgenScene.drawRiver`.
- Added connector rendering from river body toward connected downstream/upstream river neighbors.
- Added connector rendering to adjacent non-ocean water endpoints.
- Preserved existing river color and `RiverScale`.
- Rebuilt `bin/stoneheart.exe`.
- Verified with QA capture for seed 42.

## Problem

Rivers currently render as small centered squares inside river tiles. At zoomed-in views, diagonal and thin river runs look like broken dotted lines, even when the hydrology data is connected.

## Scope

- Keep the hydrology generation unchanged.
- In `MapgenScene` rendering, draw each river tile as a center body plus connectors toward connected neighbors.
- Prefer connections to downstream `FlowDir`, upstream river neighbors, and adjacent lake/ocean water endpoints.
- Preserve variable `RiverScale` so larger rivers remain visually thicker.
- Keep the result pixel-art friendly and square-grid readable.

## Non-Goals

- Do not add river sprites/autotiles yet.
- Do not smooth or resample hydrology paths.
- Do not change river generation thresholds in this spec.

## Acceptance Criteria

- Rivers no longer look like dotted/broken lines at zoomed-in views.
- Existing river color and variable scale are preserved.
- QA capture verifies visibly connected river segments.
- `go test ./...` and `go vet ./...` pass with repo-local `GOCACHE`.
- `bin/stoneheart.exe` is rebuilt.
