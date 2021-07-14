use crc_all::Crc;
use nrf52840_hal::{
    pac::TIMER1,
    prelude::*,
    timer::OneShot,
    twim::{Error, Instance, Twim},
    Timer,
};

pub struct SensorData {
    pub co2: f32,
    pub temperature: f32,
    pub humidity: f32,
}

pub const DEFAULT_ADDRESS: u8 = 0x61;
pub struct SCD30<T: Instance>(Twim<T>);

impl<T> SCD30<T>
where
    T: Instance,
{
    pub fn init(i2c2: Twim<T>) -> Self {
        SCD30(i2c2)
    }

    pub fn get_firmware_version(&mut self) -> Result<[u8; 2], Error> {
        let command: [u8; 2] = [0xd1, 0x00];
        let mut rd_buffer = [0u8; 2];

        self.0.write(DEFAULT_ADDRESS, &command)?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let major = u8::from_be(rd_buffer[0]);
        let minor = u8::from_be(rd_buffer[1]);

        Ok([major, minor])
    }

    pub fn soft_reset(&mut self) -> Result<(), Error> {
        let command: [u8; 2] = [0xd3, 0x04];

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    fn get_crc(&mut self) -> Crc<u8> {
        let crc = Crc::<u8>::new(0x31, 8, 0xFF, 0x00, false);

        crc
    }

    pub fn set_temperature_offset(&mut self, temperature_offset: u16) -> Result<(), Error> {
        let temperature_offset_bytes: &[u8; 2] = &temperature_offset.to_be_bytes();

        let mut command: [u8; 5] = [
            0x54,
            0x03,
            temperature_offset_bytes[0],
            temperature_offset_bytes[1],
            0x00,
        ];

        let mut crc = self.get_crc();
        crc.update(temperature_offset_bytes);

        command[4] = crc.finish();

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn read_temperature_offset(&mut self) -> Result<u16, Error> {
        let command: [u8; 2] = [0x54, 0x03];
        let mut rd_buffer = [0u8; 3];

        self.0.write(DEFAULT_ADDRESS, &command)?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(u16::from_be_bytes([rd_buffer[0], rd_buffer[1]]))
    }

    pub fn start_continuous_measurement(&mut self, pressure: &u16) -> Result<(), Error> {
        let argument_bytes = &pressure.to_be_bytes();
        let mut crc = self.get_crc();
        crc.update(argument_bytes);

        let command: [u8; 5] = [
            0x00,
            0x10,
            argument_bytes[0],
            argument_bytes[1],
            crc.finish(),
        ];

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn stop_continuous_measurement(&mut self) -> Result<(), Error> {
        let command: [u8; 2] = [0x01, 0x04];

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn set_measurement_interval(&mut self, interval: u16) -> Result<(), Error> {
        let argument_bytes = &interval.to_be_bytes();

        let mut crc = self.get_crc();
        crc.update(argument_bytes);

        let command: [u8; 5] = [
            0x46,
            0x00,
            argument_bytes[0],
            argument_bytes[1],
            crc.finish(),
        ];

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn get_measurement_interval(&mut self) -> Result<u16, Error> {
        let command: [u8; 2] = [0x46, 0x00];
        let mut rd_buffer = [0u8; 3];

        self.0.write(DEFAULT_ADDRESS, &command)?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(u16::from_be_bytes([rd_buffer[0], rd_buffer[1]]))
    }

    pub fn data_ready(&mut self, timer: &mut Timer<TIMER1, OneShot>) -> Result<bool, Error> {
        let command: [u8; 2] = [0x02, 0x02];
        let mut rd_buffer = [0u8; 3];

        self.0.write(DEFAULT_ADDRESS, &command)?;
        timer.delay_ms(10_u32);
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(u16::from_be_bytes([rd_buffer[0], rd_buffer[1]]) == 1)
    }

    pub fn read_measurement(&mut self) -> Result<SensorData, Error> {
        let command: [u8; 2] = [0x3, 0x00];
        let mut rd_buffer = [0u8; 18];

        self.0.write(DEFAULT_ADDRESS, &command)?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let sensor_data = SensorData {
            co2: f32::from_be_bytes([rd_buffer[0], rd_buffer[1], rd_buffer[3], rd_buffer[4]]),
            temperature: f32::from_be_bytes([
                rd_buffer[6],
                rd_buffer[7],
                rd_buffer[9],
                rd_buffer[10],
            ]),
            humidity: f32::from_be_bytes([
                rd_buffer[12],
                rd_buffer[13],
                rd_buffer[15],
                rd_buffer[16],
            ]),
        };

        Ok(sensor_data)
    }
}
