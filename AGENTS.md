# stoneheart

> A turn-based tactics game built with Ebiten engine in Go.

## Project Documentation

- **ARCH.md** — Overall architecture of the game (systems, packages, data flow). Keep updated as development progresses.
- **GDD.md** — Game Design Document (mechanics, logic, rules, game loop, combat formulas, units, maps). Keep updated as design decisions are made.
- **CONV.md** — Terse running log of conversations and outcomes. Read on session start for context. Keep under ~400 lines; trim oldest entries when exceeded.
- **STATUS.md** — Running checklist of completed work and pending todos. Update after every meaningful change.

## Local Go Cache

Keep Go build/test scratch output inside the project folder during agent work. Before running `go test`, `go vet`, `go build`, or `go run`, set `GOCACHE` to `.gocache` for that shell session/command.

PowerShell example:

```powershell
$env:GOCACHE = "$PWD\.gocache"; go test ./...
```

Guidelines:
- Prefer the repo-local `.gocache/` over the default user cache outside the project.
- `.gocache/` is ignored by git and should never be committed.
- Do not use `go env -w` for this; keep the setting scoped to the current command/session.
- Use an existing binary from `bin/` when the task only needs to launch the game and does not need a rebuild.

## QA Capture / Visual Debugging

Use the built-in QA capture mode when you need to inspect the game visually, debug a render issue, or verify that a scene launches correctly. Prefer this over OS screenshots or ebitengine-mcp.

```bash
go run ./src --qa-scene mapgen --qa-seed 42 --qa-capture .qa-captures/mapgen.png
```

Supported scenes: `menu`, `mapgen`, `new`, `battle`, `settings`.

Guidelines:
- Use `--qa-scene <name>` to launch directly into the scene under review.
- Use `--qa-seed <n>` for deterministic mapgen captures.
- Use `--qa-capture <path>` to save one rendered 320x180 PNG and exit.
- Keep captures in `.qa-captures/`, `.tmp-qa-captures/`, or `captures/`; these folders are ignored by git.
- Do not commit captures unless the user explicitly asks for visual artifacts.
- After capturing, inspect images directly and use them to guide debugging/fixes.

## Design Principles

| Principle | Details |
|-----------|---------|
| **Libraries first** | Prefer well-done, updated, popular Go libraries over writing from scratch. |
| **Separate logic & rendering** | Game logic (rules, state, combat) must never depend on rendering code. Rendering reads game state; game state never knows about pixels. |
| **Data-driven design** | Use config files / data structures for gameplay values (stats, abilities, maps). Enables future mod support — treat data as content, not code. |
| **Decoupled entities** | Units, items, abilities, terrain — each in its own package with minimal cross-deps. Use interfaces at boundaries. |
| **Simple & elegant** | Err on the side of less code, fewer packages, flatter hierarchies. A human has to maintain this. No cleverness for its own sake. |

### Additional Standards

| Guideline | Details |
|-----------|---------|
| **Testable logic** | Combat, pathfinding, AI — all must be testable without Ebiten running. Pure functions over methods on big structs. |
| **State as value** | Game state is a single snapshot struct passed by value. Mutations return new state. Enables save/load, replay, undo. |
| **No init() magic** | Explicit initialization over `init()` functions. AI agents lose track of implicit init ordering. |
| **Config over convention** | Every tunable number lives in a config struct/file. No hardcoded values, even "temporary" ones. |
| **One-way data flow** | Input → Logic → State → Render. No back-channels. No render callbacks mutating state. |
| **Small interfaces** | 1–3 methods per interface. Named for what they consume (e.g., `MovementGrid`), not what implements them. |
| **No global state** | No package-level `var` for game state. Pass context or state structs explicitly. |
| **Asset pipeline** | Assets loaded once at startup into read-only caches. No filesystem reads during the game loop. |

### Tactics-Specific Standards

| Guideline | Details |
|-----------|---------|
| **Deterministic logic** | All game logic must be deterministic — same seed + same inputs = same outcome. Foundation for replays, AI debugging, and networked play. |
| **Compute off main loop** | AI turns, pathfinding, simulation — run in separate goroutines. Render loop must never block; show progress/animations while computing. |
| **Action validation** | Every action validates preconditions, computes result, then applies. Never assume validity. Failed validations are bugs, not user errors. |
| **Turn state machine** | Formal turn/phase FSM. Predictable order: start-turn hooks → player input → execute actions → AI turn → end-turn hooks → next round. |
| **Immutable event log** | Log every action as an immutable event entry. Enables replay debugging, undo, and delta-based save tracking without extra work. |
| **Profile, don't guess** | Tactics AI and pathfinding will dominate CPU. Set early perf budgets. Profile before optimizing. No premature complexity. |
| **FSM-first design** | Model game objects (units, abilities, AI, UI panels, turns) as Finite State Machines wherever applicable. FSMs make behavior explicit, testable, and prevent invalid state transitions. Use a lightweight FSM library (e.g., `github.com/looplab/fsm`) rather than hand-rolling state switches. |

## 🚨 CRITICAL: Before ANY Task

**STOP and check these first:**

1. **Discover context** → Use `board` tool to see project state
2. **Search for related work** → Use `search` tool before creating new specs
3. **Never create files manually** → Always use `create` tool for new specs
4. **Never implement without approval** → Creating a spec is NOT a signal to implement. Wait for explicit "implement" or "execute" confirmation before writing any code.

> **Why?** Skipping discovery creates duplicate work. Manual file creation breaks LeanSpec tooling. Premature implementation bypasses the design review step, wastes work if requirements change, and leaves specs out of sync with code.

## 🔧 Managing Specs

### MCP Tools (Preferred) with CLI Fallback

| Action         | MCP Tool   | CLI Fallback                                   |
| -------------- | ---------- | ---------------------------------------------- |
| Project status | `board`    | `lean-spec board`                              |
| List specs     | `list`     | `lean-spec list`                               |
| Search specs   | `search`   | `lean-spec search "query"`                     |
| View spec      | `view`     | `lean-spec view <spec>`                        |
| Create spec    | `create`   | `lean-spec create <name>`                      |
| Update spec    | `update`   | `lean-spec update <spec> --status <status>`    |
| Link specs     | `link`     | `lean-spec link <spec> --depends-on <other>`   |
| Unlink specs   | `unlink`   | `lean-spec unlink <spec> --depends-on <other>` |
| Dependencies   | `deps`     | `lean-spec deps <spec>`                        |
| Token count    | `tokens`   | `lean-spec tokens <spec>`                      |
| Validate specs | `validate` | `lean-spec validate`                           |

## ⚠️ Core Rules

| Rule                                | Details                                                                                                               |
| ----------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| **NEVER edit frontmatter manually** | Use `update`, `link`, `unlink` for: `status`, `priority`, `tags`, `assignee`, `transitions`, timestamps, `depends_on` |
| **ALWAYS link spec references**     | Content mentions another spec → `lean-spec link <spec> --depends-on <other>`                                          |
| **Track status transitions**        | `planned` → `in-progress` (before coding) → `complete` (after done)                                                   |
| **Keep specs current**              | Document progress, decisions, and learnings as work happens. Obsolete specs mislead both humans and AI                |
| **No nested code blocks**           | Use indentation instead                                                                                               |

### 🚫 Common Mistakes

| ❌ Don't                             | ✅ Do Instead                                |
| ----------------------------------- | ------------------------------------------- |
| Create spec files manually          | Use `create` tool                           |
| Skip discovery                      | Run `board` and `search` first              |
| Leave status as "planned"           | Update to `in-progress` before coding       |
| Edit frontmatter manually           | Use `update` tool                           |
| Complete spec without documentation | Document progress, prompts, learnings first |

## 📋 SDD Workflow

```
BEFORE: board → search → check existing specs
DURING: update status to in-progress → code → document decisions → link dependencies
        → update ARCH.md + GDD.md as architecture/design evolves
AFTER:  document completion → update status to complete
        → ensure ARCH.md and GDD.md reflect final state
```

**Status tracks implementation, NOT spec writing.**

**ARCH.md and GDD.md are living documents.** After every meaningful code change:
- Update ARCH.md when packages, systems, or data flow change
- Update GDD.md when game mechanics, rules, or design decisions are made
- Update STATUS.md (tick completed items, add new todos, keep it current)

## Spec Dependencies

Use `depends_on` to express blocking relationships between specs:
- **`depends_on`** = True blocker, work order matters, directional (A depends on B)

Link dependencies when one spec builds on another:
```bash
lean-spec link <spec> --depends-on <other-spec>
```

## When to Use Specs

| ✅ Write spec        | ❌ Skip spec                |
| ------------------- | -------------------------- |
| Multi-part features | Bug fixes                  |
| Breaking changes    | Trivial changes            |
| Design decisions    | Self-explanatory refactors |

## Token Thresholds

| Tokens      | Status               |
| ----------- | -------------------- |
| <2,000      | ✅ Optimal            |
| 2,000-3,500 | ✅ Good               |
| 3,500-5,000 | ⚠️ Consider splitting |
| >5,000      | 🔴 Must split         |

## Quality Validation

Before completing work, validate spec quality:
```bash
lean-spec validate              # Check structure and quality
lean-spec validate --check-deps # Verify dependency alignment
```

Validation checks:
- Missing required sections
- Excessive length (>400 lines)
- Content/frontmatter dependency misalignment
- Invalid frontmatter fields

## First Principles (Priority Order)

1. **Context Economy** - <2,000 tokens optimal, >3,500 needs splitting
2. **Signal-to-Noise** - Every word must inform a decision
3. **Intent Over Implementation** - Capture why, let how emerge
4. **Bridge the Gap** - Both human and AI must understand
5. **Progressive Disclosure** - Add complexity only when pain is felt

---

**Remember:** LeanSpec tracks what you're building. Keep specs in sync with your work!
