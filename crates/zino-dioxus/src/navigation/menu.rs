/// A pop-up menu for quick access to relevant commands.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextMenu {
    /// A flag to indicate whether the context menu is visible or not.
    visible: bool,
    /// The x-coordinate of the context menu.
    position_x: f64,
    /// The y-coordinate of the context menu.
    position_y: f64,
}

impl ContextMenu {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the context menu.
    #[inline]
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hides the context menu.
    #[inline]
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Sets the position of the context menu.
    #[inline]
    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position_x = x;
        self.position_y = y;
    }

    /// Returns `true` if the context menu is visible.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the x-coordinate of the context menu.
    #[inline]
    pub fn position_x(&self) -> f64 {
        self.position_x
    }

    /// Returns the y-coordinate of the context menu.
    #[inline]
    pub fn position_y(&self) -> f64 {
        self.position_y
    }
}
