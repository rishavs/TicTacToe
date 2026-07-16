package mapgen

type BiomeType int

const (
	BiomeDeepOcean              BiomeType = iota
	BiomeShallowOcean
	BiomeDeepLake
	BiomeShallowLake
	BiomeMarsh
	BiomeIce
	BiomeBeach
	BiomeSnow
	BiomeTundra
	BiomeBare
	BiomeScorched
	BiomeTaiga
	BiomeShrubland
	BiomeTemperateDesert
	BiomeTemperateRainForest
	BiomeTemperateDeciduousForest
	BiomeGrassland
	BiomeTropicalRainForest
	BiomeTropicalSeasonalForest
	BiomeSubtropicalDesert
)

type Tile struct {
	Elevation   float64
	Moisture    float64
	Temperature float64
	Light       float64
	Biome       BiomeType
	Downslope   int
	IsWater     bool
	IsOcean     bool
	IsCoast     bool
	IsShallow   bool
	IsRiver     bool
	RiverWidth  int
}

type GameMap struct {
	Width      int
	Height     int
	Tiles      []Tile
	Seed       int64
	NoisyEdges *NoisyEdges
}

type MapConfig struct {
	Width           int
	Height          int
	Seed            int64
	IslandRoundness float64
	IslandInflate   float64
	NoiseAmplitudes []float64
	NumRivers       int
	RiverMaxWidth   int
	MoistureBias    float64
	NorthTempBias   float64
	SouthTempBias   float64
}

func DefaultConfig() MapConfig {
	return MapConfig{
		Width:           256,
		Height:          256,
		Seed:            42,
		IslandRoundness: 0.5,
		IslandInflate:   0.4,
		NoiseAmplitudes: []float64{0.5, 0.25, 0.125, 0.0625},
		NumRivers:       30,
		RiverMaxWidth:   4,
		MoistureBias:    0.0,
		NorthTempBias:   0.0,
		SouthTempBias:   0.0,
	}
}

func NewGameMap(cfg MapConfig) *GameMap {
	return &GameMap{
		Width:  cfg.Width,
		Height: cfg.Height,
		Tiles:  make([]Tile, cfg.Width*cfg.Height),
		Seed:   cfg.Seed,
	}
}

func (m *GameMap) Index(x, y int) int {
	return y*m.Width + x
}

func (m *GameMap) Coord(idx int) (x, y int) {
	y = idx / m.Width
	x = idx % m.Width
	return
}

func (m *GameMap) InBounds(x, y int) bool {
	return x >= 0 && x < m.Width && y >= 0 && y < m.Height
}

func (m *GameMap) Tile(x, y int) *Tile {
	return &m.Tiles[m.Index(x, y)]
}

func (m *GameMap) Each(f func(x, y int, t *Tile)) {
	for y := 0; y < m.Height; y++ {
		for x := 0; x < m.Width; x++ {
			f(x, y, m.Tile(x, y))
		}
	}
}

var N4Offsets = [4][2]int{
	{1, 0},  // 0: E
	{0, 1},  // 1: S
	{-1, 0}, // 2: W
	{0, -1}, // 3: N
}

var D8Offsets = [8][2]int{
	{1, 0},   // 0: E
	{1, 1},   // 1: SE
	{0, 1},   // 2: S
	{-1, 1},  // 3: SW
	{-1, 0},  // 4: W
	{-1, -1}, // 5: NW
	{0, -1},  // 6: N
	{1, -1},  // 7: NE
}

func D8Neighbor(idx, w, h int, dir int) (int, bool) {
	x := idx % w
	y := idx / w
	nx := x + D8Offsets[dir][0]
	ny := y + D8Offsets[dir][1]
	if nx < 0 || nx >= w || ny < 0 || ny >= h {
		return 0, false
	}
	return ny*w + nx, true
}

func N4Neighbor(idx, w, h int, dir int) (int, bool) {
	x := idx % w
	y := idx / w
	nx := x + N4Offsets[dir][0]
	ny := y + N4Offsets[dir][1]
	if nx < 0 || nx >= w || ny < 0 || ny >= h {
		return 0, false
	}
	return ny*w + nx, true
}
