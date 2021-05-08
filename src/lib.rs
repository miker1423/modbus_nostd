#![no_std]

use core::convert::Into;
use embedded_hal::serial::Read;
use crate::ring_buffer::RingBuffer;

pub mod coils;
pub mod registers;
mod ring_buffer;

pub struct ModbusClient<'a> {
    buffer: RingBuffer<'a, u8>
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
    pub fn new(buffer: &'a mut [u8; 253]) -> ModbusClient<'a> {
        ModbusClient {
            buffer: RingBuffer::new(buffer)
        }
    }

    pub fn write_registers_from(self, start_address: Address) -> registers::WriteRegisterModbusClient<'a> {
        registers::WriteRegisterModbusClient::new(start_address, self.buffer)
    }

    pub fn read_register_from(self, start_address: Address) -> registers::ReadRegisterModbusClient<'a> {
        registers::ReadRegisterModbusClient::new(start_address, self.buffer)
    }

    pub fn write_coil_from(self, start_address: Address) -> coils::WriteCoilModbusClient<'a> {
        coils::WriteCoilModbusClient::new(start_address, self.buffer)
    }

    pub fn read_coil_from(self, start_address: Address) -> coils::ReadCoilModbusClient<'a> {
        coils::ReadCoilModbusClient::new(start_address, self.buffer)
    }
}

fn read_response<'a, W, R>(id: u8, quantity: u16, reader: &mut impl Read<u8, Error = R>, buffer: &'a mut RingBuffer<'a, u8>)
    -> Result<(usize, &'a [u8]), Error<W, R>> {
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

    let byte_count =
        match id {
            0x05 | 0x06 | 0x0F | 0x10 => 4,
            0x04 => 2 * quantity,
            0x01 => match quantity % 8 {
                0 => (quantity / 8) + 1,
                _ => (quantity / 8)
            }
            _ => 0x00
        };

    buffer.clear();
    buffer.push_single(read_value);
    for _ in 0..byte_count {
        match nb::block!(reader.read()) {
            Ok(data) => buffer.push_single(data),
            Err(e) => return Err(Error::UartReadErr(e)),
        };
    }

    let slice = buffer.get_written();
    Ok((buffer.len(), slice))
}
