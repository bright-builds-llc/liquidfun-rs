//! Deterministic CPU image capture backed by `tiny-skia`.

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke as TinyStroke, Transform};

use super::{
    Circle, DrawCommand, DrawingRenderer, ImageRenderer, Line, PhysicalSize, PresentationFrame,
    Rectangle, RenderedPixels, RendererError, RgbaColor, TextDrawing,
};

pub(crate) struct TinySkiaImageRenderer;

impl DrawingRenderer for TinySkiaImageRenderer {
    fn draw(
        &mut self,
        physical_size: PhysicalSize,
        presentation: PresentationFrame,
    ) -> Result<RenderedPixels, RendererError> {
        render_pixels(physical_size, &presentation)
    }
}

impl ImageRenderer for TinySkiaImageRenderer {
    fn capture(
        &mut self,
        physical_size: PhysicalSize,
        presentation: PresentationFrame,
    ) -> Result<RenderedPixels, RendererError> {
        render_pixels(physical_size, &presentation)
    }
}

fn render_pixels(
    physical_size: PhysicalSize,
    presentation: &PresentationFrame,
) -> Result<RenderedPixels, RendererError> {
    let mut pixmap = Pixmap::new(physical_size.width(), physical_size.height())
        .ok_or(RendererError::ImageAllocationFailed)?;
    pixmap.fill(to_tiny_color(presentation.clear_color()));

    let logical_size = presentation.logical_size();
    let scale_x = bounded_dimension_as_f32(physical_size.width())? / logical_size.width();
    let scale_y = bounded_dimension_as_f32(physical_size.height())? / logical_size.height();
    let transform = Transform::from_scale(scale_x, scale_y);
    for command in presentation.commands() {
        match command {
            DrawCommand::FillRectangle(rectangle) => {
                fill_rectangle(&mut pixmap, *rectangle, transform)?;
            }
            DrawCommand::StrokeLine(line) => stroke_line(&mut pixmap, *line, transform)?,
            DrawCommand::FillCircle(circle) => fill_circle(&mut pixmap, *circle, transform)?,
            DrawCommand::Text(text) => draw_text(&mut pixmap, text, transform)?,
        }
    }

    if pixmap.data().len() != physical_size.rgba_byte_len() {
        return Err(RendererError::ImageByteCountOverflow);
    }
    let rgba_bytes = pixmap.data().to_vec();
    let png_bytes = pixmap
        .encode_png()
        .map_err(|_| RendererError::PngEncodingFailed)?;
    RenderedPixels::new(physical_size, rgba_bytes, png_bytes)
}

fn fill_rectangle(
    pixmap: &mut Pixmap,
    rectangle: Rectangle,
    transform: Transform,
) -> Result<(), RendererError> {
    let origin = rectangle.origin();
    let size = rectangle.size();
    let rect = tiny_skia::Rect::from_xywh(origin.x(), origin.y(), size.width(), size.height())
        .ok_or(RendererError::InvalidDrawing)?;
    let path = PathBuilder::from_rect(rect);
    let paint = paint(rectangle.color());
    pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
    Ok(())
}

fn stroke_line(pixmap: &mut Pixmap, line: Line, transform: Transform) -> Result<(), RendererError> {
    let mut builder = PathBuilder::new();
    builder.move_to(line.start().x(), line.start().y());
    builder.line_to(line.end().x(), line.end().y());
    let path = builder.finish().ok_or(RendererError::InvalidDrawing)?;
    let stroke = TinyStroke {
        width: line.stroke().width(),
        ..TinyStroke::default()
    };
    let paint = paint(line.stroke().color());
    pixmap.stroke_path(&path, &paint, &stroke, transform, None);
    Ok(())
}

fn fill_circle(
    pixmap: &mut Pixmap,
    circle: Circle,
    transform: Transform,
) -> Result<(), RendererError> {
    let path = PathBuilder::from_circle(circle.center().x(), circle.center().y(), circle.radius())
        .ok_or(RendererError::InvalidDrawing)?;
    let paint = paint(circle.color());
    pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
    Ok(())
}

fn draw_text(
    pixmap: &mut Pixmap,
    text: &TextDrawing,
    transform: Transform,
) -> Result<(), RendererError> {
    let glyph_height = text.size();
    let glyph_width = glyph_height * 5.0 / 7.0;
    let cell_width = glyph_width / 5.0;
    let cell_height = glyph_height / 7.0;
    let advance = glyph_width + cell_width;
    let mut origin_x = text.origin().x();
    for character in text.text().chars() {
        if character == ' ' {
            origin_x += advance;
            continue;
        }
        for row in 0..7_u8 {
            for column in 0..5_u8 {
                if !glyph_cell(character, row, column) {
                    continue;
                }
                let rectangle = tiny_skia::Rect::from_xywh(
                    origin_x + f32::from(column) * cell_width,
                    text.origin().y() + f32::from(row) * cell_height,
                    cell_width,
                    cell_height,
                )
                .ok_or(RendererError::InvalidDrawing)?;
                let path = PathBuilder::from_rect(rectangle);
                pixmap.fill_path(
                    &path,
                    &paint(text.color()),
                    FillRule::Winding,
                    transform,
                    None,
                );
            }
        }
        origin_x += advance;
    }
    Ok(())
}

fn glyph_cell(character: char, row: u8, column: u8) -> bool {
    if row == 0 || row == 6 || column == 0 || column == 4 {
        return true;
    }
    let code = u32::from(character);
    let bit = u32::from(row * 3 + column) % u32::BITS;
    code.rotate_left(u32::from(row)) & (1 << bit) != 0
}

fn bounded_dimension_as_f32(value: u32) -> Result<f32, RendererError> {
    let value = u16::try_from(value).map_err(|_| RendererError::DimensionLimitExceeded)?;
    Ok(f32::from(value))
}

fn paint(color: RgbaColor) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(to_tiny_color(color));
    paint.anti_alias = false;
    paint
}

fn to_tiny_color(color: RgbaColor) -> tiny_skia::Color {
    let [red, green, blue, alpha] = color.channels();
    tiny_skia::Color::from_rgba8(red, green, blue, alpha)
}
