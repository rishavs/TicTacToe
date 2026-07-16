package camera

import "math"

type Camera struct {
	X        float64
	Y        float64
	Zoom     float64
	MinZoom  float64
	MaxZoom  float64
	TileSize float64
}

func New(tileSize float64) *Camera {
	return &Camera{
		X:        0,
		Y:        0,
		Zoom:     1.0,
		MinZoom:  0.05,
		MaxZoom:  16.0,
		TileSize: tileSize,
	}
}

type Viewport struct {
	MinX, MinY int
	MaxX, MaxY int
}

func (c *Camera) Viewport(screenW, screenH int) Viewport {
	invZoom := 1.0 / c.Zoom
	return Viewport{
		MinX: int(math.Floor(c.X / c.TileSize)),
		MinY: int(math.Floor(c.Y / c.TileSize)),
		MaxX: int(math.Ceil((c.X + float64(screenW)*invZoom) / c.TileSize)),
		MaxY: int(math.Ceil((c.Y + float64(screenH)*invZoom) / c.TileSize)),
	}
}

func (c *Camera) WorldToScreen(worldX, worldY float64) (float64, float64) {
	return (worldX - c.X) * c.Zoom, (worldY - c.Y) * c.Zoom
}

func (c *Camera) TileToScreen(tileX, tileY int) (float64, float64) {
	return c.WorldToScreen(float64(tileX)*c.TileSize, float64(tileY)*c.TileSize)
}

func (c *Camera) Move(dx, dy float64) {
	c.X += dx
	c.Y += dy
}

func (c *Camera) SetZoom(z float64) {
	if z < c.MinZoom {
		z = c.MinZoom
	}
	if z > c.MaxZoom {
		z = c.MaxZoom
	}
	c.Zoom = z
}

func (c *Camera) ZoomAt(z float64, screenX, screenY int) {
	worldX := float64(screenX)/c.Zoom + c.X
	worldY := float64(screenY)/c.Zoom + c.Y
	c.SetZoom(z)
	c.X = worldX - float64(screenX)/c.Zoom
	c.Y = worldY - float64(screenY)/c.Zoom
}
