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
        buffer.push((value >> 8) as u8);
        buffer.push(value as u8);
        buffer
    }

    fn create_package_multiple(self, id: u8, address: Address, value: &[bool]) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(id);
        buffer.push((address.0 >> 8) as u8);
        buffer.push(address.0 as u8);
        value.iter().for_each(|v| {
            buffer.push((v >> 8) as u8);
            buffer.push(*v as u8);
        });
        buffer
    }
}