---
status: complete
created: 2026-07-15
tags:
- scene-management
- ui
- menu
- ebitenui
created_at: 2026-07-15T04:07:37.393621600Z
updated_at: 2026-07-15T09:37:00.000000000Z
---
# Main Menu & Scene System

Add a main menu with 5 buttons (New, Mapgen, Battle, Settings, Quit) backed by a simple scene manager. Each button navigates to an empty placeholder scene with a Back button.

## Requirements

### Scene Manager
- Simple scene stack or switch-based manager in its own package (`scene`)
- `Scene` interface with `Update()` and `Draw()` methods
- `Manager` type that holds current scene and transitions between them

### Main Menu Scene
- Title "STONEHEART" centered at top
- 5 vertically stacked buttons: New, Mapgen, Battle, Settings, Quit
- Quit exits the game
- Other 4 buttons transition to their respective placeholder scene

### Placeholder Scenes (New, Mapgen, Battle, Settings)
- Each is an empty scene with:
  - Scene name displayed centered (e.g. "New Game")
  - A "Back" button that returns to the main menu

### Libraries Used
- `github.com/ebitenui/ebitenui` — retained-mode UI engine (buttons, containers, layouts)
- `github.com/hajimehoshi/ebiten/v2/text/v2` — text rendering with Go font
- `golang.org/x/image/font/gofont/goregular` — embedded Go font TTF

## Files Created
- `src/scene/scene.go` — Scene interface
- `src/scene/manager.go` — switch-based manager, holds current scene, delegates Update/Draw
- `src/scene/resources.go` — shared theme: button images (NineSliceColor), text colors, font faces
- `src/scene/menu.go` — MenuScene with ebitenui buttons in vertical RowLayout, centered via AnchorLayout
- `src/scene/placeholder.go` — generic placeholder with scene name + Back button using ebitenui

## Files Modified
- `src/main.go` — uses scene.Manager instead of inline Hello World
- `go.mod` — added `github.com/ebitenui/ebitenui`, `golang.org/x/image`

## Design Decisions
- **ebitenui** chosen over manual rendering, furex-ui, and etk for widget maturity and production use
- Simple switch-based manager (no stack yet — single scene at a time)
- Button backgrounds via `image.NewNineSliceColor` (solid color, no asset files needed)
- Font via embedded Go font (`goregular.TTF`), parsed with `text/v2`
- Scene transitions via `nextScene` field set in button ClickedHandler
- Quit returns `ebiten.Termination` sentinel error
- Internal resolution 320×240, window 640×480 (scaled by Ebiten's default filter)
