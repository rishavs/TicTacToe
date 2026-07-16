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
