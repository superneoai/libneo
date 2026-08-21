//! Native glass effects.
//!
//! A glass effect takes its position from GPUI layout, and AppKit draws it
//! above the GPUI content. [`GlassEffectContent`] puts native views on the glass effect.

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Refineable, Rgba, Style, StyleRefinement, Styled, Window,
};

/// Selects the glass effect style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlassEffectStyle {
    /// Uses the regular style.
    Regular,
    /// Uses the clear style.
    Clear,
}

/// Groups nearby glass effects in an `NSGlassEffectContainerView`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassEffectGroup {
    pub(crate) id: String,
    pub(crate) spacing: Pixels,
}

impl GlassEffectGroup {
    /// Creates a group with the supplied effect-merging distance.
    pub fn new(id: impl Into<String>, spacing: Pixels) -> Self {
        Self {
            id: id.into(),
            spacing,
        }
    }

    /// Sets the distance at which the effects merge.
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = spacing;
        self
    }
}

/// Selects the native content of a glass effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlassEffectContent {
    /// Shows a static text label.
    Label(String),
}

/// Configures a native glass effect's presentation and content.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassEffectConfiguration {
    /// Selects the AppKit glass style.
    pub style: GlassEffectStyle,
    /// Sets the corner radius in logical pixels.
    pub corner_radius: Pixels,
    /// Sets a tint color, or leaves the effect untinted.
    pub tint: Option<Rgba>,
    /// Joins a glass effect group, or leaves the effect ungrouped.
    pub group: Option<GlassEffectGroup>,
    /// Supplies native content, or leaves the effect empty.
    pub content: Option<GlassEffectContent>,
}

/// Places a native `NSGlassEffectView` in GPUI layout.
///
/// # Panics
///
/// Painting panics unless [`crate::install`] initialized the application and
/// [`crate::NativeRoot`] wraps this window's root. It also panics off the main
/// thread or when AppKit fails to create the `NSGlassEffectView`.
pub struct GlassEffect {
    id: String,
    configuration: GlassEffectConfiguration,
    style: StyleRefinement,
}

/// Creates a glass effect with caller-supplied presentation and content.
pub fn glass_effect(id: impl Into<String>, configuration: GlassEffectConfiguration) -> GlassEffect {
    GlassEffect {
        id: id.into(),
        configuration,
        style: StyleRefinement::default(),
    }
}

impl GlassEffect {
    /// Sets the glass effect style.
    pub fn effect_style(mut self, style: GlassEffectStyle) -> Self {
        self.configuration.style = style;
        self
    }

    /// Sets the corner radius in logical pixels.
    pub fn corner_radius(mut self, radius: Pixels) -> Self {
        self.configuration.corner_radius = radius;
        self
    }

    /// Sets the glass tint color.
    pub fn tint(mut self, color: Rgba) -> Self {
        self.configuration.tint = Some(color);
        self
    }

    /// Adds the effect to a glass effect group.
    pub fn group(mut self, group: GlassEffectGroup) -> Self {
        self.configuration.group = Some(group);
        self
    }

    /// Sets the native content of the glass effect.
    pub fn content(mut self, content: GlassEffectContent) -> Self {
        self.configuration.content = Some(content);
        self
    }
}

impl IntoElement for GlassEffect {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for GlassEffect {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Element for GlassEffect {
    type RequestLayoutState = Style;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone().into())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = window.request_layout(style.clone(), [], cx);
        (layout_id, style)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        style: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        style.paint(bounds, window, cx, |_, _| {});

        let window_id = window.window_handle().window_id();
        let mtm = crate::lifecycle::main_thread_marker(cx);
        crate::native_views::record_glass_effect(
            window_id,
            window,
            GlassEffectFrame {
                id: self.id.clone(),
                configuration: self.configuration.clone(),
                bounds,
            },
            mtm,
            cx,
        )
        .unwrap_or_else(|error| panic!("the glass effect must apply: {error}"));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlassEffectFrame {
    pub(crate) id: String,
    pub(crate) configuration: GlassEffectConfiguration,
    pub(crate) bounds: Bounds<Pixels>,
}
