package mapgen

import (
	"math"
	"sort"
)

func findMoistureSeeds(m *GameMap) []int {
	seedSet := make(map[int]bool)

	for i := range m.Tiles {
		t := &m.Tiles[i]

		if t.IsRiver {
			for dir := 0; dir < 4; dir++ {
				nidx, ok := N4Neighbor(i, m.Width, m.Height, dir)
				if ok {
					seedSet[nidx] = true
				}
			}
		}

		if t.IsWater && !t.IsOcean {
			seedSet[i] = true
			for dir := 0; dir < 4; dir++ {
				nidx, ok := N4Neighbor(i, m.Width, m.Height, dir)
				if ok {
					seedSet[nidx] = true
				}
			}
		}
	}

	seeds := make([]int, 0, len(seedSet))
	for s := range seedSet {
		seeds = append(seeds, s)
	}
	return seeds
}

func assignMoisture(m *GameMap, seeds []int) {
	dist := make([]int, len(m.Tiles))
	for i := range dist {
		dist[i] = -1
	}

	for i := range m.Tiles {
		if m.Tiles[i].IsWater {
			m.Tiles[i].Moisture = 1.0
		}
	}

	var queue []int
	for _, s := range seeds {
		if dist[s] == -1 && !m.Tiles[s].IsWater {
			dist[s] = 0
			queue = append(queue, s)
		}
	}

	head := 0
	maxDist := 1
	for head < len(queue) {
		idx := queue[head]
		head++
		d := dist[idx]

		for dir := 0; dir < 4; dir++ {
			nidx, ok := N4Neighbor(idx, m.Width, m.Height, dir)
			if !ok {
				continue
			}
			if m.Tiles[nidx].IsWater {
				continue
			}
			if dist[nidx] != -1 {
				continue
			}
			nd := d + 1
			dist[nidx] = nd
			if nd > maxDist {
				maxDist = nd
			}
			queue = append(queue, nidx)
		}
	}

	if maxDist == 0 {
		maxDist = 1
	}

	for i := range m.Tiles {
		if m.Tiles[i].IsWater {
			continue
		}
		d := dist[i]
		if d < 0 {
			m.Tiles[i].Moisture = 0
			continue
		}
		m.Tiles[i].Moisture = 1.0 - math.Pow(float64(d)/float64(maxDist), 0.5)
	}
}

func redistributeMoisture(m *GameMap, minMoisture, maxMoisture float64) {
	var landTiles []int
	for i := range m.Tiles {
		if !m.Tiles[i].IsWater {
			landTiles = append(landTiles, i)
		}
	}
	if len(landTiles) < 2 {
		return
	}

	sorted := make([]int, len(landTiles))
	copy(sorted, landTiles)
	tiles := m.Tiles
	sort.Slice(sorted, func(i, j int) bool {
		return tiles[sorted[i]].Moisture < tiles[sorted[j]].Moisture
	})

	for i, idx := range sorted {
		m.Tiles[idx].Moisture = minMoisture + (maxMoisture-minMoisture)*float64(i)/float64(len(sorted)-1)
	}
}
