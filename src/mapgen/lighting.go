package mapgen

func assignLighting(m *GameMap) {
	for y := 1; y < m.Height-1; y++ {
		for x := 1; x < m.Width-1; x++ {
			t := m.Tile(x, y)
			if t.IsWater {
				t.Light = 1.0
				continue
			}
			dx := m.Tile(x+1, y).Elevation - m.Tile(x-1, y).Elevation
			dy := m.Tile(x, y-1).Elevation - m.Tile(x, y+1).Elevation
			l := 0.5 + (dx+dy)*2.0
			if l < 0.2 {
				l = 0.2
			}
			if l > 1.0 {
				l = 1.0
			}
			t.Light = l
		}
	}

	for x := 0; x < m.Width; x++ {
		m.Tile(x, 0).Light = 1.0
		m.Tile(x, m.Height-1).Light = 1.0
	}
	for y := 0; y < m.Height; y++ {
		m.Tile(0, y).Light = 1.0
		m.Tile(m.Width-1, y).Light = 1.0
	}
}
