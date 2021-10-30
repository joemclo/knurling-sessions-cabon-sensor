use crc_all::Crc;
use embedded_hal::blocking::i2c::{Read, Write};

const DEFAULT_ADDRESS: u8 = 0x69;

enum Command {
    StartMeasurement = 0x0010,
    StopMeasurement = 0x0104,
    ReadDataReady = 0x0202,
    ReadMeasuredValue = 0x0300,
    StartFanCleaning = 0x5607,
    ReadProductType = 0xD002,
    ReadSerialNumber = 0xD033,
    ReadFirmwareVersion = 0xD100,
    ReadDeviceStatus = 0xD206,
    ClearDeviceStatus = 0xD210,
    Reset = 0xD304,
}

pub struct SPS30<T>(T);

fn calculate_new_index(index: usize) -> usize {
    (index / 3 * 2) + (index % 3)
}

fn get_bit_at(input: u32, n: u8) -> bool {

    if n < 32 {
        input & (1 << n) != 0
    } else {
        false
    }
}

fn get_device_status(input: u32, n: u8) -> &'static str {

    if get_bit_at(input, n) == false { "ok"} else {"error"}
}

fn remove_crc_bits(buffer: &[u8]) -> [u8; 60] {

    let mut new_buffer = [0u8; 60];

    for (index, byte) in buffer.iter().enumerate() {
        if (index + 1) % 3 != 0 {
            let new_index = calculate_new_index(index);
            new_buffer[new_index] = u8::from_be(*byte);
        }
    }

    new_buffer
}

impl<T, E> SPS30<T>
where
    T: Read<Error = E> + Write<Error = E>,
{
    pub fn init(i2c2: T) -> Self {
        SPS30(i2c2)
    }

    fn get_crc(&mut self) -> Crc<u8> {
        let crc = Crc::<u8>::new(0x31, 8, 0xFF, 0x00, false);

        crc
    }

    pub fn start_measurement(&mut self) -> Result<(), E> {
        let argument_bytes = &[0x03, 0x00];

        let mut crc = self.get_crc();
        crc.update(argument_bytes);

        let command = (Command::StartMeasurement as u16).to_be_bytes();

        let command: [u8; 5] = [
            command[0],
            command[1],
            argument_bytes[0],
            argument_bytes[1],
            crc.finish(),
        ];

        defmt::info!("start_measurement_command offset : {=[u8]}", command);

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn data_ready(&mut self) -> Result<bool, E> {
        let mut rd_buffer = [0u8; 3];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadDataReady as u16).to_be_bytes(),
        )?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        Ok(rd_buffer[1] == 0x01)
    }

    pub fn read_measurement(&mut self) -> Result<(), E> {
        let mut rd_buffer = [0u8; 60];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadMeasuredValue as u16).to_be_bytes(),
        )?;
        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let measurements = remove_crc_bits(&rd_buffer);

        defmt::info!("PM1.0 {=?}", f32::from_be_bytes([
            measurements[0],
            measurements[1],
            measurements[2],
            measurements[3],
        ]));

        defmt::info!("PM2.5 {=?}", f32::from_be_bytes([
            measurements[4],
            measurements[5],
            measurements[6],
            measurements[7],
        ]));

        Ok(())
    }

    pub fn start_fan_cleaning(&mut self) -> Result<(), E> {
        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::StartFanCleaning as u16).to_be_bytes(),
        )?;

        Ok(())
    }

    pub fn read_product_type(&mut self) -> Result<[u8; 32], E> {
        let mut rd_buffer = [0u8; 48];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadProductType as u16).to_be_bytes(),
        )?;

        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let mut product_type = [0u8; 32];
        for (index, byte) in rd_buffer.iter().enumerate() {
            if (index + 1) % 3 != 0 {
                let new_index = calculate_new_index(index);
                product_type[new_index] = u8::from_be(*byte);
            }
        }

        Ok(product_type)
    }

    pub fn read_serial_number(&mut self) -> Result<[u8; 60], E> {
        let mut rd_buffer = [0u8; 48];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadSerialNumber as u16).to_be_bytes(),
        )?;

        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let serial_number = remove_crc_bits(&rd_buffer);
        Ok(serial_number)
    }

    pub fn read_device_status(&mut self) -> Result<bool, E> {
        let mut rd_buffer = [0u8; 6];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadDeviceStatus as u16).to_be_bytes(),
        )?;

        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let device_status =
            u32::from_be_bytes([rd_buffer[0], rd_buffer[1], rd_buffer[3], rd_buffer[4]]);

        defmt::info!("device status");

        defmt::info!("Speed State {=?}", get_device_status(device_status, 21));
        defmt::info!("Lazer State {=?}", get_device_status(device_status, 5));
        defmt::info!("Fan State {=?}",  get_device_status(device_status, 4));

        Ok(true)
    }

    pub fn clear_device_status(&mut self) -> Result<(), E> {
        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ClearDeviceStatus as u16).to_be_bytes(),
        )?;

        Ok(())
    }


    pub fn read_firmware_version(&mut self) -> Result<[u8; 2], E> {
        let mut rd_buffer = [0u8; 3];

        self.0.write(
            DEFAULT_ADDRESS,
            &(Command::ReadFirmwareVersion as u16).to_be_bytes(),
        )?;

        self.0.read(DEFAULT_ADDRESS, &mut rd_buffer)?;

        let major = u8::from_be(rd_buffer[0]);
        let minor = u8::from_be(rd_buffer[1]);

        Ok([major, minor])
    }

    pub fn reset_device(&mut self) -> Result<(), E> {
        self.0
            .write(DEFAULT_ADDRESS, &(Command::Reset as u16).to_be_bytes())?;

        Ok(())
    }

    pub fn free(self) -> T {
        self.0
    }

    pub fn alloc(&mut self, i2c2: T) -> () {
        self.0 = i2c2;

        ()
    }
}
