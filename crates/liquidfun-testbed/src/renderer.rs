//! Passive presentation contracts for the replacement desktop renderer.
//!
//! The boundary owns semantic presentation values and returns only passive
//! input, clipboard, or pixel results. Simulation and comparison authority
//! remain outside the renderer.

#[path = "renderer/image.rs"]
pub(crate) mod image;

use thiserror::Error;

pub(crate) const MAX_IMAGE_DIMENSION: u32 = 4_096;
const RGBA_CHANNEL_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum RendererError {
    #[error("renderer dimensions must be finite and greater than zero")]
    InvalidDimensions,
    #[error("renderer dimensions exceed the supported image limit")]
    DimensionLimitExceeded,
    #[error("renderer image byte count overflowed")]
    ImageByteCountOverflow,
    #[error("renderer could not allocate the requested image")]
    ImageAllocationFailed,
    #[error("renderer could not encode PNG output")]
    PngEncodingFailed,
    #[error("renderer drawing input is invalid")]
    InvalidDrawing,
    #[error("renderer clipboard operation failed")]
    ClipboardUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RgbaColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl RgbaColor {
    pub(crate) const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub(crate) const fn channels(self) -> [u8; RGBA_CHANNEL_COUNT] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PhysicalSize {
    width: u32,
    height: u32,
    rgba_byte_len: usize,
}

impl PhysicalSize {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::InvalidDimensions);
        }
        if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
            return Err(RendererError::DimensionLimitExceeded);
        }
        let width = usize::try_from(width).map_err(|_| RendererError::ImageByteCountOverflow)?;
        let height = usize::try_from(height).map_err(|_| RendererError::ImageByteCountOverflow)?;
        let rgba_byte_len = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(RGBA_CHANNEL_COUNT))
            .ok_or(RendererError::ImageByteCountOverflow)?;
        Ok(Self {
            width: u32::try_from(width).map_err(|_| RendererError::ImageByteCountOverflow)?,
            height: u32::try_from(height).map_err(|_| RendererError::ImageByteCountOverflow)?,
            rgba_byte_len,
        })
    }

    pub(crate) const fn width(self) -> u32 {
        self.width
    }

    pub(crate) const fn height(self) -> u32 {
        self.height
    }

    pub(crate) const fn rgba_byte_len(self) -> usize {
        self.rgba_byte_len
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LogicalSize {
    width: f32,
    height: f32,
}

impl LogicalSize {
    pub(crate) fn new(width: f32, height: f32) -> Result<Self, RendererError> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return Err(RendererError::InvalidDimensions);
        }
        Ok(Self { width, height })
    }

    pub(crate) const fn width(self) -> f32 {
        self.width
    }

    pub(crate) const fn height(self) -> f32 {
        self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LogicalPoint {
    x: f32,
    y: f32,
}

impl LogicalPoint {
    pub(crate) fn new(x: f32, y: f32) -> Result<Self, RendererError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(RendererError::InvalidDrawing);
        }
        Ok(Self { x, y })
    }

    pub(crate) const fn x(self) -> f32 {
        self.x
    }

    pub(crate) const fn y(self) -> f32 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Rectangle {
    origin: LogicalPoint,
    size: LogicalSize,
    color: RgbaColor,
}

impl Rectangle {
    pub(crate) const fn new(origin: LogicalPoint, size: LogicalSize, color: RgbaColor) -> Self {
        Self {
            origin,
            size,
            color,
        }
    }

    pub(crate) const fn origin(self) -> LogicalPoint {
        self.origin
    }

    pub(crate) const fn size(self) -> LogicalSize {
        self.size
    }

    pub(crate) const fn color(self) -> RgbaColor {
        self.color
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Stroke {
    color: RgbaColor,
    width: f32,
}

impl Stroke {
    pub(crate) fn new(color: RgbaColor, width: f32) -> Result<Self, RendererError> {
        if !width.is_finite() || width <= 0.0 {
            return Err(RendererError::InvalidDrawing);
        }
        Ok(Self { color, width })
    }

    pub(crate) const fn color(self) -> RgbaColor {
        self.color
    }

    pub(crate) const fn width(self) -> f32 {
        self.width
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Line {
    start: LogicalPoint,
    end: LogicalPoint,
    stroke: Stroke,
}

impl Line {
    pub(crate) const fn new(start: LogicalPoint, end: LogicalPoint, stroke: Stroke) -> Self {
        Self { start, end, stroke }
    }

    pub(crate) const fn start(self) -> LogicalPoint {
        self.start
    }

    pub(crate) const fn end(self) -> LogicalPoint {
        self.end
    }

    pub(crate) const fn stroke(self) -> Stroke {
        self.stroke
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Circle {
    center: LogicalPoint,
    radius: f32,
    color: RgbaColor,
}

impl Circle {
    pub(crate) fn new(
        center: LogicalPoint,
        radius: f32,
        color: RgbaColor,
    ) -> Result<Self, RendererError> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(RendererError::InvalidDrawing);
        }
        Ok(Self {
            center,
            radius,
            color,
        })
    }

    pub(crate) const fn center(self) -> LogicalPoint {
        self.center
    }

    pub(crate) const fn radius(self) -> f32 {
        self.radius
    }

    pub(crate) const fn color(self) -> RgbaColor {
        self.color
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextDrawing {
    origin: LogicalPoint,
    text: String,
    size: f32,
    color: RgbaColor,
}

impl TextDrawing {
    pub(crate) fn new(
        origin: LogicalPoint,
        text: String,
        size: f32,
        color: RgbaColor,
    ) -> Result<Self, RendererError> {
        if text.is_empty() || !size.is_finite() || size <= 0.0 {
            return Err(RendererError::InvalidDrawing);
        }
        Ok(Self {
            origin,
            text,
            size,
            color,
        })
    }

    pub(crate) const fn origin(&self) -> LogicalPoint {
        self.origin
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) const fn size(&self) -> f32 {
        self.size
    }

    pub(crate) const fn color(&self) -> RgbaColor {
        self.color
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DrawCommand {
    FillRectangle(Rectangle),
    StrokeLine(Line),
    FillCircle(Circle),
    Text(TextDrawing),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PresentationFrame {
    logical_size: LogicalSize,
    clear_color: RgbaColor,
    commands: Vec<DrawCommand>,
}

impl PresentationFrame {
    pub(crate) const fn new(
        logical_size: LogicalSize,
        clear_color: RgbaColor,
        commands: Vec<DrawCommand>,
    ) -> Self {
        Self {
            logical_size,
            clear_color,
            commands,
        }
    }

    pub(crate) const fn logical_size(&self) -> LogicalSize {
        self.logical_size
    }

    pub(crate) const fn clear_color(&self) -> RgbaColor {
        self.clear_color
    }

    pub(crate) fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderedPixels {
    size: PhysicalSize,
    rgba_bytes: Vec<u8>,
    png_bytes: Vec<u8>,
}

impl RenderedPixels {
    pub(crate) fn new(
        size: PhysicalSize,
        rgba_bytes: Vec<u8>,
        png_bytes: Vec<u8>,
    ) -> Result<Self, RendererError> {
        if rgba_bytes.len() != size.rgba_byte_len() {
            return Err(RendererError::ImageByteCountOverflow);
        }
        Ok(Self {
            size,
            rgba_bytes,
            png_bytes,
        })
    }

    pub(crate) const fn size(&self) -> PhysicalSize {
        self.size
    }

    pub(crate) fn rgba_bytes(&self) -> &[u8] {
        &self.rgba_bytes
    }

    /// Returns encoded PNG bytes; validating and writing a path is caller-owned.
    pub(crate) fn png_bytes(&self) -> &[u8] {
        &self.png_bytes
    }
}

pub(crate) trait DrawingRenderer {
    fn draw(
        &mut self,
        physical_size: PhysicalSize,
        presentation: PresentationFrame,
    ) -> Result<RenderedPixels, RendererError>;
}

pub(crate) trait ImageRenderer {
    fn capture(
        &mut self,
        physical_size: PhysicalSize,
        presentation: PresentationFrame,
    ) -> Result<RenderedPixels, RendererError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SemanticKey {
    Space,
    Enter,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Character(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InputEvent {
    Key {
        key: SemanticKey,
        state: KeyState,
    },
    PointerMoved(LogicalPoint),
    PointerButton {
        position: LogicalPoint,
        pressed: bool,
    },
    Text(String),
    Resized {
        logical: LogicalSize,
        physical: PhysicalSize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClipboardRequest {
    Read,
    Write(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClipboardResponse {
    Contents(String),
    Written,
}

pub(crate) trait PassiveDesktopWindow {
    fn physical_size(&self) -> PhysicalSize;
    fn logical_size(&self) -> LogicalSize;
    fn take_input(&mut self) -> Vec<InputEvent>;
    fn clipboard(&mut self, request: ClipboardRequest) -> Result<ClipboardResponse, RendererError>;
}
