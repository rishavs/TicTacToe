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
	opts := parseQAOptions()
	initial, err := newInitialScene(opts)
	if err != nil {
		log.Fatal(err)
	}

	ebiten.SetWindowSize(scene.InternalWidth*scene.WindowScale, scene.InternalHeight*scene.WindowScale)
	ebiten.SetWindowResizingMode(ebiten.WindowResizingModeEnabled)
	ebiten.SetWindowTitle("Stoneheart")

	var game ebiten.Game
	if opts.capture != "" {
		game = newCaptureGame(initial, opts.capture)
	} else {
		game = &Game{manager: scene.NewManager(initial)}
	}

	if err := ebiten.RunGame(game); err != nil {
		log.Fatal(err)
	}
}
