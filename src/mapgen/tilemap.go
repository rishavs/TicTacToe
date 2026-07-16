package mapgen

type BiomeType int

const (
	BiomeDeepOcean BiomeType = iota
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
	Rainfall    float64
	Flow        float64
	RiverScale  float64
	Temperature float64
	Light       float64
	Biome       BiomeType
	Downslope   int
	FlowDir     int
	WatershedID int
	LakeID      int
	IsWater     bool
	IsOcean     bool
	IsCoast     bool
	IsShallow   bool
	IsRiver     bool
}

type Lake struct {
	ID           int
	TileCount    int
	SurfaceLevel float64
	Outlet       int
	Flow         float64
}

type Watershed struct {
	ID     int
	Outlet int
	LakeID int
	Area   int
	Flow   float64
}

type GameMap struct {
	Width      int
	Height     int
	Tiles      []Tile
	Seed       int64
	Lakes      []Lake
	Watersheds []Watershed
}

type MapConfig struct {
	Width           int
	Height          int
	Seed            int64
	IslandRoundness float64
	IslandInflate   float64
	NoiseAmplitudes []float64
	MaxLakes        int
	MinLakeArea     int
	MaxLakeArea     int
	LakeDepth       float64
	LakeSpacing     int
	RainfallScale   float64
	RainfallAmp     float64
	RiverThreshold  float64
	RiverMaxScale   float64
	MoistureBias    float64
	NorthTempBias   float64
	SouthTempBias   float64
	WaterDepthScale float64
	WaterDepthAmp   float64
	WaterDepthLimit float64
	LightAmbient    float64
	LightSlopeScale float64
	LightMin        float64
	LightMax        float64
}

func DefaultConfig() MapConfig {
	return MapConfig{
		Width:           256,
		Height:          256,
		Seed:            42,
		IslandRoundness: 0.5,
		IslandInflate:   0.4,
		NoiseAmplitudes: []float64{0.5, 0.25, 0.125, 0.0625},
		MaxLakes:        8,
		MinLakeArea:     8,
		MaxLakeArea:     90,
		LakeDepth:       0.08,
		LakeSpacing:     18,
		RainfallScale:   3.5,
		RainfallAmp:     0.45,
		RiverThreshold:  90,
		RiverMaxScale:   3,
		MoistureBias:    0.0,
		NorthTempBias:   0.0,
		SouthTempBias:   0.0,
		WaterDepthScale: 200,
		WaterDepthAmp:   0.15,
		WaterDepthLimit: -0.25,
		LightAmbient:    0.65,
		LightSlopeScale: 2.0,
		LightMin:        0.2,
		LightMax:        1.0,
	}
}

func NewGameMap(cfg MapConfig) *GameMap {
	m := &GameMap{
		Width:  cfg.Width,
		Height: cfg.Height,
		Tiles:  make([]Tile, cfg.Width*cfg.Height),
		Seed:   cfg.Seed,
	}
	for i := range m.Tiles {
		m.Tiles[i].Downslope = -1
		m.Tiles[i].FlowDir = -1
		m.Tiles[i].WatershedID = -1
		m.Tiles[i].LakeID = -1
	}
	return m
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

func (m *GameMap) EachN4(idx int, f func(nidx int, dir int)) {
	for dir := 0; dir < 4; dir++ {
		nidx, ok := N4Neighbor(idx, m.Width, m.Height, dir)
		if ok {
			f(nidx, dir)
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
