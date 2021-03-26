#![no_std]
#![feature(alloc)]

extern crate alloc;

use core::convert::Into;

pub mod coils;
pub mod registers;

pub struct ModbusClient;

pub struct Address(pub u16);

impl Into<Address> for u16 {
    fn into(self) -> Address {
        Address(self)
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let coil_read = ModbusClient::new().read_coil_from(0x15.into()).with_quantity(0x78);
    }
}
