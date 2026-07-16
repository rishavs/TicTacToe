package mapgen

func assignTemperature(m *GameMap, biasNorth, biasSouth float64) {
	for i := range m.Tiles {
		t := &m.Tiles[i]
		_, y := m.Coord(i)
		lat := float64(y) / float64(m.Height)
		delta := lerp(biasNorth, biasSouth, lat)
		t.Temperature = 1.0 - t.Elevation + delta
	}
}

func assignBiomes(m *GameMap) {
	for i := range m.Tiles {
		t := &m.Tiles[i]
		ocean := t.IsOcean
		water := t.IsWater
		coast := t.IsCoast
		temp := t.Temperature
		moist := t.Moisture

		if ocean {
			if t.IsShallow {
				t.Biome = BiomeShallowOcean
			} else {
				t.Biome = BiomeDeepOcean
			}
		} else if water {
			if temp > 0.9 {
				t.Biome = BiomeMarsh
			} else if temp < 0.2 {
				t.Biome = BiomeIce
			} else if t.IsShallow {
				t.Biome = BiomeShallowLake
			} else {
				t.Biome = BiomeDeepLake
			}
		} else if coast {
			t.Biome = BiomeBeach
		} else if temp < 0.2 {
			if moist > 0.50 {
				t.Biome = BiomeSnow
			} else if moist > 0.33 {
				t.Biome = BiomeTundra
			} else if moist > 0.16 {
				t.Biome = BiomeBare
			} else {
				t.Biome = BiomeScorched
			}
		} else if temp < 0.4 {
			if moist > 0.66 {
				t.Biome = BiomeTaiga
			} else if moist > 0.33 {
				t.Biome = BiomeShrubland
			} else {
				t.Biome = BiomeTemperateDesert
			}
		} else if temp < 0.7 {
			if moist > 0.83 {
				t.Biome = BiomeTemperateRainForest
			} else if moist > 0.50 {
				t.Biome = BiomeTemperateDeciduousForest
			} else if moist > 0.16 {
				t.Biome = BiomeGrassland
			} else {
				t.Biome = BiomeTemperateDesert
			}
		} else {
			if moist > 0.66 {
				t.Biome = BiomeTropicalRainForest
			} else if moist > 0.33 {
				t.Biome = BiomeTropicalSeasonalForest
			} else if moist > 0.16 {
				t.Biome = BiomeGrassland
			} else {
				t.Biome = BiomeSubtropicalDesert
			}
		}
	}
}
