use arrayvec::ArrayString;
use core::fmt::Write;
use embedded_graphics::{
    egtext,
    fonts::{Font, Font12x16, Font24x32},
    geometry::Point,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    style::PrimitiveStyle,
    text_style,
};
use epd_waveshare::epd4in2::*;

fn draw_text<F>(display: &mut Display4in2, text: &str, position: (i32, i32), font: F) -> ()
where
    F: Font + Clone + Copy,
{
    egtext!(
        text = text,
        top_left = position,
        style = text_style!(font = font, text_color = BinaryColor::On,)
    )
    .draw(display)
    .unwrap();
}

fn draw_large_text(display: &mut Display4in2, text: &str, position: (i32, i32)) -> () {
    draw_text(display, text, position, Font24x32);
}

fn draw_mid_text(display: &mut Display4in2, text: &str, position: (i32, i32)) -> () {
    draw_text(display, text, position, Font12x16);
}

pub fn draw_titles(mut display: Display4in2) -> Display4in2 {
    draw_large_text(&mut display, "Air Quality", (20, 30));

    draw_mid_text(&mut display, "Carbon Dioxide:", (20, 90));
    draw_mid_text(&mut display, "Temperature:", (20, 130));
    draw_mid_text(&mut display, "Humidity:", (20, 170));

    display
}

pub fn draw_numbers(
    value: f32,
    unit: &str,
    position: (i32, i32),
    mut display: Display4in2,
) -> Display4in2 {
    let mut buf = ArrayString::<[_; 12]>::new();

    write!(&mut buf, "{:.2} {}", value, unit).expect("Failed to write to buffer");

    egtext!(
        text = &buf,
        top_left = position,
        style = text_style!(font = Font12x16, text_color = BinaryColor::On,)
    )
    .draw(&mut display)
    .unwrap();

    display
}

pub fn clear_numbers(
    mut display: Display4in2,
    top_left: (i32, i32),
    bottom_right: (i32, i32),
) -> Display4in2 {
    Rectangle::new(
        Point::new(top_left.0, top_left.1),
        Point::new(bottom_right.0, bottom_right.1),
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
    .draw(&mut display)
    .unwrap();

    display
}
