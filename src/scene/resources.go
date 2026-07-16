package scene

import (
	"bytes"
	"embed"
	"sync"

	"github.com/ebitenui/ebitenui/themes"
	"github.com/ebitenui/ebitenui/widget"
	"github.com/hajimehoshi/ebiten/v2/text/v2"
)

//go:embed fonts/PressStart2P-Regular.ttf
var pressStart2PTTF embed.FS

var (
	themeOnce sync.Once
	theme     *widget.Theme
	titleFace text.Face
	nameFace  text.Face
)

func loadTheme() {
	themeOnce.Do(func() {
		fontSrc := mustLoadSource()

		face8 := text.Face(&text.GoTextFace{Source: fontSrc, Size: 8})
		titleFace = &text.GoTextFace{Source: fontSrc, Size: 12}
		nameFace = &text.GoTextFace{Source: fontSrc, Size: 10}

		theme = themes.GetBasicDarkTheme()
		theme.DefaultFace = &face8
		theme.ButtonTheme.TextFace = &face8
		theme.ButtonTheme.TextPadding = &widget.Insets{Left: 4, Right: 4, Top: 1, Bottom: 1}
		theme.LabelTheme.Face = &face8
		theme.TextTheme.Face = &face8
		theme.CheckboxTheme.Label.Face = &face8
	})
}

func mustLoadSource() *text.GoTextFaceSource {
	data, err := pressStart2PTTF.ReadFile("fonts/PressStart2P-Regular.ttf")
	if err != nil {
		panic(err)
	}
	s, err := text.NewGoTextFaceSource(bytes.NewReader(data))
	if err != nil {
		panic(err)
	}
	return s
}
