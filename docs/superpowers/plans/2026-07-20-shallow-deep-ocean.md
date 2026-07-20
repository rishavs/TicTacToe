# Shallow / Deep Ocean Split Plan

Date: 2026-07-20

## Goal

Split the single ocean biome into shallow and deep ocean. Shallow ocean should broadly follow the island coastline without tracing it exactly.

## Design

- Keep ocean classification deterministic for the same seed and map options.
- Reuse the existing map graph instead of adding a separate mask image.
- Assign ocean distance with a breadth-first search starting from ocean centers adjacent to non-water land.
- Mark near-coast ocean as shallow.
- Add deterministic coordinate jitter at the outer shallow boundary so the shelf feels organic but remains reproducible.
- Connect disconnected islands back to the mainland by converting only the shortest needed non-border deep-ocean path into shallow ocean.
- Keep the change classification-only for now. Movement, combat, resources, and other game rules are not implemented yet.

## Implementation Checklist

- [x] Add failing tests proving the ocean splits into shallow and deep categories.
- [x] Add tests proving shallow ocean stays closer to land than deep ocean.
- [x] Store ocean distance and shallow-ocean state on centers.
- [x] Assign ocean depth after ocean/coast/land detection and before biome histograms.
- [x] Promote enclosed deep-ocean pockets to shallow when they are fully surrounded by shallow ocean.
- [x] Preserve border-connected open deep ocean during shallow cleanup.
- [x] Add tests proving landmasses are connected through land or shallow ocean.
- [x] Add a deterministic shallow bridge post-process from disconnected islands to the mainland.
- [x] Preserve border-connected deep ocean after shallow bridge cleanup.
- [x] Render shallow ocean with a separate palette color.
- [x] Update architecture, design, and status docs.
- [x] Run final format, compile, tests, and visual capture.

## Verification

To complete this pass:

- `cargo fmt`
- `cargo check`
- `cargo test`
- Capture the mapgen scene in biome view and inspect the resulting image.
