#![no_std]
#![feature(alloc)]

extern crate alloc;

use core::convert::Into;
use core::cell::RefCell;
use embedded_hal::serial::Read;

pub mod coils;
pub mod registers;

static BUFFER: RefCell<[u8; 253]> = RefCell::new([0; 253]);

pub struct ModbusClient;

pub struct Address(pub u16);

impl Into<Address> for u16 {
    fn into(self) -> Address {
        Address(self)
    }
}

pub enum ModbusError<R, W> {
    NotSupportedFunction,
    StartAddressOrQuantityInvalid,
    AddressInvalid,
    TypeInvalid,
    Unknown,
    ReadErr(R),
    WriteErr(W)
}

fn byte_to_error(code: u8) -> ModbusError<(), ()> {
    match code {
        1 => ModbusError::NotSupportedFunction,
        2 => ModbusError::StartAddressOrQuantityInvalid,
        3 => ModbusError::AddressInvalid,
        4 => ModbusError::TypeInvalid,
        _ => ModbusError::Unknown
    }
}

impl ModbusClient {
    pub fn write_registers_from(self, start_address: Address) -> registers::WriteRegisterModbusClient {
        registers::WriteRegisterModbusClient::new(start_address)
    }

    pub fn read_register_from(self, start_address: Address) -> registers::ReadRegisterModbusClient {
        registers::ReadRegisterModbusClient::new(start_address)
    }

    pub fn write_coil_from(self, start_address: Address) -> coils::WriteCoilModbusClient {
        coils::WriteCoilModbusClient::new(start_address)
    }

    pub fn read_coil_from(self, start_address: Address) -> coils::ReadCoilModbusClient {
        coils::ReadCoilModbusClient::new(start_address)
    }
}

fn read_response<E>(id: u8, quantity: u16, reader: &mut impl Read<u8, Error = E>) -> Result<usize, ModbusError<(), E>> {
    let read_value = nb::block!(reader.read());
    if let Err(err) = read_value {
        return Err(ModbusError::ReadErr(err));
    }

    let read_value = read_value.unwrap();
    if id + 0x80 == read_value {
        return match nb::block!(reader.read()) {
            Ok(err_code) => Err(byte_to_error(err_code)),
            Err(e) => Err(ModbusError::ReadErr(e))
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

    let buffer = BUFFER.borrow_mut();
    buffer[0] = read_value;
    let mut written_bytes = 1;
    for _ in 0..byte_count {

    }



    Ok(written_bytes)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let coil_read = ModbusClient::new().read_coil_from(0x15.into()).with_quantity(0x78);
    }
}
