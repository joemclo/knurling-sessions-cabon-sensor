#![no_main]
#![no_std]

use carbon_sensor::{
    self as _, alert, buzzer, dk_button, number_representations::Unit, rgb_led, scd30,
};
// global logger + panicking-behavior + memory layout
use nb::block;
use nrf52840_hal::{
    self as hal,
    gpio::p0,
    prelude::*,
    twim::{self, Twim},
    Temp, Timer,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = hal::pac::Peripherals::take().unwrap();
    let mut periodic_timer = Timer::periodic(board.TIMER0);
    let mut one_shot_timer = Timer::one_shot(board.TIMER1);

    let mut millis: u64 = 0;

    let pins = p0::Parts::new(board.P0);

    let led_channel_red = pins.p0_03.degrade();
    let led_channel_blue = pins.p0_04.degrade();
    let led_channel_green = pins.p0_29.degrade();

    let mut light = rgb_led::LEDColour::init(led_channel_red, led_channel_blue, led_channel_green);

    let mut buzzer = buzzer::Buzzer::init(pins.p0_28.degrade());

    let mut co2_alert = alert::CO2alert::init(700_f32, 1000_f32);

    let scl = pins.p0_30.degrade();
    let sda = pins.p0_31.degrade();
    let twim_pins = twim::Pins { scl, sda };
    let i2c = Twim::new(board.TWIM0, twim_pins, twim::Frequency::K100);

    let mut sensor = scd30::SCD30::init(i2c);

    one_shot_timer.delay_ms(100_u32); // delay to allow sensors to boot

    let firmware_version = sensor.get_firmware_version().unwrap();

    defmt::info!(
        "Firmware Version: {=u8}.{=u8}",
        firmware_version[0],
        firmware_version[1]
    );

    let temperature_offset = sensor.read_temperature_offset().unwrap();
    defmt::info!("Temperature offset : {=u16}", temperature_offset);

    let mut button_1 = dk_button::Button::new(pins.p0_11.degrade());
    let mut button_2 = dk_button::Button::new(pins.p0_12.degrade());
    let mut button_3 = dk_button::Button::new(pins.p0_24.degrade());
    let mut button_4 = dk_button::Button::new(pins.p0_25.degrade());

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

            if sensor.data_ready(&mut one_shot_timer).unwrap() {
                defmt::info!("Sensor Data ready.");
                light.blue();
                one_shot_timer.delay_ms(50_u32);

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
                sensor.set_measurement_interval(10_u16).unwrap();
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
                light.blink(&mut one_shot_timer);
            }
        }

        block!(periodic_timer.wait()).unwrap();
        millis = millis.saturating_add(1);
    }
}
