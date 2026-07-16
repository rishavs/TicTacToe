package mapgen

import "testing"

func TestFlowAccumulationCreatesRiverFromTributaries(t *testing.T) {
	cfg := DefaultConfig()
	cfg.Width = 5
	cfg.Height = 5
	cfg.RiverThreshold = 4
	cfg.RiverMaxScale = 3

	m := NewGameMap(cfg)
	for y := 0; y < m.Height; y++ {
		for x := 0; x < m.Width; x++ {
			tile := m.Tile(x, y)
			if y == m.Height-1 {
				tile.IsWater = true
				tile.IsOcean = true
				tile.Elevation = -1
				tile.Rainfall = 1
				continue
			}
			tile.Elevation = float64(m.Height-1-y)*0.25 + float64(absInt(x-2))*0.04
			tile.Rainfall = 1
		}
	}

	assignFlowDirections(m)
	assignFlowAccumulation(m, cfg)
	assignWatersheds(m)
	assignRivers(m, cfg)

	mouth := m.Index(2, 3)
	if m.Tiles[mouth].Flow <= cfg.RiverThreshold {
		t.Fatalf("expected tributaries to accumulate at outlet tile, got flow %.2f", m.Tiles[mouth].Flow)
	}
	if !m.Tiles[mouth].IsRiver {
		t.Fatalf("expected outlet tile to become a river")
	}
	if m.Tiles[mouth].RiverScale < 1 {
		t.Fatalf("expected river scale to be assigned, got %.2f", m.Tiles[mouth].RiverScale)
	}
	if len(m.Watersheds) == 0 {
		t.Fatalf("expected watershed assignment")
	}
}

func TestGenerateCreatesHydrologyLayers(t *testing.T) {
	cfg := DefaultConfig()
	cfg.Width = 96
	cfg.Height = 96
	cfg.Seed = 42
	cfg.MaxLakes = 6
	cfg.RiverThreshold = 55

	m := Generate(cfg)

	rivers := 0
	maxFlow := 0.0
	missingWatershed := 0
	for i := range m.Tiles {
		tile := &m.Tiles[i]
		if tile.IsRiver {
			rivers++
		}
		if tile.Flow > maxFlow {
			maxFlow = tile.Flow
		}
		if !tile.IsOcean && tile.WatershedID < 0 {
			missingWatershed++
		}
	}

	if len(m.Lakes) == 0 {
		t.Fatalf("expected generated map to include explicit lakes")
	}
	if len(m.Watersheds) == 0 {
		t.Fatalf("expected generated map to include watersheds")
	}
	if rivers == 0 {
		t.Fatalf("expected generated map to include flow-threshold rivers")
	}
	if maxFlow <= cfg.RiverThreshold {
		t.Fatalf("expected max flow to exceed river threshold, got %.2f", maxFlow)
	}
	if missingWatershed > 0 {
		t.Fatalf("expected every non-ocean tile to have a watershed, missing %d", missingWatershed)
	}
}

func absInt(v int) int {
	if v < 0 {
		return -v
	}
	return v
}
