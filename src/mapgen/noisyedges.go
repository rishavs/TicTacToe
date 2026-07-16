package mapgen

import "math/rand/v2"

type Pt [2]float64

func lerpPt(a, b Pt, t float64) Pt {
	return Pt{a[0]*(1-t) + b[0]*t, a[1]*(1-t) + b[1]*t}
}

type EdgeKey struct {
	X1, Y1 int
	X2, Y2 int
}

type NoisyEdges struct {
	Segments map[EdgeKey][]Pt
}

func (ne *NoisyEdges) subdivide(a, b, p, q Pt, length, amplitude float64, rng *rand.Rand) []Pt {
	dx := a[0] - b[0]
	dy := a[1] - b[1]
	if dx*dx+dy*dy < length*length {
		return []Pt{b}
	}

	ap := lerpPt(a, p, 0.5)
	bp := lerpPt(b, p, 0.5)
	aq := lerpPt(a, q, 0.5)
	bq := lerpPt(b, q, 0.5)

	division := 0.5*(1-amplitude) + rng.Float64()*amplitude
	center := lerpPt(p, q, division)

	r1 := ne.subdivide(a, center, ap, aq, length, amplitude, rng)
	r2 := ne.subdivide(center, b, bp, bq, length, amplitude, rng)

	return append(r1, r2...)
}

func assignNoisyEdges(m *GameMap, rng *rand.Rand) *NoisyEdges {
	ne := &NoisyEdges{Segments: make(map[EdgeKey][]Pt)}
	const subdivLength = 10.0
	const amplitude = 0.2

	for y := 0; y < m.Height; y++ {
		for x := 0; x < m.Width; x++ {
			t := m.Tile(x, y)
			if t.IsOcean {
				continue
			}

			for dir := 0; dir < 4; dir++ {
				nidx, ok := N4Neighbor(m.Index(x, y), m.Width, m.Height, dir)
				if !ok {
					continue
				}
				nx, ny := m.Coord(nidx)
				nt := m.Tile(nx, ny)
				if !nt.IsOcean {
					continue
				}

				a := Pt{float64(x) + 0.5, float64(y) + 0.5}
				b := Pt{float64(nx) + 0.5, float64(ny) + 0.5}
				var p, q Pt
				switch dir {
				case 0:
					p = Pt{float64(x + 1), float64(y)}
					q = Pt{float64(x + 1), float64(y + 1)}
				case 1:
					p = Pt{float64(x + 1), float64(y + 1)}
					q = Pt{float64(x), float64(y + 1)}
				case 2:
					p = Pt{float64(x), float64(y + 1)}
					q = Pt{float64(x), float64(y)}
				case 3:
					p = Pt{float64(x), float64(y)}
					q = Pt{float64(x + 1), float64(y)}
				}

				key := EdgeKey{x, y, nx, ny}
				if _, exists := ne.Segments[key]; !exists {
					ne.Segments[key] = ne.subdivide(a, b, p, q, subdivLength, amplitude, rng)
				}
			}
		}
	}

	return ne
}
