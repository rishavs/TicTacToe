package scene

import (
	"image/color"

	"github.com/ebitenui/ebitenui"
	"github.com/ebitenui/ebitenui/widget"
	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/text/v2"
)

type PlaceholderScene struct {
	ui        *ebitenui.UI
	nextScene Scene
	name      string
}

func NewPlaceholderScene(name string) *PlaceholderScene {
	loadTheme()
	s := &PlaceholderScene{name: name}

	root := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewAnchorLayout()),
	)

	backBtn := widget.NewButton(
		widget.ButtonOpts.TextLabel("Back"),
		widget.ButtonOpts.TextPadding(&widget.Insets{
			Left: 20, Right: 20, Top: 4, Bottom: 4,
		}),
		widget.ButtonOpts.WidgetOpts(
			widget.WidgetOpts.LayoutData(widget.AnchorLayoutData{
				HorizontalPosition: widget.AnchorLayoutPositionCenter,
				VerticalPosition:   widget.AnchorLayoutPositionEnd,
			}),
		),
		widget.ButtonOpts.ClickedHandler(func(_ *widget.ButtonClickedEventArgs) {
			s.nextScene = NewMenuScene()
		}),
	)
	root.AddChild(backBtn)

	s.ui = &ebitenui.UI{Container: root, PrimaryTheme: theme}
	return s
}

func (s *PlaceholderScene) Update() (Scene, error) {
	s.ui.Update()
	if s.nextScene != nil {
		next := s.nextScene
		s.nextScene = nil
		return next, nil
	}
	return nil, nil
}

func (s *PlaceholderScene) Draw(screen *ebiten.Image) {
	screen.Fill(color.RGBA{0x2e, 0x2e, 0x2e, 0xff})

	w, _ := text.Measure(s.name, nameFace, 0)
	op := &text.DrawOptions{}
	op.GeoM.Translate((InternalWidth-w)/2, 80)
	op.ColorScale.ScaleWithColor(color.White)
	text.Draw(screen, s.name, nameFace, op)

	s.ui.Draw(screen)
}
