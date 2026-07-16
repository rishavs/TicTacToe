package main

import (
	"flag"
	"fmt"
	"image"
	"image/png"
	"os"
	"path/filepath"
	"strings"

	"github.com/hajimehoshi/ebiten/v2"

	"stoneheart/src/scene"
)

type qaOptions struct {
	sceneName string
	seed      int64
	capture   string
}

func parseQAOptions() qaOptions {
	opts := qaOptions{sceneName: "menu", seed: 42}
	flag.StringVar(&opts.sceneName, "qa-scene", opts.sceneName, "initial scene for QA launch: menu, mapgen, new, battle, settings")
	flag.Int64Var(&opts.seed, "qa-seed", opts.seed, "seed used by QA scene captures that support deterministic generation")
	flag.StringVar(&opts.capture, "qa-capture", "", "write one rendered frame to this PNG path and exit")
	flag.Parse()
	opts.sceneName = strings.ToLower(strings.TrimSpace(opts.sceneName))
	return opts
}

func newInitialScene(opts qaOptions) (scene.Scene, error) {
	switch opts.sceneName {
	case "", "menu":
		return scene.NewMenuScene(), nil
	case "mapgen":
		return scene.NewMapgenSceneWithSeed(opts.seed), nil
	case "new":
		return scene.NewPlaceholderScene("New Game"), nil
	case "battle":
		return scene.NewPlaceholderScene("Battle"), nil
	case "settings":
		return scene.NewPlaceholderScene("Settings"), nil
	default:
		return nil, fmt.Errorf("unknown QA scene %q", opts.sceneName)
	}
}

type captureGame struct {
	scene  scene.Scene
	output string
	err    error
	done   bool
}

func newCaptureGame(initial scene.Scene, output string) *captureGame {
	return &captureGame{scene: initial, output: output}
}

func (g *captureGame) Update() error {
	if g.err != nil {
		return g.err
	}
	if g.done {
		return ebiten.Termination
	}

	next, err := g.scene.Update()
	if err != nil {
		return err
	}
	if next != nil {
		g.scene = next
	}
	return nil
}

func (g *captureGame) Draw(screen *ebiten.Image) {
	g.scene.Draw(screen)
	if g.done {
		return
	}
	g.err = saveScreenPNG(screen, g.output)
	g.done = true
}

func (g *captureGame) Layout(outsideWidth, outsideHeight int) (screenWidth, screenHeight int) {
	return scene.InternalWidth, scene.InternalHeight
}

func saveScreenPNG(screen *ebiten.Image, output string) error {
	bounds := screen.Bounds()
	w, h := bounds.Dx(), bounds.Dy()
	pixels := make([]byte, 4*w*h)
	screen.ReadPixels(pixels)

	img := image.NewNRGBA(image.Rect(0, 0, w, h))
	copy(img.Pix, pixels)

	dir := filepath.Dir(output)
	if dir != "." && dir != "" {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return fmt.Errorf("create capture dir: %w", err)
		}
	}

	file, err := os.Create(output)
	if err != nil {
		return fmt.Errorf("create capture file: %w", err)
	}
	defer file.Close()

	if err := png.Encode(file, img); err != nil {
		return fmt.Errorf("encode capture PNG: %w", err)
	}
	return nil
}
