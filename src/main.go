package main

import (
	"log"

	"github.com/hajimehoshi/ebiten/v2"

	"stoneheart/src/scene"
)

type Game struct {
	manager *scene.Manager
}

func (g *Game) Update() error {
	return g.manager.Update()
}

func (g *Game) Draw(screen *ebiten.Image) {
	g.manager.Draw(screen)
}

func (g *Game) Layout(outsideWidth, outsideHeight int) (screenWidth, screenHeight int) {
	return scene.InternalWidth, scene.InternalHeight
}

func main() {
	manager := scene.NewManager(scene.NewMenuScene())

	ebiten.SetWindowSize(scene.InternalWidth*scene.WindowScale, scene.InternalHeight*scene.WindowScale)
	ebiten.SetWindowResizingMode(ebiten.WindowResizingModeEnabled)
	ebiten.SetWindowTitle("Stoneheart")
	if err := ebiten.RunGame(&Game{manager: manager}); err != nil {
		log.Fatal(err)
	}
}
