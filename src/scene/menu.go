package scene

import (
	"image/color"

	"github.com/ebitenui/ebitenui"
	"github.com/ebitenui/ebitenui/widget"
	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/text/v2"
)

type MenuScene struct {
	ui        *ebitenui.UI
	nextScene Scene
	quit      bool
}

func NewMenuScene() *MenuScene {
	loadTheme()

	root := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewAnchorLayout()),
	)

	buttons := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Direction(widget.DirectionVertical),
			widget.RowLayoutOpts.Spacing(8),
		)),
		widget.ContainerOpts.WidgetOpts(
			widget.WidgetOpts.LayoutData(widget.AnchorLayoutData{
				HorizontalPosition: widget.AnchorLayoutPositionCenter,
				VerticalPosition:   widget.AnchorLayoutPositionCenter,
			}),
		),
	)

	s := &MenuScene{}

	entries := []struct {
		Label string
		Click func()
	}{
		{"New", func() { s.nextScene = NewPlaceholderScene("New Game") }},
		{"Mapgen", func() { s.nextScene = NewMapgenScene() }},
		{"Battle", func() { s.nextScene = NewPlaceholderScene("Battle") }},
		{"Settings", func() { s.nextScene = NewPlaceholderScene("Settings") }},
		{"Quit", func() { s.quit = true }},
	}

	for _, e := range entries {
		e := e
		btn := widget.NewButton(
			widget.ButtonOpts.TextLabel(e.Label),
			widget.ButtonOpts.ClickedHandler(func(_ *widget.ButtonClickedEventArgs) {
				e.Click()
			}),
		)
		buttons.AddChild(btn)
	}

	root.AddChild(buttons)
	s.ui = &ebitenui.UI{Container: root, PrimaryTheme: theme}
	return s
}

func (s *MenuScene) Update() (Scene, error) {
	s.ui.Update()
	if s.quit {
		return nil, ebiten.Termination
	}
	if s.nextScene != nil {
		next := s.nextScene
		s.nextScene = nil
		return next, nil
	}
	return nil, nil
}

func (s *MenuScene) Draw(screen *ebiten.Image) {
	screen.Fill(color.RGBA{0x1a, 0x1a, 0x2e, 0xff})

	title := "STONEHEART"
	w, _ := text.Measure(title, titleFace, 0)
	op := &text.DrawOptions{}
	op.GeoM.Translate((320-w)/2, 40)
	op.ColorScale.ScaleWithColor(color.White)
	text.Draw(screen, title, titleFace, op)

	s.ui.Draw(screen)
}
