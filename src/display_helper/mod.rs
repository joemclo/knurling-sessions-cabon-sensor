use arrayvec::ArrayString;
use core::fmt::Write;
use embedded_graphics::{
    egtext,
    fonts::{Font, Font12x16, Font24x32, Font6x12},
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

fn draw_small_text(display: &mut Display4in2, text: &str, position: (i32, i32)) -> () {
    draw_text(display, text, position, Font6x12);
}

pub fn draw_titles(mut display: Display4in2) -> Display4in2 {
    draw_large_text(&mut display, "Air Quality", (20, 30));

    draw_mid_text(&mut display, "Carbon Dioxide:", (20, 90));
    draw_mid_text(&mut display, "Temperature:", (20, 130));
    draw_mid_text(&mut display, "Humidity:", (20, 170));

    draw_small_text(&mut display, "Counter:", (20, 250));
    draw_small_text(&mut display, "Counter:", (20, 270));

    display
}

pub fn draw_large(mut display: Display4in2, text: &str, position: (i32, i32)) -> Display4in2 {
    draw_large_text(&mut display, text, position);

    display
}

pub fn draw_medium(mut display: Display4in2, text: &str, position: (i32, i32)) -> Display4in2 {
    draw_mid_text(&mut display, text, position);

    display
}

pub fn draw_small(mut display: Display4in2, text: &str, position: (i32, i32)) -> Display4in2 {
    draw_small_text(&mut display, text, position);

    display
}

pub fn draw_time(count: i32, position: (i32, i32), mut display: Display4in2) -> Display4in2 {
    let mut buf = ArrayString::<[_; 30]>::new();

    write!(&mut buf, "{} seconds elapsed", count).expect("Failed to write to buffer");

    draw_small_text(&mut display, &buf, position);

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

    draw_mid_text(&mut display, &buf, position);

    display
}

pub fn clear_screen(mut display: Display4in2) -> Display4in2 {
    Rectangle::new(Point::new(0, 0), Point::new(400, 300))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
        .draw(&mut display)
        .unwrap();
    display
}
