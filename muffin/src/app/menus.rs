pub mod create;
pub mod delete;
pub mod presets;
pub mod rename;
pub mod sessions;

use crate::app::driver::{AppEvent, AppState};

pub trait Menu {
    /// How the menu should handle the event.
    ///
    /// This can involve manipulating state, which can result in state transitions
    /// (i.e) on Escape, a menu can set state.mode = AppMode::Sessions
    fn handle_event(&mut self, event: AppEvent, state: &mut AppState);

    /// Update logic that should be run before the rendering phase
    ///
    /// Usually this will be empty, but some menus may need to update their internal state directly
    /// after a previous menu switched modes but before they have to render.
    #[allow(unused_variables)]
    fn pre_render(&mut self, state: &mut AppState) {}
}
