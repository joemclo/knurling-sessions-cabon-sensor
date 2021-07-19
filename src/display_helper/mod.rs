use embedded_graphics::{
    egtext,
    fonts::{Font12x16, Font24x32, Text},
    geometry::Point,
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyle,
    text_style,
};
use epd_waveshare::epd4in2::*;

fn draw_large_text(display: &mut Display4in2, text: &str, point: Point) -> () {
    Text::new(text, point)
        .into_styled(TextStyle::new(Font24x32, BinaryColor::On))
        .draw(display)
        .unwrap();
}

fn draw_mid_text(display: &mut Display4in2, text: &str, point: Point) -> () {
    Text::new(text, point)
        .into_styled(TextStyle::new(Font12x16, BinaryColor::On))
        .draw(display)
        .unwrap();
}

pub fn draw_text(mut display: Display4in2) -> Display4in2 {
    draw_large_text(&mut display, "Air Quality", Point::new(20, 30));

    draw_mid_text(&mut display, "Carbon Dioxide:", Point::new(20, 90));
    draw_mid_text(&mut display, "Temperature:", Point::new(20, 130));
    draw_mid_text(&mut display, "Humidity:", Point::new(20, 170));

    display
}
