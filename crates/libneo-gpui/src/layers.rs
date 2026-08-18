//! Places layers in the window.
//!
//! [`overlay`] sets the position and paint order of floating content.

use gpui::{
    AnchoredPositionMode, AnyElement, Deferred, IntoElement, ParentElement, Pixels, Point,
    anchored, deferred,
};

pub use gpui::{Anchor as AnchorCorner, Edges, point};

/// The default paint priority of [`overlay`].
pub const OVERLAY_PRIORITY: usize = 0;

/// Selects the origin of the anchor position.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LayerPositionMode {
    /// Uses window coordinates.
    #[default]
    Window,
    /// Uses coordinates of the parent element.
    Local,
}

/// Selects the method that keeps content inside the window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LayerFitting {
    /// Moves the anchor to the opposite side.
    #[default]
    SwitchAnchor,
    /// Moves the content to the nearest window edge.
    SnapToWindow,
    /// Moves the content to the nearest window edge, and keeps the margins.
    SnapToWindowWithMargin(Edges<Pixels>),
}

/// Places floating content in the window.
pub struct Overlay {
    child: AnyElement,
    anchor: AnchorCorner,
    position: Option<Point<Pixels>>,
    offset: Option<Point<Pixels>>,
    position_mode: LayerPositionMode,
    fitting: LayerFitting,
    priority: usize,
}

/// Creates an overlay with [`OVERLAY_PRIORITY`].
pub fn overlay(child: impl IntoElement) -> Overlay {
    Overlay {
        child: child.into_any_element(),
        anchor: AnchorCorner::TopLeft,
        position: None,
        offset: None,
        position_mode: LayerPositionMode::Window,
        fitting: LayerFitting::SwitchAnchor,
        priority: OVERLAY_PRIORITY,
    }
}

impl Overlay {
    /// Sets the anchor corner.
    pub fn anchor(mut self, anchor: AnchorCorner) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the anchor position.
    pub fn position(mut self, position: Point<Pixels>) -> Self {
        self.position = Some(position);
        self
    }

    /// Adds an offset to the anchor position.
    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the origin for the position.
    pub fn position_mode(mut self, mode: LayerPositionMode) -> Self {
        self.position_mode = mode;
        self
    }

    /// Sets the method that keeps the content inside the window.
    pub fn fitting(mut self, fitting: LayerFitting) -> Self {
        self.fitting = fitting;
        self
    }

    /// Sets the paint priority. A higher value paints above a lower value.
    pub fn priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}

impl IntoElement for Overlay {
    type Element = Deferred;

    fn into_element(self) -> Self::Element {
        let mut placement =
            anchored()
                .anchor(self.anchor)
                .position_mode(match self.position_mode {
                    LayerPositionMode::Window => AnchoredPositionMode::Window,
                    LayerPositionMode::Local => AnchoredPositionMode::Local,
                });

        if let Some(position) = self.position {
            placement = placement.position(position);
        }
        if let Some(offset) = self.offset {
            placement = placement.offset(offset);
        }
        placement = match self.fitting {
            LayerFitting::SwitchAnchor => placement,
            LayerFitting::SnapToWindow => placement.snap_to_window(),
            LayerFitting::SnapToWindowWithMargin(margin) => {
                placement.snap_to_window_with_margin(margin)
            }
        };

        deferred(placement.child(self.child)).with_priority(self.priority)
    }
}
