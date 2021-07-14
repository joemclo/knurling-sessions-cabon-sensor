pub enum Unit {
    Fahrenheit,
    Celsius,
    Kelvin,
}

impl Unit {
    pub fn convert_temperature(&self, temperature: &f32) -> f32 {
        match self {
            Unit::Fahrenheit => (temperature * 9.0 / 5.0) + 32.0,
            Unit::Kelvin => temperature + 273.15,
            Unit::Celsius => *temperature,
        }
    }
}
