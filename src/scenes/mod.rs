pub mod battle;
pub mod mapgen;
pub mod menu;
pub mod play;
pub mod settings;

pub enum Scene {
    MainMenu,
    Play,
    Mapgen,
    Battle,
    Settings,
}
