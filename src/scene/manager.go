package scene

import (
	"github.com/hajimehoshi/ebiten/v2"
)

type Manager struct {
	current Scene
}

func NewManager(initial Scene) *Manager {
	return &Manager{current: initial}
}

func (m *Manager) Update() error {
	next, err := m.current.Update()
	if err != nil {
		return err
	}
	if next != nil {
		m.current = next
	}
	return nil
}

func (m *Manager) Draw(screen *ebiten.Image) {
	m.current.Draw(screen)
}
