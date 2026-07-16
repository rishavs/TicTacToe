package mapgen

import (
	"math/rand/v2"
)

func Generate(cfg MapConfig) *GameMap {
	m := NewGameMap(cfg)

	rng := rand.New(rand.NewPCG(uint64(cfg.Seed), uint64(cfg.Seed>>32)))

	assignIslandWater(m, cfg)

	assignOcean(m)

	assignCoast(m)

	assignElevation(m, rng)

	assignWaterDepth(m, cfg.Seed)

	redistributeElevation(m)

	springs := findSprings(m)
	randomShuffle(springs, rng)
	if cfg.NumRivers < len(springs) {
		springs = springs[:cfg.NumRivers]
	}
	assignRiverFlow(m, springs)

	widenRivers(m, cfg.RiverMaxWidth)

	moistureSeeds := findMoistureSeeds(m)
	assignMoisture(m, moistureSeeds)
	redistributeMoisture(m, cfg.MoistureBias, 1+cfg.MoistureBias)

	assignTemperature(m, cfg.NorthTempBias, cfg.SouthTempBias)

	assignBiomes(m)

	assignLighting(m)

	m.NoisyEdges = assignNoisyEdges(m, rng)

	return m
}
