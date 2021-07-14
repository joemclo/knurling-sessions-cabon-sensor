use nrf52840_hal::{
    gpio::{Level, Output, Pin, PushPull},
    pac::TIMER1,
    prelude::*,
    timer::OneShot,
    Timer,
};

pub struct Buzzer {
    pin: Pin<Output<PushPull>>,
}

impl Buzzer {
    pub fn init<Mode>(pin: Pin<Mode>) -> Buzzer {
        Buzzer {
            pin: pin.into_push_pull_output(Level::Low),
        }
    }

    pub fn high(&mut self) {
        self.pin.set_high().unwrap();
    }

    pub fn low(&mut self) {
        self.pin.set_low().unwrap();
    }

    pub fn buzz(&mut self, timer: &mut Timer<TIMER1, OneShot>) {
        for _ in 1..250 {
            self.low();
            timer.delay_ms(3_u32);

            self.high();
            timer.delay_ms(3_u32);
        }
    }
}
