#![feature(libc, std_misc)]

extern crate libc;
extern crate usb;
extern crate byteorder;

use std::io::{self};

use byteorder::{BigEndian, ReadBytesExt};
use usb::libusb::libusb_transfer_status;
use usb::libusb::libusb_transfer_type::LIBUSB_TRANSFER_TYPE_INTERRUPT;

const REQ_INT_LEN: usize = 8;
const ENDPOINT: u8 = 0x82;
const TIMEOUT: u32 = 5000;

static COMMAND_TEMP: &'static [u8] = b"\x01\x80\x33\x01\x00\x00\x00\x00";
static COMMAND_INI1: &'static [u8] = b"\x01\x82\x77\x01\x00\x00\x00\x00";
static COMMAND_INI2: &'static [u8] = b"\x01\x86\xff\x01\x00\x00\x00\x00";

#[derive(Debug)]
pub enum TemperReadErr {
    UsbTransfer(libusb_transfer_status)
}

impl From<libusb_transfer_status> for TemperReadErr {
    fn from(e: libusb_transfer_status) -> TemperReadErr {
        TemperReadErr::UsbTransfer(e)
    }
}

pub struct Temper<'a> {
    scale: f64,
    offset: f64,
    dev: usb::DeviceHandle<'a>,
}

impl<'a> Temper<'a> {
    pub fn new(dev: usb::DeviceHandle<'a>) -> Temper<'a> {
        Temper {
            scale: 1.0,
            offset: 0.0,
            dev: dev,
        }
    }

    pub fn initialize_maybe(&mut self) -> Result<(), TemperReadErr> {
        let interupt = LIBUSB_TRANSFER_TYPE_INTERRUPT;

        try!(self.dev.ctrl_write(0x21, 0x09, 0x0200, 0x01, COMMAND_TEMP, TIMEOUT));
        try!(self.dev.read(ENDPOINT, interupt, REQ_INT_LEN, TIMEOUT));

        try!(self.dev.ctrl_write(0x21, 0x09, 0x0200, 0x01, COMMAND_INI1, TIMEOUT));
        try!(self.dev.read(ENDPOINT, interupt, REQ_INT_LEN, TIMEOUT));

        try!(self.dev.ctrl_write(0x21, 0x09, 0x0200, 0x01, COMMAND_INI2, TIMEOUT));
        try!(self.dev.read(ENDPOINT, interupt, REQ_INT_LEN, TIMEOUT));
        try!(self.dev.read(ENDPOINT, interupt, REQ_INT_LEN, TIMEOUT));

        Ok(())
    }

    pub fn get_raw_temperature(&mut self) -> Result<i16, TemperReadErr> {
        let interupt = LIBUSB_TRANSFER_TYPE_INTERRUPT;

        try!(self.dev.ctrl_write(0x21, 0x09, 0x0200, 0x01, COMMAND_TEMP, TIMEOUT));
        let buf = try!(self.dev.read(ENDPOINT, interupt, REQ_INT_LEN, TIMEOUT));
        let mut reader = io::Cursor::new(&buf[2..4]);
        Ok(reader.read_i16::<BigEndian>().unwrap())
    }

    pub fn get_temperature(&mut self) -> Result<f64, TemperReadErr> {
        let val = try!(self.get_raw_temperature());
        Ok(125.0 * val as f64 / 32000.0)
    }
}

#[test]
fn it_works() {
    let usbctx = usb::Context::new();
    let dev = usbctx.find_by_vid_pid(0x0C45, 0x7401).expect("Device not found");
    let mut temper = Temper::new(dev.open().ok().expect("Device open failed"));
    panic!("{:?}", temper.get_temperature());
}
