---
status: complete
created: 2026-07-16
tags:
- mapgen
- terrain
- performance
- viewer
created_at: 2026-07-16T09:06:15.268754200Z
updated_at: 2026-07-16T09:54:09.352338800Z
completed_at: 2026-07-16T09:54:09.352338800Z
transitions:
- status: planned
  at: 2026-07-16T09:54:08.790112900Z
- status: in-progress
  at: 2026-07-16T09:54:09.074268800Z
- status: complete
  at: 2026-07-16T09:54:09.352338800Z
---
# Try 512x512 Map Viewer Generation

Increase the map viewer generation size from 200x200 to 512x512 and verify that the larger overworld scale remains usable.

## Current Status

Complete. The active map viewer now generates 512x512 maps.

## Implemented

- Changed `MapgenScene` map size constants from 200x200 to 512x512.
- Lowered camera minimum zoom from 0.05 to 0.02 so fit-to-view can frame the larger map.
- Updated status/conversation docs.
- Rebuilt `bin/stoneheart.exe`.
- Verified QA capture from the rebuilt executable; seed 42 captured successfully in about 2.1 seconds.

## Problem

The current mapgen viewer uses a 200x200 grid. That is useful for generation tuning, but it is small for a campaign overworld. We want to try 512x512 as the next target scale.

## Scope

- Change `MapgenScene` map size constants from 200x200 to 512x512.
- Keep the underlying `mapgen.DefaultConfig()` size unchanged unless needed.
- Verify fit-to-view still frames the full map.
- Verify regenerate and QA capture still complete.
- Rebuild `bin/stoneheart.exe` so the normal launch path uses the new size.
- Update project docs/status that mention the current viewer map size.

## Non-Goals

- Do not add map-size UI controls yet.
- Do not optimize rendering unless 512x512 exposes a real problem.
- Do not change terrain generation algorithms for this size trial.

## Acceptance Criteria

- Mapgen viewer generates 512x512 maps.
- QA capture works for `--qa-scene mapgen --qa-seed 42`.
- `go test ./...` and `go vet ./...` pass with repo-local `GOCACHE`.
- `bin/stoneheart.exe` is rebuilt.
