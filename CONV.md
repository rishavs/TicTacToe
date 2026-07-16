# Conversation Log

> Terse running log. Trim oldest entries when exceeding ~400 lines.

## 2026-07-14

- **Setup**: Verified LeanSpec MCP connected. Updated AGENTS.md with project instructions (Ebiten turn-based tactics). Created ARCH.md + GDD.md stubs.
- **ebitengine-mcp**: Attempted integration of sedyh/ebitengine-mcp. Hit 3 Windows bugs in the lib (GOROOT, .exe stat, .exe temp output). Patched all. MCP server worked manually but opencode spawning was unreliable on Windows. Outcome: **reverted** — removed dep, patched local copy, opencode config entry. Not viable on Windows with opencode.
- **CONV.md**: Established this log. AGENTS.md updated with CONV.md maintenance rule.

## 2026-07-15

- **Ebiten vs Macroquad**: Researched both. Macroquad is Rust, not Go. Recommended sticking with Ebitengine (larger Go ecosystem, already set up).
- **002 — Main Menu system**: Created spec, implemented scene manager (`src/scene/`), main menu with 5 buttons (New/Mapgen/Battle/Settings/Quit), placeholder scenes with Back button. Added `golang.org/x/image` dep for text rendering. Build verified.
- **ebitenui migration**: Replaced manual `vector.DrawFilledRect` + `inpututil` click detection with `github.com/ebitenui/ebitenui` widgets. Added `resources.go` for shared theme (NineSliceColor buttons, Go font faces). Removed unused deps (`basicfont`, `inpututil`, `vector`). Build & vet clean.
- **Mapgen controls**: Added right panel (80×180) with sliders & checkboxes. Removed ScrollContainer (was crashing). Switched to built-in `themes.GetBasicDarkTheme()` with Press Start 2P font override.
- **Scrollable panel**: Re-added ScrollContainer with proper scrollbar slider (vertical Slider wired via GridLayout 2-col, mouse wheel → ScrolledEvent → slider.Current). Compact 10×10 ebiten images for checkboxes. Back/Regenerate buttons fixed outside scroll area using 3-row GridLayout. Build clean.
- **Slider bugfix**: Content sliders invisible — `PreferredSize()` width=0 for horizontal orientation. Fixed with `RowLayoutData{Stretch: true}` + `MinSize(0, 16)`. User confirmed sliders work.
- **Size reduction**: Panel 80→50px, checkboxes 10×10→6×6, slider heights→8px, scrollbar→6px, font stays 8px (6px blurry).
- **003 — Square-grid map generator**: Spec created, implemented 9 files in `src/mapgen/`. Island via simplex noise + radial falloff, D8 flow accumulation rivers with log2 width, elevation-threshold coast, 9 biomes, camera/viewport. Map viewer scene with biome-colored tiles. Build & vet clean.
- **004 — Close-port mapgen2 terrain pipeline**: Full rewrite of all 9 mapgen files + new `noisyedges.go`. Island formula (`lerp(noise,0.5,round) - (1-inflate)*dist²` with Chebyshev distance), topological coast, lake flat-bottom BFS, BFS-derived N4 downslope, N-spring rivers (elev 0.3–0.9), riverbanks+lakeshores moisture seeds, sqrt falloff, linear moisture redistribution, `1.0 - elevation` temperature, 18-biome mapgen2 classification, BFS randomization, noisy edges recursive subdivision. Removed ElevThreshold, FlowDir, FlowAccum, IsLake from types. New Downslope field, NumRivers config. Build & vet clean.
- **Code review & cleanup**: Replaced O(n²) bubble sorts with `sort.Slice` (stdlib) in elevation+mpoisture. Fixed leaky `queue[1:]` with ring-buffer head index in all 3 BFS files. Deleted dead `edgeLess` in noisyedges.go. Updated ARCH.md, GDD.md, CONV.md, STATUS.md to sync with code.
- **Shallow/deep water**: Split water biomes into DeepOcean/ShallowOcean/DeepLake/ShallowLake. Added `IsShallow` + `assignWaterDepth` (noise-blended elevation threshold `Elevation + noise*0.15 > -0.25`). 20 biomes total. Added to pipeline after elevation.
- **Watershed simulation (reverted)**: Attempted rainfall→flow accumulation→percentile river detection + watershed heatmap rendering. Rivers too few/too small, map broke. Reverted to N-spring approach.
- **Biome collapse (reverted)**: Attempted merging ShallowOcean/DeepLake/ShallowLake into BiomeShallowWater + cliff shores + rivers slider. Map broke. Reverted to 20-biome split water system.

## 2026-07-16

- **Spec reconciliation**: Warmed project state against code. Updated 004 body to current implementation and closed it complete. Updated 005 body to current partial state and marked in-progress: lighting/rendering + noisy-edge data exist; Edges/Fills viewer toggles still pending. Noted LeanSpec MCP content updates work, but status updates did not apply, and `lean-spec` CLI was unavailable on PATH; status frontmatter corrected directly as fallback.
- **005 closed**: User clarified Edges/Fills were tried earlier, gave no benefit, and were removed from code. Updated 005 as a complete closed decision: Biomes/Light remain, Edges/Fills are intentionally not planned. 006 later removed the unused noisy-edge generation path.
- **006 cleanup implemented**: Created cleanup spec and implemented review items. Removed unused noisy-edge generation/file, river width/widening config/path, D8 helpers, and unused camera zoom-at helper. Added shared scene constants, config-backed spring/water-depth/lighting tunables, reused FBM noise generators, extracted map viewer config/input/render helpers, and debounced slider regeneration. `go test ./...` and `go vet ./...` pass.
- **ARCH mapgen logic**: Added dedicated `Mapgen Logic` section to ARCH.md covering `GameMap`, `Tile`, `MapConfig`, the full generation pipeline, and the scene/rendering boundary. Future mapgen changes should update this section.
- **007 QA capture**: Added `--qa-scene`, `--qa-seed`, and `--qa-capture` flags. Agents can launch directly into menu/mapgen/new/battle/settings, save one 320x180 PNG, and exit. Verified menu + mapgen captures, `go test ./...`, and `go vet ./...`. Updated AGENTS.md + ARCH.md with usage guidance; capture folders are gitignored.
- **Local Go cache**: Added `.gocache/` to gitignore and documented that agents should scope `GOCACHE` to the repo before running Go checks, avoiding default user cache writes outside the project.
- **008 hydrology spec**: Created spec for replacing random spring-line rivers with watershed hydrology: explicit basins/lakes, rainfall, flow accumulation, watershed IDs, variable river scale, moisture recomputation, and coherent water rendering.
- **008 hydrology implemented**: Added `hydrology.go` with deterministic rainfall, explicit inland lake basins/outlets, D8 flow directions, flow accumulation, watershed IDs, and flow-threshold rivers with `RiverScale`. Reworked moisture to blend rainfall with river/lake/coast proximity, adjusted water palette/river rendering, and added mapgen hydrology tests. QA capture for seed 42 shows calmer water colors, a visible lake, and fewer flow-based rivers.
- **009 always-on rendering**: Removed MapgenScene Biomes/Light checkboxes and Render label. The map viewer always renders biome colors with lighting. Rebuilt `bin/stoneheart.exe`.
- **Lighting tune**: Raised default mapgen lighting ambient from 0.50 to 0.65 so lit terrain reads brighter while keeping hillshade contrast.
- **010 512 map trial**: Created spec and changed MapgenScene generation size from 200x200 to 512x512 for a larger overworld scale. Lowered camera min zoom to 0.02 so fit-to-view can frame the full map.
- **011 river rendering connectors**: Created/implemented spec to make rivers visually continuous. MapgenScene now draws river center bodies plus connectors along actual upstream/downstream flow relationships and adjacent lake endpoints, preserving variable river scale. Build, vet, and QA capture passed.
- **012 river centerlines**: User noted rivers were still disconnected after the first centerline attempt. Root cause was draw order: river strokes were rendered inside the terrain tile loop, then later terrain tiles erased parts of them. Fixed by drawing terrain first and rivers second as continuous downstream centerline overlay strokes. Hydrology data unchanged.
