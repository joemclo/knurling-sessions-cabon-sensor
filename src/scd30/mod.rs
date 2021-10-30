use crc_all::Crc;
use embedded_hal::blocking::i2c::{Read, Write};

pub struct SensorData {
    pub co2: f32,
    pub temperature: f32,
    pub humidity: f32,
}

const DEFAULT_ADDRESS: u8 = 0x61;

enum Command {
    StartContinuousMeasurement = 0x0010,
    StopContinuousMeasurement = 0x0104,
    MeasurementInterval = 0x4600,
    GetDataReadyStatus = 0x0202,
    ReadMeasurement = 0x0300,
    ASC = 0x5306,
    // FRC = 0x5204,
    TemperatureOffset = 0x5403,
    // AltitudeCompensation = 0x5102,
    ReadFirmwareVersion = 0xd100,
    SoftReset = 0xd304,
}

pub struct SCD30<T>(T);

impl<T, E> SCD30<T>
where
    T: Read<Error = E> + Write<Error = E>,
{
    pub fn init(i2c2: T) -> Self {
        SCD30(i2c2)
    }

    pub fn read_firmware_version(&mut self) -> Result<[u8; 2], E> {
        let mut rd_buffer = [0u8; 2];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadFirmwareVersion as u16).to_be_bytes(),
        )?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let major = u8::from_be(rd_buffer[0]);
        let minor = u8::from_be(rd_buffer[1]);

        Ok([major, minor])
    }

    pub fn soft_reset(&mut self) -> Result<(), E> {
        self.0
            .write(DEFAULT_ADDRESS, &(Command::SoftReset as u16).to_be_bytes())?;

        Ok(())
    }

    fn get_crc(&mut self) -> Crc<u8> {
        let crc = Crc::<u8>::new(0x31, 8, 0xFF, 0x00, false);

        crc
    }

    pub fn set_temperature_offset(&mut self, temperature_offset: u16) -> Result<(), E> {
        let temperature_offset_bytes: &[u8; 2] = &temperature_offset.to_be_bytes();

        let command: [u8; 2] = (Command::TemperatureOffset as u16).to_be_bytes();

        let mut command: [u8; 5] = [
            command[0],
            command[1],
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

    pub fn read_temperature_offset(&mut self) -> Result<u16, E> {
        let mut rd_buffer = [0u8; 3];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::TemperatureOffset as u16).to_be_bytes(),
        )?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(u16::from_be_bytes([rd_buffer[0], rd_buffer[1]]))
    }

    pub fn start_continuous_measurement(&mut self, pressure: &u16) -> Result<(), E> {
        let argument_bytes = &pressure.to_be_bytes();
        let mut crc = self.get_crc();
        crc.update(argument_bytes);

        let command = (Command::StartContinuousMeasurement as u16).to_be_bytes();

        let command: [u8; 5] = [
            command[0],
            command[1],
            argument_bytes[0],
            argument_bytes[1],
            crc.finish(),
        ];

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn stop_continuous_measurement(&mut self) -> Result<(), E> {
        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::StopContinuousMeasurement as u16).to_be_bytes(),
        )?;

        Ok(())
    }

    pub fn set_measurement_interval(&mut self, interval: u16) -> Result<(), E> {
        let argument_bytes = &interval.to_be_bytes();

        let mut crc = self.get_crc();
        crc.update(argument_bytes);

        let command = (Command::MeasurementInterval as u16).to_be_bytes();

        let command: [u8; 5] = [
            command[0],
            command[1],
            argument_bytes[0],
            argument_bytes[1],
            crc.finish(),
        ];

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn get_measurement_interval(&mut self) -> Result<u16, E> {
        let mut rd_buffer = [0u8; 3];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::MeasurementInterval as u16).to_be_bytes(),
        )?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(u16::from_be_bytes([rd_buffer[0], rd_buffer[1]]))
    }

    pub fn data_ready(&mut self) -> Result<bool, E> {
        let mut rd_buffer = [0u8; 3];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::GetDataReadyStatus as u16).to_be_bytes(),
        )?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(u16::from_be_bytes([rd_buffer[0], rd_buffer[1]]) == 1)
    }

    pub fn read_measurement(&mut self) -> Result<SensorData, E> {
        let mut rd_buffer = [0u8; 18];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadMeasurement as u16).to_be_bytes(),
        )?;
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

    pub fn activate_auto_self_calibration(&mut self) -> Result<bool, E> {
        let argument_bytes: [u8; 2] = [0x00, 0x01];
        let mut crc = self.get_crc();
        crc.update(&argument_bytes);

        let command = (Command::ASC as u16).to_be_bytes();

        let command: [u8; 5] = [
            command[0],
            command[1],
            argument_bytes[0],
            argument_bytes[1],
            crc.finish(),
        ];

        self.0.write(DEFAULT_ADDRESS, &command)?;

        self.0
            .write(DEFAULT_ADDRESS, &(Command::ASC as u16).to_be_bytes())?;

        let mut rd_buffer = [0u8; 3];

        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(u16::from_be_bytes([rd_buffer[0], rd_buffer[1]]) == 1)
    }

    pub fn free(self) -> T {
        self.0
    }

    pub fn alloc(&mut self, i2c2: T) -> () {
        self.0 = i2c2;

        ()
    }
}

