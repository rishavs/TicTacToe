package mapgen

import (
	"math"

	"github.com/ojrac/opensimplex-go"
)

func newFBMGenerators(seed int64, octaves int) []opensimplex.Noise {
	generators := make([]opensimplex.Noise, octaves)
	for i := range generators {
		generators[i] = opensimplex.New(seed + int64(i))
	}
	return generators
}

func fbmNoise(generators []opensimplex.Noise, amplitudes []float64, nx, ny float64) float64 {
	if len(amplitudes) == 0 {
		return 0
	}
	var sum, sumAmps float64
	for i, amp := range amplitudes {
		freq := float64(int(1) << i)
		sum += amp * generators[i].Eval2(nx*freq, ny*freq)
		sumAmps += amp
	}
	return sum / sumAmps
}

func lerp(a, b, t float64) float64 {
	return a*(1-t) + b*t
}

func assignIslandWater(m *GameMap, cfg MapConfig) {
	w := float64(m.Width)
	h := float64(m.Height)
	noise := newFBMGenerators(cfg.Seed+1, len(cfg.NoiseAmplitudes))

	for y := 0; y < m.Height; y++ {
		for x := 0; x < m.Width; x++ {
			nx := 2 * (float64(x)/w - 0.5)
			ny := 2 * (float64(y)/h - 0.5)
			distance := math.Max(math.Abs(nx), math.Abs(ny))

			n := fbmNoise(noise, cfg.NoiseAmplitudes, nx, ny)
			n = lerp(n, 0.5, cfg.IslandRoundness)

			m.Tile(x, y).IsWater = n-(1.0-cfg.IslandInflate)*distance*distance < 0
		}
	}
}
