use crate::{Address, Error, ServerAddress};
use embedded_hal::serial::{Write, Read};
use core::{cell::RefCell};
use crate::ring_buffer::RingBuffer;

pub struct ReadRegisterModbusClient<'a> {
    server_address: ServerAddress,
    start_address: Address,
    quantity: RefCell<u16>,
    buffer: RingBuffer<'a, u8>,
}

pub struct WriteRegisterModbusClient<'a> {
    server_address: ServerAddress,
    start_address: Address,
    buffer: RingBuffer<'a, u8>,
}

impl<'a> ReadRegisterModbusClient<'a> {
    pub fn new(server_address: ServerAddress, start_address: Address, buffer: RingBuffer<'a, u8>) -> ReadRegisterModbusClient {
        ReadRegisterModbusClient {
            server_address,
            start_address,
            quantity: RefCell::default(),
            buffer,
        }
    }

    pub fn with_quantity(&self, quantity: u16) -> &Self {
        self.quantity.replace(quantity);
        self
    }

    pub fn send<WE, RE>(&'a mut self, writer: &mut impl Write<u8, Error =WE>, reader: &mut impl Read<u8, Error =RE>)
                        -> Result<(usize, &'a [u8]), Error<WE, RE>> {
        let quantity = *self.quantity.borrow();
        self.buffer.clear();
        self.buffer.push_single(self.server_address.0);
        self.buffer.push_single(0x04);
        self.buffer.push_single((self.start_address.0 >> 8) as u8);
        self.buffer.push_single(self.start_address.0 as u8);
        self.buffer.push_single((quantity >> 8) as u8);
        self.buffer.push_single(quantity as u8);

        let crc = crate::add_crc(&self.buffer);
        self.buffer.push_single(crc as u8);
        self.buffer.push_single((crc >> 8) as u8);

        let buffer = self.buffer.get_written();
        buffer.iter().map(|v| {
           nb::block!(writer.write(*v))
        });

        crate::read_response(0x04, quantity, reader, &mut self.buffer)
    }
}

impl<'a> WriteRegisterModbusClient<'a> {
    pub fn new(server_address: ServerAddress, start_address: Address, buffer: RingBuffer<'a, u8>) -> WriteRegisterModbusClient {
        WriteRegisterModbusClient {
            server_address,
            start_address,
            buffer
        }
    }

    pub fn send<WE, RE>(&'a mut self, values: &[u16], writer: &mut impl Write<u8, Error =WE>, reader: &mut impl Read<u8, Error =RE>)
                        -> Result<(usize, &'a [u8]), Error<WE, RE>> {
        let id = if values.len() > 1 { 0x10 } else { 0x06 };
        self.buffer.clear();
        self.buffer.push_single(self.server_address.0);
        if values.len() > 1 {
            self.create_package_multiple(id, values);
        } else if values.len() == 1 {
            self.create_package_single(id, *values.first().unwrap());
        } else {
        }
        let crc = crate::add_crc(&self.buffer);
        self.buffer.push_single(crc as u8);
        self.buffer.push_single((crc >> 8) as u8);

        let buffer = self.buffer.get_written();
        buffer.iter().for_each(|v| {
            nb::block!(writer.write(*v));
        });

        crate::read_response(id, values.len() as u16, reader, &mut self.buffer)
    }

    fn create_package_single(&mut self, id: u8, value: u16) {
        self.buffer.push_single(id);
        self.buffer.push_single((self.start_address.0 >> 8) as u8);
        self.buffer.push_single(self.start_address.0 as u8);
        self.buffer.push_single((value >> 8) as u8);
        self.buffer.push_single(value as u8);
    }

    fn create_package_multiple(&mut self, id: u8, values: &[u16]) {
        self.buffer.push_single(id);
        self.buffer.push_single((self.start_address.0 >> 8) as u8);
        self.buffer.push_single(self.start_address.0 as u8);
        for value in values.iter() {
            self.buffer.push_single((value >> 8) as u8);
            self.buffer.push_single(*value as u8);
        }
    }
}