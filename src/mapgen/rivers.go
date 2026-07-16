package mapgen

import "math/rand/v2"

func findSprings(m *GameMap, cfg MapConfig) []int {
	var springs []int
	for i := range m.Tiles {
		t := &m.Tiles[i]
		if t.IsWater {
			continue
		}
		if t.Elevation < cfg.MinSpringElev || t.Elevation > cfg.MaxSpringElev {
			continue
		}
		hasWaterNeighbor := false
		m.EachN4(i, func(nidx int, _ int) {
			if m.Tiles[nidx].IsWater {
				hasWaterNeighbor = true
			}
		})
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
