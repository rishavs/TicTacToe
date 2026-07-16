---
status: complete
created: 2026-07-16
priority: medium
tags:
- tooling
- qa
- rendering
- scene-management
created_at: 2026-07-16T07:06:35.210633Z
updated_at: 2026-07-16T07:15:40.180434600Z
---

# Dev screenshot capture and direct scene launch

Implement Phase 1 of the game-debugging harness so agents can launch a deterministic scene, capture a rendered PNG, and use that image for visual debugging without relying on ebitengine-mcp.

## Current Status

Complete.

## Implemented

- Added command-line QA flags on the normal executable:
  - `--qa-scene` selects the initial scene.
  - `--qa-seed` controls deterministic mapgen setup.
  - `--qa-capture` writes one rendered PNG and exits.
- Supported scenes:
  - `menu`
  - `mapgen`
  - `new`
  - `battle`
  - `settings`
- Added deterministic mapgen construction via `scene.NewMapgenSceneWithSeed(seed)`.
- Added `captureGame`, which uses the normal scene `Update`/`Draw` path, reads the rendered Ebiten frame with `ReadPixels`, encodes PNG, and exits.
- Normal gameplay launch remains unchanged when QA flags are omitted.
- `--qa-scene` can also be used without `--qa-capture` to launch directly into a scene interactively.
- Added gitignore entries for generated capture folders:
  - `captures/`
  - `.qa-captures/`
  - `.tmp-qa-captures/`
- Updated support docs:
  - `AGENTS.md` now explains when/how agents should use capture mode.
  - `ARCH.md` documents the QA capture harness.
  - `STATUS.md` and `CONV.md` record the capability.

## Usage

```bash
go run ./src --qa-scene mapgen --qa-seed 42 --qa-capture .qa-captures/mapgen.png
```

## Verification

- `go test ./...` passes.
- `go vet ./...` passes.
- Verified menu capture:
  - `go run ./src --qa-scene menu --qa-capture .tmp-qa-captures/menu.png`
  - PNG created at 320x180.
- Verified mapgen capture:
  - `go run ./src --qa-scene mapgen --qa-seed 42 --qa-capture .tmp-qa-captures/mapgen.png`
  - PNG created at 320x180.
- Temporary verification captures were removed after inspection.
