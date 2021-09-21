use arrayvec::ArrayVec;
use hal::{pac::UARTE0, prelude::*};
use sensirion_hdlc::{decode, encode, SpecialChars};

use nrf52840_hal::{
    self as hal,
    uarte::{Error, Uarte},
};

const ADDRESS: u8 = 0x0;

enum Command {
    StartMeasurement = 0x00,
    StopMeasurement = 0x01,
    ReadMeasurement = 0x03,
    Sleep = 0x10,
    Wake = 0x11,
    FanClean = 0x56,
    AutoCleanInterval = 0x80,
    DeviceInfo = 0xD0,
    ReadVersion = 0xD1,
    ReadStatus = 0xD2,
    Reset = 0xD3,
}

enum ErrorCodes {
    NoError = 0x00,
    WrongData = 0x01,
    UnknownCommand = 0x02,
    NoAccessRight = 0x03,
    IllegalParameter = 0x04,
    InternalFuncError = 0x28,
    CommandStateForbidden = 0x43,
}

fn compute_checksum(data: &[u8]) -> u8 {
    let mut checksum: u8 = 0;
    for &byte in data.iter() {
        let val: u16 = checksum as u16 + byte as u16;
        let lsb = val % 256;
        checksum = lsb as u8;
    }

    255 - checksum
}

pub struct SPS30(Uarte<UARTE0>);

impl SPS30 {
    pub fn init(serial: Uarte<UARTE0>) -> Self {
        SPS30(serial)
    }

    pub fn send_uart_data(&mut self, data: &[u8]) -> Result<(), Error> {
        let encoded = encode(&data, SpecialChars::default()).unwrap();

        for x in encoded.iter() {
            defmt::info!("byte {=u8}", x);
        }

        self.0.write(&encoded)
    }

    pub fn read_uart_data(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.0.read(buffer)
    }

    pub fn version(&mut self) -> Result<[u8; 2], Error> {
        let mut data = ArrayVec::<[u8; 100]>::new();

        let tx_length: u8 = 0x00;
        let command = [ADDRESS, Command::ReadVersion as u8, tx_length];

        for word in &command {
            data.push(*word);
        }
        let checksum = compute_checksum(&command);
        data.push(checksum);

        defmt::info!("Sending");

        self.send_uart_data(&data)?;

        defmt::info!("Sent");

        let mut buffer = [0x00, 0x00];

        self.0.read(&mut buffer).unwrap();

        for x in buffer.iter() {
            defmt::info!("byte {=u8}", x);
        }

        Ok([0x34, 0x34])
    }
}
