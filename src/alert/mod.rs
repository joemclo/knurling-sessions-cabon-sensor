use nrf52840_hal::{pac::TIMER1, timer::OneShot, Timer};

use crate::{buzzer::Buzzer, rgb_led::LEDColour};

pub struct CO2alert {
    warning_level_1: f32,
    warning_level_2: f32,
    limit_level: f32,
    buzzer_count: u16,
}

impl CO2alert {
    pub fn init(warning_level_1: f32, warning_level_2: f32, limit_level: f32) -> CO2alert {
        CO2alert {
            warning_level_1,
            warning_level_2,
            limit_level,
            buzzer_count: 0,
        }
    }

    pub fn check_level(
        &mut self,
        current_level: &f32,
        buzzer: &mut Buzzer,
        led: &mut LEDColour,
        mut timer: &mut Timer<TIMER1, OneShot>,
    ) {
        if *current_level > self.limit_level {
            led.red();
            if self.buzzer_count < 5 {
                buzzer.buzz(&mut timer);
                self.buzzer_count += 1;
            }
        } else if *current_level > self.warning_level_2 {
            led.yellow();
        } else if *current_level > self.warning_level_1 {
            led.blue();
        } else {
            led.green();
            self.buzzer_count = 0;
        }
    }
}
