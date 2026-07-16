---
status: archived
created: 2026-07-14
priority: high
tags:
- tooling
- mcp
- ebitengine
created_at: 2026-07-14T14:52:26.336644800Z
updated_at: 2026-07-15T03:53:04.599729500Z
completed_at: 2026-07-14T14:55:57.933975300Z
transitions:
- status: planned
  at: 2026-07-14T14:52:47.931255200Z
- status: in-progress
  at: 2026-07-14T14:53:24.434984100Z
- status: complete
  at: 2026-07-14T14:55:57.933975300Z
- status: archived
  at: 2026-07-15T03:53:04.599729500Z
---
# Add ebitengine-mcp integration

Integrate [sedyh/ebitengine-mcp](https://github.com/sedyh/ebitengine-mcp) into the project so opencode can inspect and debug the running game.

## What is ebitengine-mcp?

An MCP server that connects to a running Ebiten game, providing tools to:
- Capture build/launch logs and app errors
- Record N frames with M delay in milliseconds
- Inspect game state visually for debugging

## Steps

### 1. Add Go dependency

```
go get github.com/sedyh/ebitengine-mcp@v1.1.0
```

### 2. Wrap the game

In `src/main.go`, import and wrap the game with `mcp.Wrap()`:

```
import "github.com/sedyh/ebitengine-mcp/mcp"

func main() {
  ...
  if err := ebiten.RunGame(mcp.Wrap(&Game{})); err != nil {
    log.Fatal(err)
  }
}
```

### 3. Configure opencode MCP server

Add to `~/.config/opencode/opencode.jsonc` under the `mcp` section:

```json
"ebitengine-mcp": {
  "type": "local",
  "command": ["go", "run", "github.com/sedyh/ebitengine-mcp/cmd/server@v1.1.0"],
  "workdir": "<project-root>"
}
```

### 4. Verify

Restart opencode and confirm the ebitengine-mcp tools are available.

## Notes

- Requires the game to be running for the MCP to capture state
- Works via reverse connection — the server starts the game, the decorator connects back
- `DrawFinalScreen` is supported; `LayoutF` is not
