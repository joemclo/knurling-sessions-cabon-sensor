#![no_main]
#![no_std]

use carbon_sensor::{
    self as _, alert, buzzer, dk_button, number_representations::Unit, rgb_led, scd30,
};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    prelude::{Point, Primitive},
    primitives::{Circle, Triangle},
    style::PrimitiveStyle,
};
use epd_waveshare::{epd4in2::*, prelude::*};
// global logger + panicking-behavior + memory layout
use nb::block;
use nrf52840_hal::{
    self as hal,
    gpio::{p0, p1, Level},
    prelude::*,
    spim::{self, Spim},
    twim::{self, Twim},
    Temp, Timer,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = hal::pac::Peripherals::take().unwrap();
    let mut periodic_timer = Timer::periodic(board.TIMER0);
    let mut one_shot_timer = Timer::one_shot(board.TIMER1);
    let mut delay = Timer::new(board.TIMER3);

    let mut millis: u64 = 0;

    let pins_0 = p0::Parts::new(board.P0);
    let pins_1 = p1::Parts::new(board.P1);

    let din = pins_1.p1_01.into_push_pull_output(Level::Low).degrade();
    let clk = pins_1.p1_02.into_push_pull_output(Level::Low).degrade();
    let cs = pins_1.p1_03.into_push_pull_output(Level::Low);
    let dc = pins_1.p1_04.into_push_pull_output(Level::Low);
    let rst = pins_1.p1_05.into_push_pull_output(Level::Low);
    let busy = pins_1.p1_06.into_floating_input();

    let spi_pins = spim::Pins {
        sck: clk,
        miso: None,
        mosi: Some(din),
    };

    let mut spi = Spim::new(
        board.SPIM3,
        spi_pins,
        spim::Frequency::K500,
        spim::MODE_0,
        0,
    );

    let mut epd4in2 = EPD4in2::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

    let mut display = Display4in2::default();

    let led_channel_red = pins_0.p0_03.degrade();
    let led_channel_blue = pins_0.p0_04.degrade();
    let led_channel_green = pins_0.p0_28.degrade();

    let mut light = rgb_led::LEDColour::init(led_channel_red, led_channel_blue, led_channel_green);

    let mut buzzer = buzzer::Buzzer::init(pins_0.p0_29.degrade());

    let mut co2_alert = alert::CO2alert::init(500_f32, 700_f32, 1000_f32);

    let scl = pins_0.p0_30.degrade();
    let sda = pins_0.p0_31.degrade();
    let twim_pins = twim::Pins { scl, sda };
    let i2c = Twim::new(board.TWIM0, twim_pins, twim::Frequency::K100);

    let mut sensor = scd30::SCD30::init(i2c);

    one_shot_timer.delay_ms(100_u32); // delay to allow sensors to boot

    let firmware_version = sensor.read_firmware_version().unwrap();

    defmt::info!(
        "Firmware Version: {=u8}.{=u8}",
        firmware_version[0],
        firmware_version[1]
    );

    let temperature_offset = sensor.read_temperature_offset().unwrap();
    defmt::info!("Temperature offset : {=u16}", temperature_offset);

    let mut button_1 = dk_button::Button::new(pins_0.p0_11.degrade());
    let mut button_2 = dk_button::Button::new(pins_0.p0_12.degrade());
    let mut button_3 = dk_button::Button::new(pins_0.p0_24.degrade());
    let mut button_4 = dk_button::Button::new(pins_0.p0_25.degrade());

    let mut temp = Temp::new(board.TEMP);

    let mut current_unit = Unit::Celsius;
    let mut temperature;

    light.white();
    one_shot_timer.delay_ms(500_u32);
    light.blue();
    one_shot_timer.delay_ms(500_u32);
    light.red();
    one_shot_timer.delay_ms(500_u32);
    light.green();

    Circle::new(Point::new(171, 110), 30)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();
    Circle::new(Point::new(229, 110), 30)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();
    Triangle::new(
        Point::new(259, 120),
        Point::new(141, 120),
        Point::new(200, 200),
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
    .draw(&mut display)
    .unwrap();

    epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
    epd4in2
        .display_frame(&mut spi)
        .expect("display frame new graphics");

    loop {
        periodic_timer.start(1000u32);

        if (millis % 1000) == 0 {
            defmt::info!("Tick (milliseconds): {=u64}", millis);
            temperature = temp.measure().to_num();
            let converted_temp = current_unit.convert_temperature(&temperature);

            let unit = match current_unit {
                Unit::Fahrenheit => "°F",
                Unit::Kelvin => "K",
                Unit::Celsius => "°C",
            };

            light.blink(&mut one_shot_timer);

            defmt::info!("{=f32} {}", converted_temp, unit);

            if sensor.data_ready().unwrap() {
                defmt::info!("Sensor Data ready.");
                one_shot_timer.delay_ms(50_u32);
                light.blink(&mut one_shot_timer);

                let measurement_interval = sensor.get_measurement_interval().unwrap();

                defmt::info!("measurement_interval: {}", measurement_interval);

                let result = sensor.read_measurement().unwrap();

                let co2 = result.co2;
                let temp = result.temperature;
                let humidity = result.humidity;

                defmt::info!(
                    "
                CO2 {=f32} ppm
                Temperature {=f32} °C
                Humidity {=f32} %
                ",
                    co2,
                    temp,
                    humidity
                );

                co2_alert.check_level(&co2, &mut buzzer, &mut light, &mut one_shot_timer);
            } else {
                defmt::info!("Sensor Data Not Ready.");
            }
        };

        if (millis % 5) == 0 {
            if button_1.check_rising_edge() {
                current_unit = match current_unit {
                    Unit::Fahrenheit => Unit::Kelvin,
                    Unit::Kelvin => Unit::Celsius,
                    Unit::Celsius => Unit::Fahrenheit,
                };

                light.blink(&mut one_shot_timer);
                defmt::info!("Unit changed");
            }

            if button_2.check_rising_edge() {
                sensor.stop_continuous_measurement().unwrap();

                light.blink(&mut one_shot_timer);
                defmt::info!("Stop continuous measurement");
            }

            if button_3.check_rising_edge() {
                sensor.set_measurement_interval(2_u16).unwrap();
                sensor.set_temperature_offset(0_u16).unwrap();

                let air_pressure_london = 1012_u16;
                sensor
                    .start_continuous_measurement(&air_pressure_london)
                    .unwrap();

                defmt::info!(
                    "Temperature offset : {=u16}",
                    sensor.read_temperature_offset().unwrap()
                );

                defmt::info!(
                    "Measurement interval : {=u16}",
                    sensor.get_measurement_interval().unwrap()
                );

                light.blink(&mut one_shot_timer);
            }

            if button_4.check_rising_edge() {
                sensor.soft_reset().unwrap();
                defmt::info!("Sensor reset");
                one_shot_timer.delay_ms(50_u32);
                let auto_status = sensor.activate_auto_self_calibration().unwrap();
                defmt::info!("Auto Calib Status, {}", auto_status);

                light.blink(&mut one_shot_timer);
            }
        }

        block!(periodic_timer.wait()).unwrap();
        millis = millis.saturating_add(1);
    }
}
