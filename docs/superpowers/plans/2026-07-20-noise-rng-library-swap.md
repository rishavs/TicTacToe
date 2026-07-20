# Noise/RNG Library Swap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace custom procedural noise and PRNG helpers with Macroquad capabilities where available and maintained Cargo libraries where Macroquad does not provide the capability, while keeping the current mapgen module structure intact.

**Architecture:** Add small local wrapper functions/types around `noise` and Macroquad's owned RNG so the rest of `src/scenes/mapgen.rs` changes minimally. This first pass may change generated map visuals; structural refactoring is deferred.

**Tech Stack:** Rust 2024, Macroquad 0.4.15, `noise`, `macroquad::rand::RandGenerator`.

## Global Constraints

- Keep `src/scenes/mapgen.rs` as the active mapgen scene for this pass.
- Preserve deterministic behavior for same seed and same options.
- Do not add broad graph/enum/refactor dependencies in this pass.
- Run `cargo fmt` and `cargo test`.

---

### Task 1: Add Library Wrapper Tests

**Files:**
- Modify: `src/scenes/mapgen.rs`

**Interfaces:**
- Produces: tests for `map_random_u32`, `map_random_f32`, `map_random_i32`, `fractal_noise_2d`, and `simplex_fractal_noise_2d`.

- [x] Add tests proving seeded RNG wrappers are deterministic and ranged.
- [x] Add tests proving seeded noise wrappers are deterministic and seed-sensitive.
- [x] Keep wrapper coverage in place while swapping the RNG implementation behind it.

### Task 2: Add Dependencies And Replace Custom Helpers

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/scenes/mapgen.rs`

**Interfaces:**
- Consumes: tests from Task 1.
- Produces: `MapRng = macroquad::rand::RandGenerator`, `map_rng`, `map_random_*`, `fractal_noise_2d`, and `simplex_fractal_noise_2d`.

- [x] Add the `noise` dependency and use Macroquad's existing RNG capability.
- [x] Replace `PmPrng` with `macroquad::rand::RandGenerator`.
- [x] Replace hand-written value/simplex noise internals with the `noise` crate.
- [x] Remove custom helpers made redundant by the library swap.
- [x] Run targeted tests and then the full test suite.

### Task 3: Update Documentation

**Files:**
- Modify: `AGENTS.md`
- Modify: `ARCH.md`
- Modify: `STATUS.md`

**Interfaces:**
- Consumes: implemented library swap.
- Produces: docs that name the active Cargo libraries.

- [x] Document the new mapgen dependency choices.
- [x] Run formatting and full tests.
