package scene

import (
	"image/color"
	"math"
	"math/rand/v2"

	"github.com/ebitenui/ebitenui"
	"github.com/ebitenui/ebitenui/image"
	"github.com/ebitenui/ebitenui/widget"
	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/inpututil"
	"github.com/hajimehoshi/ebiten/v2/vector"

	"stoneheart/src/camera"
	"stoneheart/src/mapgen"
)

var biomeColors = map[mapgen.BiomeType]color.RGBA{
	mapgen.BiomeDeepOcean:                {0x0d, 0x1f, 0x3a, 0xff},
	mapgen.BiomeShallowOcean:             {0x1a, 0x3a, 0x5c, 0xff},
	mapgen.BiomeDeepLake:                 {0x1a, 0x4a, 0x5c, 0xff},
	mapgen.BiomeShallowLake:              {0x2a, 0x6a, 0x8c, 0xff},
	mapgen.BiomeMarsh:                    {0x3a, 0x7a, 0x5a, 0xff},
	mapgen.BiomeIce:                      {0xdd, 0xee, 0xff, 0xff},
	mapgen.BiomeBeach:                    {0xc2, 0xb2, 0x80, 0xff},
	mapgen.BiomeSnow:                     {0xf0, 0xf0, 0xf0, 0xff},
	mapgen.BiomeTundra:                   {0x8f, 0x9d, 0x9d, 0xff},
	mapgen.BiomeBare:                     {0x9e, 0x8e, 0x7e, 0xff},
	mapgen.BiomeScorched:                 {0xb8, 0x88, 0x60, 0xff},
	mapgen.BiomeTaiga:                    {0x4a, 0x6e, 0x4a, 0xff},
	mapgen.BiomeShrubland:                {0x8b, 0xa3, 0x6c, 0xff},
	mapgen.BiomeTemperateDesert:          {0xcc, 0xa8, 0x70, 0xff},
	mapgen.BiomeTemperateRainForest:      {0x3d, 0x7a, 0x2d, 0xff},
	mapgen.BiomeTemperateDeciduousForest: {0x5a, 0x8e, 0x3a, 0xff},
	mapgen.BiomeGrassland:                {0x6b, 0x8e, 0x23, 0xff},
	mapgen.BiomeTropicalRainForest:       {0x2d, 0x5a, 0x1e, 0xff},
	mapgen.BiomeTropicalSeasonalForest:   {0x4a, 0x7a, 0x2e, 0xff},
	mapgen.BiomeSubtropicalDesert:        {0xe0, 0xbc, 0x70, 0xff},
}

const (
	tileSize    = 16
	panSpeed    = 4
	zoomSpeed   = 1.1
	panelWidth  = 50
	mapAreaW    = InternalWidth - panelWidth
	mapgenSizeW = 200
	mapgenSizeH = 200
	regenDelay  = 8
)

type MapgenScene struct {
	m      *mapgen.GameMap
	cam    *camera.Camera
	ui     *ebitenui.UI
	goBack bool
	seed   int64

	moistureSlider *widget.Slider
	nTempSlider    *widget.Slider
	sTempSlider    *widget.Slider
	roundSlider    *widget.Slider

	biomesOn     bool
	lightingOn   bool
	regenQueued  bool
	regenCounter int
}

func NewMapgenScene() *MapgenScene {
	return NewMapgenSceneWithSeed(42)
}

func NewMapgenSceneWithSeed(seed int64) *MapgenScene {
	loadTheme()
	s := &MapgenScene{biomesOn: true, lightingOn: true, seed: seed}
	s.cam = camera.New(tileSize)
	s.ui = buildPanel(s)
	s.regenerate(s.seed)
	return s
}

func buildPanel(s *MapgenScene) *ebitenui.UI {
	backBtn := widget.NewButton(
		widget.ButtonOpts.TextLabel("Back"),
		widget.ButtonOpts.ClickedHandler(func(_ *widget.ButtonClickedEventArgs) {
			s.goBack = true
		}),
	)

	regenerateBtn := widget.NewButton(
		widget.ButtonOpts.TextLabel("Regenerate"),
		widget.ButtonOpts.ClickedHandler(func(_ *widget.ButtonClickedEventArgs) {
			s.regenerate(rand.Int64())
		}),
	)

	uncheckedImg := ebiten.NewImage(6, 6)
	uncheckedImg.Fill(color.NRGBA{0x88, 0x88, 0xaa, 0xff})
	checkedImg := ebiten.NewImage(6, 6)
	checkedImg.Fill(color.NRGBA{0x66, 0xaa, 0xff, 0xff})
	chkImg := &widget.CheckboxImage{
		Unchecked:        image.NewFixedNineSlice(uncheckedImg),
		Checked:          image.NewFixedNineSlice(checkedImg),
		UncheckedHovered: image.NewFixedNineSlice(uncheckedImg),
		CheckedHovered:   image.NewFixedNineSlice(checkedImg),
	}

	trackImg := &widget.SliderTrackImage{
		Idle: image.NewBorderedNineSliceColor(
			color.NRGBA{0x33, 0x44, 0x55, 0xff},
			color.NRGBA{0x55, 0x66, 0x77, 0xff}, 1),
		Hover: image.NewBorderedNineSliceColor(
			color.NRGBA{0x44, 0x55, 0x66, 0xff},
			color.NRGBA{0x66, 0x77, 0x88, 0xff}, 1),
	}
	hdlImg := &widget.ButtonImage{
		Idle:    image.NewBorderedNineSliceColor(color.NRGBA{0xcc, 0xdd, 0xee, 0xff}, color.NRGBA{0xee, 0xee, 0xff, 0xff}, 1),
		Hover:   image.NewBorderedNineSliceColor(color.NRGBA{0xdd, 0xee, 0xff, 0xff}, color.NRGBA{0xff, 0xff, 0xff, 0xff}, 1),
		Pressed: image.NewBorderedNineSliceColor(color.NRGBA{0xbb, 0xcc, 0xdd, 0xff}, color.NRGBA{0xdd, 0xee, 0xff, 0xff}, 1),
	}
	sliderOpts := func(min, max, initial int, fn func()) []widget.SliderOpt {
		return []widget.SliderOpt{
			widget.SliderOpts.WidgetOpts(
				widget.WidgetOpts.LayoutData(widget.RowLayoutData{Stretch: true}),
				widget.WidgetOpts.MinSize(0, 6),
			),
			widget.SliderOpts.MinHandleSize(6),
			widget.SliderOpts.Images(trackImg, hdlImg),
			widget.SliderOpts.MinMax(min, max),
			widget.SliderOpts.InitialCurrent(initial),
			widget.SliderOpts.FixedHandleSize(6),
			widget.SliderOpts.ChangedHandler(func(_ *widget.SliderChangedEventArgs) { fn() }),
		}
	}

	content := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Direction(widget.DirectionVertical),
			widget.RowLayoutOpts.Spacing(0),
		)),
	)

	content.AddChild(widget.NewLabel(
		widget.LabelOpts.LabelText("Dry \n Wet"),
	))
	s.moistureSlider = widget.NewSlider(sliderOpts(-100, 100, 0, s.scheduleRegenerate)...)
	content.AddChild(s.moistureSlider)

	content.AddChild(widget.NewLabel(
		widget.LabelOpts.LabelText("N-Cold \n N-Hot"),
	))
	s.nTempSlider = widget.NewSlider(sliderOpts(-100, 100, 0, s.scheduleRegenerate)...)
	content.AddChild(s.nTempSlider)

	content.AddChild(widget.NewLabel(
		widget.LabelOpts.LabelText("S-Cold \n S-Hot"),
	))
	s.sTempSlider = widget.NewSlider(sliderOpts(-100, 100, 0, s.scheduleRegenerate)...)
	content.AddChild(s.sTempSlider)

	content.AddChild(widget.NewLabel(
		widget.LabelOpts.LabelText("Jagged \n Smooth"),
	))
	s.roundSlider = widget.NewSlider(sliderOpts(0, 100, 50, s.scheduleRegenerate)...)
	content.AddChild(s.roundSlider)

	content.AddChild(widget.NewLabel(
		widget.LabelOpts.LabelText("Render"),
	))
	content.AddChild(widget.NewCheckbox(
		widget.CheckboxOpts.Image(chkImg),
		widget.CheckboxOpts.TextLabel("Biomes"),
		widget.CheckboxOpts.InitialState(widget.WidgetChecked),
		widget.CheckboxOpts.StateChangedHandler(func(args *widget.CheckboxChangedEventArgs) {
			s.biomesOn = args.State == widget.WidgetChecked
		}),
	))
	content.AddChild(widget.NewCheckbox(
		widget.CheckboxOpts.Image(chkImg),
		widget.CheckboxOpts.TextLabel("Light"),
		widget.CheckboxOpts.InitialState(widget.WidgetChecked),
		widget.CheckboxOpts.StateChangedHandler(func(args *widget.CheckboxChangedEventArgs) {
			s.lightingOn = args.State == widget.WidgetChecked
		}),
	))

	scImg := &widget.ScrollContainerImage{
		Idle: image.NewNineSliceColor(color.NRGBA{0x1a, 0x1a, 0x2e, 0xff}),
		Mask: image.NewNineSliceColor(color.White),
	}
	scroll := widget.NewScrollContainer(
		widget.ScrollContainerOpts.Content(content),
		widget.ScrollContainerOpts.StretchContentWidth(),
		widget.ScrollContainerOpts.Image(scImg),
	)

	pageSizeFunc := func() int {
		h := content.GetWidget().Rect.Dy()
		vh := scroll.ViewRect().Dy()
		if h <= 0 || vh <= 0 {
			return 1000
		}
		return int(math.Round(float64(vh) / float64(h) * 1000))
	}

	vSlider := widget.NewSlider(
		widget.SliderOpts.WidgetOpts(widget.WidgetOpts.MinSize(6, 0)),
		widget.SliderOpts.Images(trackImg, hdlImg),
		widget.SliderOpts.Direction(widget.DirectionVertical),
		widget.SliderOpts.MinMax(0, 1000),
		widget.SliderOpts.PageSizeFunc(pageSizeFunc),
		widget.SliderOpts.FixedHandleSize(8),
		widget.SliderOpts.ChangedHandler(func(args *widget.SliderChangedEventArgs) {
			scroll.ScrollTop = float64(args.Slider.Current) / 1000
		}),
	)

	scroll.GetWidget().ScrolledEvent.AddHandler(func(args interface{}) {
		if a, ok := args.(*widget.WidgetScrolledEventArgs); ok {
			p := pageSizeFunc() / 3
			if p < 1 {
				p = 1
			}
			vSlider.Current -= int(math.Round(a.Y * float64(p)))
		}
	})

	scrollArea := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewGridLayout(
			widget.GridLayoutOpts.Columns(2),
			widget.GridLayoutOpts.Stretch([]bool{true, false}, []bool{true}),
		)),
	)
	scrollArea.AddChild(scroll)
	scrollArea.AddChild(vSlider)

	panel := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewGridLayout(
			widget.GridLayoutOpts.Columns(1),
			widget.GridLayoutOpts.Stretch([]bool{true}, []bool{false, true, false}),
			widget.GridLayoutOpts.Spacing(0, 0),
		)),
		widget.ContainerOpts.WidgetOpts(
			widget.WidgetOpts.LayoutData(widget.AnchorLayoutData{
				HorizontalPosition: widget.AnchorLayoutPositionEnd,
				VerticalPosition:   widget.AnchorLayoutPositionStart,
			}),
			widget.WidgetOpts.MinSize(panelWidth, InternalHeight),
		),
	)
	panel.AddChild(backBtn)
	panel.AddChild(scrollArea)
	panel.AddChild(regenerateBtn)

	root := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewAnchorLayout()),
	)
	root.AddChild(panel)
	return &ebitenui.UI{Container: root, PrimaryTheme: theme}
}

func (s *MapgenScene) regenerate(seed int64) {
	s.seed = seed
	s.regenQueued = false
	s.regenCounter = 0
	s.m = mapgen.Generate(s.mapConfig(seed))
	s.fitMap()
}

func (s *MapgenScene) scheduleRegenerate() {
	s.regenQueued = true
	s.regenCounter = regenDelay
}

func (s *MapgenScene) updateRegeneration() {
	if !s.regenQueued {
		return
	}
	if s.regenCounter > 0 {
		s.regenCounter--
		return
	}
	s.regenerate(s.seed)
}

func (s *MapgenScene) mapConfig(seed int64) mapgen.MapConfig {
	cfg := mapgen.DefaultConfig()
	cfg.Width = mapgenSizeW
	cfg.Height = mapgenSizeH
	cfg.Seed = seed
	cfg.MoistureBias = float64(s.moistureSlider.Current) / 100
	cfg.NorthTempBias = float64(s.nTempSlider.Current) / 100
	cfg.SouthTempBias = float64(s.sTempSlider.Current) / 100
	cfg.IslandRoundness = float64(s.roundSlider.Current) / 100
	return cfg
}

func (s *MapgenScene) fitMap() {
	zoomX := float64(mapAreaW) / (float64(s.m.Width) * tileSize)
	zoomY := float64(InternalHeight) / (float64(s.m.Height) * tileSize)
	zoom := min(zoomX, zoomY)
	s.cam.SetZoom(zoom)
	totalW := float64(s.m.Width) * tileSize
	totalH := float64(s.m.Height) * tileSize
	s.cam.X = (totalW - float64(mapAreaW)/s.cam.Zoom) / 2
	s.cam.Y = (totalH - float64(InternalHeight)/s.cam.Zoom) / 2
}

func (s *MapgenScene) Update() (Scene, error) {
	if s.goBack {
		return NewMenuScene(), nil
	}

	s.updateCamera()
	s.ui.Update()
	s.updateRegeneration()
	return nil, nil
}

func (s *MapgenScene) updateCamera() {
	if ebiten.IsKeyPressed(ebiten.KeyArrowLeft) || ebiten.IsKeyPressed(ebiten.KeyA) {
		s.cam.Move(-panSpeed/s.cam.Zoom, 0)
	}
	if ebiten.IsKeyPressed(ebiten.KeyArrowRight) || ebiten.IsKeyPressed(ebiten.KeyD) {
		s.cam.Move(panSpeed/s.cam.Zoom, 0)
	}
	if ebiten.IsKeyPressed(ebiten.KeyArrowUp) || ebiten.IsKeyPressed(ebiten.KeyW) {
		s.cam.Move(0, -panSpeed/s.cam.Zoom)
	}
	if ebiten.IsKeyPressed(ebiten.KeyArrowDown) || ebiten.IsKeyPressed(ebiten.KeyS) {
		s.cam.Move(0, panSpeed/s.cam.Zoom)
	}

	if inpututil.IsKeyJustPressed(ebiten.KeyF) {
		s.fitMap()
	}

	_, wheelY := ebiten.Wheel()
	if wheelY > 0 {
		s.cam.SetZoom(s.cam.Zoom * zoomSpeed)
	} else if wheelY < 0 {
		s.cam.SetZoom(s.cam.Zoom / zoomSpeed)
	}

	if ebiten.IsKeyPressed(ebiten.KeyEqual) || ebiten.IsKeyPressed(ebiten.KeyNumpadAdd) {
		s.cam.SetZoom(s.cam.Zoom * zoomSpeed)
	}
	if ebiten.IsKeyPressed(ebiten.KeyMinus) || ebiten.IsKeyPressed(ebiten.KeyNumpadSubtract) {
		s.cam.SetZoom(s.cam.Zoom / zoomSpeed)
	}
}

func (s *MapgenScene) Draw(screen *ebiten.Image) {
	screen.Fill(color.RGBA{0x00, 0x00, 0x00, 0xff})
	s.drawMap(screen)
	s.ui.Draw(screen)
}

func (s *MapgenScene) drawMap(screen *ebiten.Image) {
	mapBg := color.RGBA{0x11, 0x11, 0x22, 0xff}
	vector.DrawFilledRect(screen, 0, 0, mapAreaW, InternalHeight, mapBg, false)

	vp := s.cam.Viewport(mapAreaW, InternalHeight)

	minX := max(0, vp.MinX)
	minY := max(0, vp.MinY)
	maxX := min(s.m.Width, vp.MaxX)
	maxY := min(s.m.Height, vp.MaxY)

	riverColor := color.RGBA{0x44, 0x88, 0xcc, 0xff}
	zoom := s.cam.Zoom

	for y := minY; y < maxY; y++ {
		for x := minX; x < maxX; x++ {
			t := s.m.Tile(x, y)
			sx, sy := s.cam.TileToScreen(x, y)

			if sx >= float64(mapAreaW) {
				continue
			}

			sz := tileSize * zoom

			vector.DrawFilledRect(screen, float32(sx), float32(sy), float32(sz), float32(sz), s.tileColor(t), false)

			if t.IsRiver {
				vector.DrawFilledRect(screen, float32(sx), float32(sy), float32(sz), float32(sz), riverColor, false)
			}
		}
	}
}

func (s *MapgenScene) tileColor(t *mapgen.Tile) color.RGBA {
	var clr color.RGBA
	if s.biomesOn {
		c := biomeColors[t.Biome]
		clr = color.RGBA{c.R, c.G, c.B, 0xff}
	} else {
		e := t.Elevation
		if e < 0 {
			e = 0
		}
		v := uint8(e * 255)
		clr = color.RGBA{v, v, v, 0xff}
	}

	if !s.lightingOn {
		return clr
	}

	l := t.Light
	return color.RGBA{
		uint8(float64(clr.R) * l),
		uint8(float64(clr.G) * l),
		uint8(float64(clr.B) * l),
		0xff,
	}
}
