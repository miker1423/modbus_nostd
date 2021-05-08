use crate::{Address, read_response};
use embedded_hal::serial::{Write, Read};
use core::{cell::RefCell};
use crate::Error;
use crate::ring_buffer::RingBuffer;

pub struct ReadCoilModbusClient<'a> {
    start_address: Address,
    quantity: RefCell<u16>,
    buffer: RingBuffer<'a, u8>,
}

pub struct WriteCoilModbusClient<'a> {
    start_address: Address,
    buffer: RingBuffer<'a, u8>,
}

impl<'a> ReadCoilModbusClient<'a> {
    pub fn new(start_address: Address, buffer: RingBuffer<'a, u8>) -> ReadCoilModbusClient {
        ReadCoilModbusClient {
            start_address,
            quantity: RefCell::default(),
            buffer
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
        self.buffer.push_single(0x01);
        self.buffer.push_single((self.start_address.0 >> 8) as u8);
        self.buffer.push_single(self.start_address.0 as u8);
        self.buffer.push_single((quantity >> 8) as u8);
        self.buffer.push_single(quantity as u8);

        self.buffer.get_written().iter().map(|v| {
            nb::block!(writer.write(*v))
        });

        read_response(0x01, quantity, reader, &mut self.buffer)
    }
}

impl<'a> WriteCoilModbusClient<'a> {
    pub fn new(start_address: Address, buffer: RingBuffer<'a, u8>,)
        -> WriteCoilModbusClient {
        WriteCoilModbusClient { start_address, buffer }
    }

    pub fn send<WE, RE>(&'a mut self, values: &[bool], _writer:&mut impl Write<u8, Error =WE>, reader: &mut impl Read<u8, Error =RE>)
                        -> Result<(usize, &'a [u8]), Error<WE, RE>> {
        self.buffer.clear();
        let id = if values.len() == 1 { 0x05 } else { 0x0F };
        if values.len() == 1 {
            self.create_package_single(id, *values.first().unwrap());
        } else if values.len() > 1 {
            self.create_package_multiple(id, values);
        } else {

        }

        read_response(id, values.len() as u16, reader, &mut self.buffer)
    }

    fn create_package_single(&mut self, id: u8, value: bool) {
        self.buffer.push_single(id);
        self.buffer.push_single((self.start_address.0 >> 8) as u8);
        self.buffer.push_single(self.start_address.0 as u8);
        let value: u8 = if value { 0xFF } else { 0x00 };
        self.buffer.push_single(value);
        self.buffer.push_single(0x00u8);
    }

    fn create_package_multiple(&mut self, id: u8, value: &[bool]) -> usize {
        self.buffer.push_single(id);
        self.buffer.push_single((self.start_address.0 >> 8) as u8);
        self.buffer.push_single(self.start_address.0 as u8);
        let expected_quantity = (value.len() / 8) + 1;
        let written_bytes = self.build_buffer(value);
        if written_bytes < expected_quantity {
            self.buffer.push_single(0x00);
            return written_bytes + 1;
        }
        written_bytes
    }

    fn build_buffer(&mut self, values: &[bool]) -> usize {
        let mut constructed_byte = 0x00u8;
        let mut written_bytes = 0;
        for (index, value) in values.iter().enumerate() {
            if index != 0 && (index % 8) == 0 {
                self.buffer.push_single(constructed_byte);
                written_bytes += 1;
                constructed_byte = 0x00;
            }
            let value = *value as u8;
            constructed_byte |= value << (index % 8);
        }

        self.buffer.push_single(constructed_byte);
        written_bytes += 1;
        written_bytes
    }
}