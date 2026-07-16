package mapgen

import (
	"math"
	"math/rand/v2"
	"sort"
)

func assignElevation(m *GameMap, rng *rand.Rand) {
	dist := make([]int, len(m.Tiles))
	for i := range dist {
		dist[i] = -1
	}
	for i := range m.Tiles {
		m.Tiles[i].Downslope = -1
	}

	isLake := func(idx int) bool {
		t := &m.Tiles[idx]
		return t.IsWater && !t.IsOcean
	}

	var queue []int
	head := 0

	m.Each(func(x, y int, t *Tile) {
		if !t.IsWater {
			return
		}
		idx := m.Index(x, y)
		hasLandNeighbor := false
		for dir := 0; dir < 4; dir++ {
			nidx, ok := N4Neighbor(idx, m.Width, m.Height, dir)
			if ok && !m.Tiles[nidx].IsWater {
				hasLandNeighbor = true
				break
			}
		}
		if hasLandNeighbor {
			dist[idx] = 0
			queue = append(queue, idx)
		}
	})

	minDist := 1
	maxDist := 1

	offsets := [4]int{0, 1, 2, 3}

	for head < len(queue) {
		if head > 0 && head*2 > len(queue) {
			queue = queue[head:]
			head = 0
		}
		idx := queue[head]
		head++

		iOffset := rng.IntN(4)
		for i := 0; i < 4; i++ {
			dir := offsets[(i+iOffset)%4]
			nidx, ok := N4Neighbor(idx, m.Width, m.Height, dir)
			if !ok {
				continue
			}

			lake := isLake(nidx)
			inc := 1
			if lake {
				inc = 0
			}

			newDist := dist[idx] + inc
			if dist[nidx] == -1 || newDist < dist[nidx] {
				dist[nidx] = newDist
				m.Tiles[nidx].Downslope = (dir + 2) % 4

				if m.Tiles[nidx].IsOcean {
					if newDist > minDist {
						minDist = newDist
					}
				} else if !m.Tiles[nidx].IsWater {
					if newDist > maxDist {
						maxDist = newDist
					}
				}

				if lake {
					queue = append([]int{nidx}, queue[head:]...)
					head = 0
				} else {
					queue = append(queue, nidx)
				}
			}
		}
	}

	if minDist == 0 {
		minDist = 1
	}
	if maxDist == 0 {
		maxDist = 1
	}

	for i := range m.Tiles {
		t := &m.Tiles[i]
		d := dist[i]
		if d < 0 {
			t.Elevation = 0
			continue
		}
		if m.Tiles[i].IsOcean {
			t.Elevation = -float64(d) / float64(minDist)
		} else {
			t.Elevation = float64(d) / float64(maxDist)
		}
	}
}

func redistributeElevation(m *GameMap) {
	var landTiles []int
	for i := range m.Tiles {
		if !m.Tiles[i].IsOcean && m.Tiles[i].Elevation > 0 {
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
		return tiles[sorted[i]].Elevation < tiles[sorted[j]].Elevation
	})

	const scale = 1.1
	for i, idx := range sorted {
		y := float64(i) / float64(len(sorted)-1)
		x := math.Sqrt(scale) - math.Sqrt(scale*(1-y))
		if x > 1.0 {
			x = 1.0
		}
		m.Tiles[idx].Elevation = x
	}
}
