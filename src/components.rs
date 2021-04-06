mod main_window;
pub use main_window::*;

mod navigator;
pub use navigator::*;

mod text_view;
pub use text_view::*;

#[derive(Debug)]
pub enum AppEvent {
    Start,
    Close,
    Get { url: String },
    Contents { contents: String },
    Err(anyhow::Error),
}

pub trait EventListener {
    fn on_event(&self, event: &AppEvent);
}

/// Application component
pub trait Component {
    /// Initializes gtk widgets
    fn initialize(&self) {}

    /// Returns component's children
    fn children_mut(&mut self) -> Option<&mut [Box<dyn EventListener>]>;

    /// Propagates `AppEvent` towards children
    fn propagate_event(&mut self, event: &AppEvent) {
        if let Some(children) = self.children_mut() {
            children.iter_mut().for_each(|child| child.on_event(event))
        }
    }
}
