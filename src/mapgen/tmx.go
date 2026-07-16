package mapgen

type TMXTileLayer struct {
	Name  string
	Width int
	Data  []int
}

type TMXMap struct {
	Width       int
	Height      int
	TileWidth   int
	TileHeight  int
	Layers      []TMXTileLayer
}
