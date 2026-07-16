package mapgen

import "math/rand/v2"

const (
	minSpringElevation = 0.3
	maxSpringElevation = 0.9
)

func findSprings(m *GameMap) []int {
	var springs []int
	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.IsWater {
			continue
		}
		if t.Elevation < minSpringElevation || t.Elevation > maxSpringElevation {
			continue
		}
		hasWaterNeighbor := false
		for dir := 0; dir < 4; dir++ {
			nidx, ok := N4Neighbor(i, m.Width, m.Height, dir)
			if ok && m.Tiles[nidx].IsWater {
				hasWaterNeighbor = true
				break
			}
		}
		if !hasWaterNeighbor {
			springs = append(springs, i)
		}
	}
	return springs
}

func randomShuffle(a []int, rng *rand.Rand) {
	for i := len(a) - 1; i > 0; i-- {
		j := rng.IntN(i + 1)
		a[i], a[j] = a[j], a[i]
	}
}

func assignRiverFlow(m *GameMap, springs []int) {
	for _, start := range springs {
		idx := start
		for {
			t := &m.Tiles[idx]
			if t.IsOcean {
				break
			}
			t.IsRiver = true

			parent := t.Downslope
			if parent < 0 {
				break
			}
			nidx, ok := N4Neighbor(idx, m.Width, m.Height, parent)
			if !ok || nidx == idx {
				break
			}
			if m.Tiles[nidx].IsRiver {
				break
			}
			idx = nidx
		}
	}
}

func widenRivers(m *GameMap, maxWidth int) {
	type riverCenter struct {
		x         int
		y         int
		width     int
		downslope int
	}
	var centers []riverCenter

	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.IsRiver && t.RiverWidth > 1 {
			x, y := m.Coord(i)
			centers = append(centers, riverCenter{x: x, y: y, width: t.RiverWidth, downslope: t.Downslope})
		}
	}

	for _, rc := range centers {
		perp1 := (rc.downslope + 1) % 4
		perp2 := (rc.downslope + 3) % 4
		sideCount := (rc.width - 1) / 2

		for side := 1; side <= sideCount; side++ {
			for _, perp := range [2]int{perp1, perp2} {
				nx := rc.x + N4Offsets[perp][0]*side
				ny := rc.y + N4Offsets[perp][1]*side
				if m.InBounds(nx, ny) {
					nt := m.Tile(nx, ny)
					if !nt.IsRiver {
						nt.IsRiver = true
						nt.RiverWidth = 1
					}
				}
			}
		}
	}
}
