package mapgen

import "github.com/ojrac/opensimplex-go"

func assignOcean(m *GameMap) {
	visited := make([]bool, len(m.Tiles))
	queue := make([]int, 0, m.Width*2+m.Height*2)
	head := 0

	enqueueEdge := func(idx int) {
		if !visited[idx] && m.Tiles[idx].IsWater {
			visited[idx] = true
			queue = append(queue, idx)
		}
	}

	for x := 0; x < m.Width; x++ {
		enqueueEdge(x)
		enqueueEdge((m.Height-1)*m.Width + x)
	}
	for y := 0; y < m.Height; y++ {
		enqueueEdge(y * m.Width)
		enqueueEdge(y*m.Width + m.Width - 1)
	}

	for head < len(queue) {
		idx := queue[head]
		head++
		m.Tiles[idx].IsOcean = true

		for dir := 0; dir < 4; dir++ {
			nidx, ok := N4Neighbor(idx, m.Width, m.Height, dir)
			if !ok {
				continue
			}
			if !visited[nidx] && m.Tiles[nidx].IsWater {
				visited[nidx] = true
				queue = append(queue, nidx)
			}
		}
	}
}

func assignCoast(m *GameMap) {
	m.Each(func(x, y int, t *Tile) {
		if t.IsWater {
			return
		}
		m.EachN4(m.Index(x, y), func(nidx int, _ int) {
			if m.Tiles[nidx].IsOcean {
				t.IsCoast = true
				return
			}
		})
	})
}

func assignWaterDepth(m *GameMap, cfg MapConfig) {
	noise := opensimplex.New(cfg.Seed + 99)
	w := float64(m.Width)
	h := float64(m.Height)

	for i := range m.Tiles {
		t := &m.Tiles[i]
		if !t.IsWater {
			continue
		}
		if !t.IsOcean {
			continue
		}
		x := float64(i % m.Width)
		y := float64(i / m.Width)
		n := noise.Eval2(x/w*cfg.WaterDepthScale, y/h*cfg.WaterDepthScale) * cfg.WaterDepthAmp
		if t.Elevation+n > cfg.WaterDepthLimit {
			t.IsShallow = true
		}
	}
}
