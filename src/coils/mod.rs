use crate::Address;
use embedded_hal::serial::{Write, Read};
use core::{convert::Infallible, cell::RefCell};
//use heapless::{Vec, consts::U8};
use alloc::vec::Vec;

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

    pub fn send<E, EU>(self, writer: impl Write<u8, Error = E>, _reader: impl Read<u8, Error = EU>)
    -> Result<(), Infallible> {
        let quantity = self.quantity.borrow();
        let mut buffer = [0u8; 5];
        buffer[0] = 0x04;
        buffer[1] = (self.start_address.0 >> 8) as u8;
        buffer[2] = self.start_address.0 as u8;
        buffer[3] = (*quantity >> 8) as u8;
        buffer[4] = *quantity as u8;

        buffer.iter().map(|v| {
            nb::block!(writer.write(*v))
        });

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
        /*
        if values.len() > vector.len() {
            if let Err(_) = vector.resize_default(values.len()) {
                // FUCK!
            }
        }
        if let Err(_) = vector.extend_from_slice(values) {
            // FUCK!
        }
        */
        self
    }

    pub fn send<E, EU>(self, _writer: impl Write<u8, Error = E>, _reader: impl Read<u8, Error = EU>)
    -> Result<(), Infallible> {
        

        Ok(())
    }

    fn create_package_single(self, id: u8, address: Address, value: bool) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(id);
        buffer.push((address.0 >> 8) as u8);
        buffer.push(address.0 as u8);
        let value: u8 = if value { 0xFF } else { 0x00 };
        buffer.push(value);
        buffer.push(0x00u8);
        buffer
    }

    fn create_package_multiple(self, id: u8, address: Address, value: &[bool]) -> Vec<u8> {
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