use crate::{Address, Error};
use embedded_hal::serial::{Write, Read};
use core::{cell::RefCell};
use alloc::vec::Vec;

pub struct ReadRegisterModbusClient {
    start_address: Address,
    quantity: RefCell<u16>,
}

pub struct WriteRegisterModbusClient {
    start_address: Address,
    values: RefCell<Vec<u16>>,
}

impl ReadRegisterModbusClient {
    pub fn new(start_address: Address) -> ReadRegisterModbusClient {
        ReadRegisterModbusClient {
            start_address,
            quantity: RefCell::default(),
        }
    }

    pub fn with_quantity(&self, quantity: u16) -> &Self {
        self.quantity.replace(quantity);
        self
    }

    pub fn send<WE, RE>(self, writer: &mut impl Write<u8, Error =WE>, reader: &mut impl Read<u8, Error =RE>)
                        -> Result<(), Error<WE, RE>> {
        let quantity = *self.quantity.borrow();
        let mut buffer = [0u8; 5];
        buffer[0] = 0x04;
        buffer[1] = (self.start_address.0 >> 8) as u8;
        buffer[2] = self.start_address.0 as u8;
        buffer[3] = (quantity >> 8) as u8;
        buffer[4] = quantity as u8;

        buffer.iter().map(|v| {
           nb::block!(writer.write(*v))
        });

        crate::read_response(0x04, quantity, reader)
    }
}

impl WriteRegisterModbusClient {
    pub fn new(start_address: Address) -> WriteRegisterModbusClient {
        WriteRegisterModbusClient {
            start_address,
            values: RefCell::new(Vec::new())
        }
    }

    pub fn with_values(&self, values: &[u16]) -> &Self {
        let mut vector = self.values.borrow_mut();
        vector.copy_from_slice(values);
        self
    }

    pub fn send<WE, RE>(self, writer: &mut impl Write<u8, Error =WE>, reader: &mut impl Read<u8, Error =RE>)
                        -> Result<(), Error<WE, RE>> {
        let values = self.values.borrow();
        let mut buffer: Option<Vec<u8>> = None;
        let id = if values.len() > 1 { 0x10 } else { 0x06 };
        if values.len() > 1 {
            buffer = Some(self.create_package_multiple(id, self.start_address, values.as_slice()));
        } else if values.len() == 1 {
            buffer = Some(self.create_package_single(id, self.start_address, *values.first().unwrap()));
        } else {
            buffer = Some(Vec::from([0x00]));
        }

        buffer.unwrap().iter().for_each(|v| {
            nb::block!(writer.write(*v));
        });

        crate::read_response(id, values.len() as u16, reader)
    }

    fn create_package_single(self, id: u8, address: Address, value: u16) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(id);
        buffer.push((address.0 >> 8) as u8);
        buffer.push(address.0 as u8);
        buffer.push((value >> 8) as u8);
        buffer.push(value as u8);
        buffer
    }

    fn create_package_multiple(self, id: u8, address: Address, value: &[u16]) -> Vec<u8> {
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