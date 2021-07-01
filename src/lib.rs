#![no_std]

use core::convert::Into;
use embedded_hal::serial::Read;
use crate::ring_buffer::RingBuffer;
use crc16::{State, MODBUS};

pub mod coils;
pub mod registers;
mod ring_buffer;

pub struct ModbusClient<'a> {
    server_address: ServerAddress,
    buffer: RingBuffer<'a, u8>,
}

#[derive(Copy, Clone)]
pub struct ServerAddress(pub u8);
impl Into<ServerAddress> for u8 {
    fn into(self) -> ServerAddress {
        ServerAddress(self)
    }
}

pub struct Address(pub u16);

impl Into<Address> for u16 {
    fn into(self) -> Address {
        Address(self)
    }
}

pub enum Error<W, R> {
    Modbus(ModbusError),
    UartReadErr(R),
    UartWriteErr(W)
}

pub enum ModbusError {
    NotSupportedFunction,
    StartAddressOrQuantityInvalid,
    AddressInvalid,
    TypeInvalid,
    Unknown,
}

fn byte_to_error(code: u8) -> ModbusError {
    match code {
        1 => ModbusError::NotSupportedFunction,
        2 => ModbusError::StartAddressOrQuantityInvalid,
        3 => ModbusError::AddressInvalid,
        4 => ModbusError::TypeInvalid,
        _ => ModbusError::Unknown
    }
}

impl<'a> ModbusClient<'a> {
    pub fn new(buffer: &'a mut [u8; 256], server_address: ServerAddress) -> ModbusClient<'a> {
        ModbusClient {
            server_address,
            buffer: RingBuffer::new(buffer),
        }
    }

    pub fn write_registers_from(self, start_address: Address) -> registers::WriteRegisterModbusClient<'a> {
        registers::WriteRegisterModbusClient::new(self.server_address, start_address, self.buffer)
    }

    pub fn read_register_from(self, start_address: Address) -> registers::ReadRegisterModbusClient<'a> {
        registers::ReadRegisterModbusClient::new(self.server_address, start_address, self.buffer)
    }

    pub fn write_coil_from(self, start_address: Address) -> coils::WriteCoilModbusClient<'a> {
        coils::WriteCoilModbusClient::new(self.server_address, start_address, self.buffer)
    }

    pub fn read_coil_from(self, start_address: Address) -> coils::ReadCoilModbusClient<'a> {
        coils::ReadCoilModbusClient::new(self.server_address, start_address, self.buffer)
    }
}

fn read_response<'a, W, R>(id: u8, quantity: u16, reader: &mut impl Read<u8, Error = R>, buffer: &'a mut RingBuffer<'a, u8>)
    -> Result<(usize, &'a [u8]), Error<W, R>> {
    let received_id = nb::block!(reader.read());
    if let Err(err) = received_id {
        return Err(Error::UartReadErr(err));
    }
    let received_id = received_id.unwrap_or_default();

    let read_value = nb::block!(reader.read());
    if let Err(err) = read_value {
        return Err(Error::UartReadErr(err));
    }

    let read_value = read_value.unwrap_or_default();
    if id + 0x80 == read_value {
        return match nb::block!(reader.read()) {
            Ok(err_code) => Err(Error::Modbus(byte_to_error(err_code))),
            Err(e) => Err(Error::UartReadErr(e))
        }
    }

    let size = nb::block!(reader.read());
    if let Err(err) = size {
        return Err(Error::UartReadErr(err));
    }
    let size = size.unwrap_or_default();

    let byte_count =
        match id {
            0x05 | 0x06 | 0x0F | 0x10 => 4,
            0x04 => 2 * quantity,
            0x01 => match quantity % 8 {
                0 => (quantity / 8),
                _ => (quantity / 8) + 1
            }
            _ => 0x00
        };

    let byte_count = byte_count + 2;
    buffer.clear();
    buffer.push_single(received_id);
    buffer.push_single(read_value);
    buffer.push_single(size);
    for _ in 0..byte_count {
        match nb::block!(reader.read()) {
            Ok(data) => buffer.push_single(data),
            Err(e) => return Err(Error::UartReadErr(e)),
        };
    }

    let slice = buffer.get_written();
    Ok((buffer.len(), slice))
}

fn add_crc<'a>(buffer: &'a RingBuffer<'a, u8>) -> u16{
    let written_buffer = buffer.get_written();
    State::<MODBUS>::calculate(written_buffer)
}