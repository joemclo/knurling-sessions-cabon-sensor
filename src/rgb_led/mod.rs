use nrf52840_hal::{
    gpio::{Level, Output, Pin, PushPull},
    pac::TIMER1,
    prelude::*,
    timer::OneShot,
    Timer,
};

pub struct LEDColour {
    r: Pin<Output<PushPull>>,
    b: Pin<Output<PushPull>>,
    g: Pin<Output<PushPull>>,
}

impl LEDColour {
    pub fn init<Mode>(led_red: Pin<Mode>, led_blue: Pin<Mode>, led_green: Pin<Mode>) -> LEDColour {
        LEDColour {
            r: led_red.into_push_pull_output(Level::High),
            b: led_blue.into_push_pull_output(Level::High),
            g: led_green.into_push_pull_output(Level::High),
        }
    }

    pub fn off(&mut self) {
        self.r.set_high().unwrap();
        self.b.set_high().unwrap();
        self.g.set_high().unwrap();
    }

    pub fn red(&mut self) {
        self.r.set_low().unwrap();
        self.g.set_high().unwrap();
        self.b.set_high().unwrap();
    }

    pub fn blue(&mut self) {
        self.r.set_high().unwrap();
        self.g.set_high().unwrap();
        self.b.set_low().unwrap();
    }

    pub fn green(&mut self) {
        self.r.set_high().unwrap();
        self.g.set_low().unwrap();
        self.b.set_high().unwrap();
    }

    pub fn yellow(&mut self) {
        self.r.set_low().unwrap();
        self.b.set_high().unwrap();
        self.g.set_low().unwrap();
    }

    pub fn white(&mut self) {
        self.r.set_low().unwrap();
        self.g.set_low().unwrap();
        self.b.set_low().unwrap();
    }

    pub fn blink(&mut self, timer: &mut Timer<TIMER1, OneShot>) {
        let current_red = self.r.is_set_high().unwrap();
        let current_green = self.g.is_set_high().unwrap();
        let current_blue = self.b.is_set_high().unwrap();

        self.white();
        timer.delay_ms(100_u32);

        let result = if current_red {
            self.r.set_high()
        } else {
            self.r.set_low()
        };
        result.unwrap();

        let result = if current_green {
            self.g.set_high()
        } else {
            self.g.set_low()
        };
        result.unwrap();

        let result = if current_blue {
            self.b.set_high()
        } else {
            self.b.set_low()
        };
        result.unwrap();
    }
}
