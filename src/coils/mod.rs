use crate::Address;
use embedded_hal::serial::{Write, Read};
use core::{convert::Infallible, cell::RefCell};
use alloc::vec::Vec;
use crate::{ ModbusError, byte_to_error, BUFFER};

pub struct ReadCoilModbusClient {
    start_address: Address,
    quantity: RefCell<u16>,
}

pub struct WriteCoilModbusClient {
    start_address: Address,
    values: RefCell<Vec<bool>>,
}

impl ReadCoilModbusClient {
    pub fn new(start_address: Address) -> ReadCoilModbusClient {
        ReadCoilModbusClient {
            start_address,
            quantity: RefCell::default(),
        }
    }

    pub fn with_quantity(&self, quantity: u16) -> &Self {
        self.quantity.replace(quantity);
        self
    }

    pub fn send<E, EU>(self, writer: impl Write<u8, Error = E>, reader: &mut impl Read<u8, Error = EU>)
    -> Result<(), ModbusError<E, EU>> {
        let quantity = self.quantity.borrow();
        let mut buffer = BUFFER.borrow_mut();
        buffer[0] = 0x01;
        buffer[1] = (self.start_address.0 >> 8) as u8;
        buffer[2] = self.start_address.0 as u8;
        buffer[3] = (*quantity >> 8) as u8;
        buffer[4] = *quantity as u8;

        buffer.iter().map(|v| {
            nb::block!(writer.write(*v))
        });

        if let Ok(value) = nb::block!(reader.read()) {
            if buffer[0] == value + 0x80 {
                if let Ok(error) = nb::block!(reader.read()){
                    return Err(byte_to_error(error));
                }
            } else {
                buffer[0] = value;
            }
        }

        let byte_count = (quantity % 3) + 1;
        let mut index = 1;
        loop {
            if let Ok(value) = nb::block!(reader.read()) {
                buffer[index] = value;
                index += 1;
            }
            if byte_count == index {
                break;
            }
        }

        Ok(())
    }
}

impl WriteCoilModbusClient {
    pub fn new(start_address: Address) -> WriteCoilModbusClient {
        WriteCoilModbusClient {
            start_address,
            values: RefCell::new(Vec::new()),
        }
    }

    pub fn with_coil(&self, values: &[bool]) -> &Self {
        let mut vector = self.values.borrow_mut();
        vector.copy_from_slice(values);
        self
    }

    pub fn send<E, EU>(self, _writer:&mut impl Write<u8, Error = E>, _reader: &mut impl Read<u8, Error = EU>)
    -> Result<(), ModbusError<E, EU>> {
        let mut buffer = BUFFER.borrow_mut();
        let values = self.values.borrow();
        if values.len() == 1 {
            self.create_package_single(0x05, self.start_address, *values.first().unwrap(), &mut buffer);
        } else if values.len() > 1 {
            self.create_package_multiple(0x0F, self.start_address, values.as_slice(), &mut buffer);
        } else {

        }

        Ok(())
    }

    fn create_package_single(self, id: u8, address: Address, value: bool, buffer: &mut [u8]) {
        buffer[0] = (id);
        buffer[1] = (address.0 >> 8) as u8;
        buffer[2] = address.0 as u8;
        let value: u8 = if value { 0xFF } else { 0x00 };
        buffer[3] = value;
        buffer[4] = 0x00u8;
    }

    fn create_package_multiple(self, id: u8, address: Address, value: &[bool], buffer: &mut [u8]) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(id);
        buffer.push((address.0 >> 8) as u8);
        buffer.push(address.0 as u8);
        let expected_quantity = (value.len() / 8) + 1;
        let values = self.build_buffer(value);
        buffer.extend(values);
        if values.len() < expected_quantity {
            buffer.push(0x00);
        }
        buffer
    }

    fn build_buffer(self, values: &[bool]) -> Vec<u8> {
        let mut constructed_byte = 0x00u8;
        let mut constructed_values = Vec::new();
        let mut counter = 0;
        for (index, value) in values.iter().enumerate() {
            if counter == 8 {
                constructed_values.push(constructed_byte);
                constructed_byte = 0x00;
            }
            let value = *value as u8;
            constructed_byte |= value << (index % 8);
            counter += 1;
        }

        constructed_values.push(constructed_byte);
        constructed_values
    }
}