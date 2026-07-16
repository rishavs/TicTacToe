package mapgen

import (
	"math"
	"sort"

	"github.com/ojrac/opensimplex-go"
)

type lakeCandidate struct {
	idx   int
	score float64
}

type watershedKey struct {
	kind int
	id   int
}

const (
	watershedOcean = iota
	watershedLake
	watershedBasin
)

func assignHydrology(m *GameMap, cfg MapConfig) {
	resetHydrology(m)
	assignRainfall(m, cfg)
	labelExistingLakes(m)
	assignLakes(m, cfg)
	assignFlowDirections(m)
	assignFlowAccumulation(m, cfg)
	assignWatersheds(m)
	assignRivers(m, cfg)
}

func resetHydrology(m *GameMap) {
	m.Lakes = nil
	m.Watersheds = nil
	for i := range m.Tiles {
		t := &m.Tiles[i]
		t.Rainfall = 0
		t.Flow = 0
		t.RiverScale = 0
		t.FlowDir = -1
		t.WatershedID = -1
		t.LakeID = -1
		t.IsRiver = false
	}
}

func assignRainfall(m *GameMap, cfg MapConfig) {
	noise := opensimplex.New(cfg.Seed + 211)
	w := float64(m.Width)
	h := float64(m.Height)

	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.IsOcean {
			t.Rainfall = 1
			continue
		}

		x, y := m.Coord(i)
		nx := float64(x) / w * cfg.RainfallScale
		ny := float64(y) / h * cfg.RainfallScale
		n := (noise.Eval2(nx, ny) + 1) * 0.5

		lat := float64(y) / h
		equator := 1 - math.Abs(lat-0.5)*2
		elevationPenalty := math.Max(0, t.Elevation) * 0.35
		rain := 0.35 + cfg.RainfallAmp*n + 0.25*equator - elevationPenalty
		t.Rainfall = clamp(rain, 0.05, 1.0)
	}
}

func assignLakes(m *GameMap, cfg MapConfig) {
	candidates := lakeCandidates(m)
	sort.Slice(candidates, func(i, j int) bool {
		return candidates[i].score > candidates[j].score
	})

	blocked := make([]bool, len(m.Tiles))
	for _, c := range candidates {
		if len(m.Lakes) >= cfg.MaxLakes {
			break
		}
		if blocked[c.idx] || m.Tiles[c.idx].IsWater {
			continue
		}

		basin := growLakeBasin(m, c.idx, cfg, blocked)
		if len(basin) < cfg.MinLakeArea {
			continue
		}

		id := len(m.Lakes)
		surface := lakeSurfaceLevel(m, basin)
		for _, idx := range basin {
			t := &m.Tiles[idx]
			t.IsWater = true
			t.IsOcean = false
			t.IsCoast = false
			t.LakeID = id
			t.IsShallow = surface-t.Elevation < cfg.LakeDepth*0.55
		}

		outlet := lakeOutlet(m, basin)
		m.Lakes = append(m.Lakes, Lake{
			ID:           id,
			TileCount:    len(basin),
			SurfaceLevel: surface,
			Outlet:       outlet,
		})

		blockLakeSpacing(m, basin, cfg.LakeSpacing, blocked)
	}
}

func labelExistingLakes(m *GameMap) {
	visited := make([]bool, len(m.Tiles))
	for i := range m.Tiles {
		if visited[i] || !m.Tiles[i].IsWater || m.Tiles[i].IsOcean {
			continue
		}

		queue := []int{i}
		visited[i] = true
		basin := make([]int, 0)
		for len(queue) > 0 {
			idx := queue[0]
			queue = queue[1:]
			basin = append(basin, idx)

			m.EachN4(idx, func(nidx int, _ int) {
				if visited[nidx] || !m.Tiles[nidx].IsWater || m.Tiles[nidx].IsOcean {
					return
				}
				visited[nidx] = true
				queue = append(queue, nidx)
			})
		}

		id := len(m.Lakes)
		surface := lakeSurfaceLevel(m, basin)
		for _, idx := range basin {
			m.Tiles[idx].LakeID = id
		}
		m.Lakes = append(m.Lakes, Lake{
			ID:           id,
			TileCount:    len(basin),
			SurfaceLevel: surface,
			Outlet:       lakeOutlet(m, basin),
		})
	}
}

func lakeCandidates(m *GameMap) []lakeCandidate {
	candidates := make([]lakeCandidate, 0)
	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.IsWater || t.IsCoast || t.Elevation < 0.08 || t.Elevation > 0.62 {
			continue
		}

		higher := 0
		total := 0
		for dir := 0; dir < 8; dir++ {
			nidx, ok := D8Neighbor(i, m.Width, m.Height, dir)
			if !ok {
				continue
			}
			total++
			if m.Tiles[nidx].Elevation > t.Elevation {
				higher++
			}
		}
		if total == 0 {
			continue
		}

		localLow := float64(higher) / float64(total)
		score := t.Rainfall*1.5 + (1-t.Elevation)*0.7 + localLow*0.8
		if localLow < 0.35 {
			score *= 0.5
		}
		candidates = append(candidates, lakeCandidate{idx: i, score: score})
	}
	return candidates
}

func growLakeBasin(m *GameMap, start int, cfg MapConfig, blocked []bool) []int {
	limit := m.Tiles[start].Elevation + cfg.LakeDepth
	seen := make(map[int]bool, cfg.MaxLakeArea)
	queue := []int{start}
	seen[start] = true
	basin := make([]int, 0, cfg.MaxLakeArea)

	for len(queue) > 0 && len(basin) < cfg.MaxLakeArea {
		idx := queue[0]
		queue = queue[1:]

		t := &m.Tiles[idx]
		if blocked[idx] || t.IsWater || t.IsOcean || t.IsCoast || t.Elevation > limit {
			continue
		}
		basin = append(basin, idx)

		for dir := 0; dir < 8; dir++ {
			nidx, ok := D8Neighbor(idx, m.Width, m.Height, dir)
			if !ok || seen[nidx] {
				continue
			}
			seen[nidx] = true
			queue = append(queue, nidx)
		}
	}
	return basin
}

func lakeSurfaceLevel(m *GameMap, basin []int) float64 {
	surface := m.Tiles[basin[0]].Elevation
	for _, idx := range basin[1:] {
		if m.Tiles[idx].Elevation > surface {
			surface = m.Tiles[idx].Elevation
		}
	}
	return surface
}

func lakeOutlet(m *GameMap, basin []int) int {
	inLake := make(map[int]bool, len(basin))
	for _, idx := range basin {
		inLake[idx] = true
	}

	outlet := -1
	outletElev := math.Inf(1)
	for _, idx := range basin {
		for dir := 0; dir < 8; dir++ {
			nidx, ok := D8Neighbor(idx, m.Width, m.Height, dir)
			if !ok || inLake[nidx] {
				continue
			}
			t := &m.Tiles[nidx]
			elev := t.Elevation
			if t.IsOcean {
				elev = -1
			}
			if elev < outletElev {
				outletElev = elev
				outlet = nidx
			}
		}
	}
	return outlet
}

func blockLakeSpacing(m *GameMap, basin []int, spacing int, blocked []bool) {
	if spacing <= 0 {
		return
	}
	radius2 := spacing * spacing
	for _, center := range basin {
		cx, cy := m.Coord(center)
		for y := max(0, cy-spacing); y < min(m.Height, cy+spacing+1); y++ {
			for x := max(0, cx-spacing); x < min(m.Width, cx+spacing+1); x++ {
				dx := x - cx
				dy := y - cy
				if dx*dx+dy*dy <= radius2 {
					blocked[m.Index(x, y)] = true
				}
			}
		}
	}
}

func assignFlowDirections(m *GameMap) {
	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.IsOcean || t.IsWater {
			t.FlowDir = -1
			continue
		}

		bestDir := -1
		bestElev := effectiveWaterElevation(m, i)
		for dir := 0; dir < 8; dir++ {
			nidx, ok := D8Neighbor(i, m.Width, m.Height, dir)
			if !ok {
				continue
			}
			if drainsBackIntoOutletLake(m, i, nidx) {
				continue
			}
			elev := effectiveWaterElevation(m, nidx)
			if m.Tiles[nidx].IsWater || elev < bestElev {
				bestElev = elev
				bestDir = dir
			}
		}

		if bestDir >= 0 {
			t.FlowDir = bestDir
			continue
		}
		if t.Downslope >= 0 {
			t.FlowDir = n4ToD8(t.Downslope)
		}
	}
}

func effectiveWaterElevation(m *GameMap, idx int) float64 {
	t := &m.Tiles[idx]
	if t.IsOcean {
		return -1
	}
	if t.IsWater && t.LakeID >= 0 && t.LakeID < len(m.Lakes) {
		return m.Lakes[t.LakeID].SurfaceLevel
	}
	return t.Elevation
}

func drainsBackIntoOutletLake(m *GameMap, idx, nidx int) bool {
	nt := &m.Tiles[nidx]
	if !nt.IsWater || nt.IsOcean || nt.LakeID < 0 || nt.LakeID >= len(m.Lakes) {
		return false
	}
	return m.Lakes[nt.LakeID].Outlet == idx
}

func n4ToD8(dir int) int {
	switch dir {
	case 0:
		return 0
	case 1:
		return 2
	case 2:
		return 4
	case 3:
		return 6
	default:
		return -1
	}
}

func assignFlowAccumulation(m *GameMap, cfg MapConfig) {
	upstream := make([][]int, len(m.Tiles))
	for i := range m.Tiles {
		if m.Tiles[i].IsOcean {
			continue
		}
		dst := downstreamIndex(m, i)
		if dst >= 0 && dst != i {
			upstream[dst] = append(upstream[dst], i)
		}
	}

	state := make([]int, len(m.Tiles))
	var accumulate func(int) float64
	accumulate = func(idx int) float64 {
		if state[idx] == 2 {
			return m.Tiles[idx].Flow
		}
		if state[idx] == 1 {
			return m.Tiles[idx].Rainfall
		}
		state[idx] = 1

		flow := m.Tiles[idx].Rainfall
		for _, up := range upstream[idx] {
			flow += accumulate(up)
		}
		m.Tiles[idx].Flow = flow
		if m.Tiles[idx].LakeID >= 0 && m.Tiles[idx].LakeID < len(m.Lakes) {
			m.Lakes[m.Tiles[idx].LakeID].Flow += flow
		}

		state[idx] = 2
		return flow
	}

	for i := range m.Tiles {
		if !m.Tiles[i].IsOcean {
			accumulate(i)
		}
	}

	for i := range m.Lakes {
		m.Lakes[i].Flow = 0
	}
	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.LakeID >= 0 && t.LakeID < len(m.Lakes) {
			m.Lakes[t.LakeID].Flow += t.Flow
		}
	}
}

func downstreamIndex(m *GameMap, idx int) int {
	t := &m.Tiles[idx]
	if t.IsOcean {
		return -1
	}
	if t.IsWater && t.LakeID >= 0 && t.LakeID < len(m.Lakes) {
		return m.Lakes[t.LakeID].Outlet
	}
	if t.FlowDir < 0 {
		return -1
	}
	nidx, ok := D8Neighbor(idx, m.Width, m.Height, t.FlowDir)
	if !ok {
		return -1
	}
	return nidx
}

func assignWatersheds(m *GameMap) {
	keyToID := make(map[watershedKey]int)
	watersheds := make([]Watershed, 0)

	for i := range m.Tiles {
		if m.Tiles[i].IsOcean {
			continue
		}
		key, outlet, lakeID := watershedTerminal(m, i)
		id, ok := keyToID[key]
		if !ok {
			id = len(watersheds)
			keyToID[key] = id
			watersheds = append(watersheds, Watershed{
				ID:     id,
				Outlet: outlet,
				LakeID: lakeID,
			})
		}
		m.Tiles[i].WatershedID = id
		watersheds[id].Area++
		if m.Tiles[i].Flow > watersheds[id].Flow {
			watersheds[id].Flow = m.Tiles[i].Flow
		}
	}
	m.Watersheds = watersheds
}

func watershedTerminal(m *GameMap, idx int) (watershedKey, int, int) {
	seen := make(map[int]bool)
	cur := idx
	for {
		if cur < 0 || cur >= len(m.Tiles) {
			return watershedKey{kind: watershedBasin, id: idx}, idx, -1
		}
		t := &m.Tiles[cur]
		if t.IsOcean {
			return watershedKey{kind: watershedOcean, id: cur}, cur, -1
		}
		if t.IsWater && t.LakeID >= 0 {
			return watershedKey{kind: watershedLake, id: t.LakeID}, cur, t.LakeID
		}
		if seen[cur] {
			return watershedKey{kind: watershedBasin, id: cur}, cur, -1
		}
		seen[cur] = true

		next := downstreamIndex(m, cur)
		if next < 0 {
			return watershedKey{kind: watershedBasin, id: cur}, cur, -1
		}
		cur = next
	}
}

func assignRivers(m *GameMap, cfg MapConfig) {
	if cfg.RiverThreshold <= 0 {
		return
	}
	logBase := math.Log(2)
	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.IsWater || t.IsOcean || t.Flow < cfg.RiverThreshold {
			continue
		}
		t.IsRiver = true
		scale := 1 + math.Log(t.Flow/cfg.RiverThreshold)/logBase
		t.RiverScale = clamp(scale, 1, cfg.RiverMaxScale)
	}
}

func clamp(v, lo, hi float64) float64 {
	if v < lo {
		return lo
	}
	if v > hi {
		return hi
	}
	return v
}
