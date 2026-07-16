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

	redistributeElevation(m)

	assignHydrology(m, cfg)

	assignWaterDepth(m, cfg)

	assignMoisture(m, cfg)
	redistributeMoisture(m, cfg.MoistureBias, 1+cfg.MoistureBias)

	assignTemperature(m, cfg.NorthTempBias, cfg.SouthTempBias)

	assignBiomes(m)

	assignLighting(m, cfg)

	return m
}
