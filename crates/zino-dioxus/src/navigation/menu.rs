/// A pop-up menu for quick access to relevant commands.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextMenu {
    /// The menu ID.
    menu_id: String,
    /// The x-coordinate of the context menu.
    position_x: f64,
    /// The y-coordinate of the context menu.
    position_y: f64,
    /// A flag to indicate whether the context menu is visible or not.
    visible: bool,
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

    /// Sets the menu ID.
    #[inline]
    pub fn set_menu_id(&mut self, menu_id: String) {
        self.menu_id = menu_id;
    }

    /// Sets the position of the context menu.
    #[inline]
    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position_x = x;
        self.position_y = y;
    }

    /// Returns the menu ID.
    #[inline]
    pub fn menu_id(&self) -> &str {
        &self.menu_id
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

    /// Returns `true` if the context menu is visible.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}
