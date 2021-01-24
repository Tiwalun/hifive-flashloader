#![no_std]
#![no_main]
// Define necessary functions for flash loader
//
// These are taken from the [ARM CMSIS-Pack documentation]
//
// [ARM CMSIS-Pack documentation]: https://arm-software.github.io/CMSIS_5/Pack/html/algorithmFunc.html

//#[cfg(debug_assertions)]
use panic_halt as _;

use core::slice;
use e310x::{qspi0, QSPI0};

fn transfer_byte(spi: &qspi0::RegisterBlock, data: u8) -> u8 {
    // Wait until TX fifo is empty
    while spi.txdata.read().full().bit_is_set() {}

    unsafe {
        spi.txdata.write(|w| w.data().bits(data));
    }

    loop {
        let rxdata = spi.rxdata.read().bits();

        // check if bit 31 is set, indicating
        // that the FIFO was empty
        if rxdata & (1 << 31) == 0 {
            return (rxdata & 0xff) as u8;
        }
    }
}

fn write_enable(spi: &qspi0::RegisterBlock) {
    transfer_byte(spi, 0x06);
}

fn read_status_register(spi: &qspi0::RegisterBlock) -> u8 {
    spi.csmode.write(|w| w.mode().hold());

    transfer_byte(spi, 0x05);

    // Read back response
    let val = transfer_byte(spi, 0);

    spi.csmode.write(|w| w.mode().auto());

    val
}

fn wait_for_wip_clear(spi: &qspi0::RegisterBlock) {
    loop {
        let status = read_status_register(spi);

        if status & 1 == 0 {
            break;
        }

        // TODO: delay?
    }
}

/// Erase the sector at the given address in flash
///
/// Returns 0 on success, 1 on failure.
#[no_mangle]
#[inline(never)]
pub extern "C" fn EraseSector(adr: u32) -> i32 {
    // Erase command for a single sector is
    // the 0xD7 / 0x20 command, followed by a 3-byte address

    // We assume that the SPI was setup correctly in the Init function
    let spi = unsafe { &(*QSPI0::ptr()) };

    write_enable(spi);

    // Ensure CS stays down for the whole transfer
    spi.csmode.write(|w| w.mode().hold());

    transfer_byte(spi, 0xd7);

    let address_bytes = adr.to_be_bytes();

    transfer_byte(spi, address_bytes[1]);
    transfer_byte(spi, address_bytes[2]);
    transfer_byte(spi, address_bytes[3]);

    spi.csmode.write(|w| w.mode().auto());

    // To verify that the erase is finished, we have to poll
    // the RDSR register

    wait_for_wip_clear(spi);

    0
}

/// Setup the device for the
#[no_mangle]
#[inline(never)]
pub extern "C" fn Init(_adr: u32, _clk: u32, _fnc: u32) -> i32 {
    let spi = unsafe { &(*QSPI0::ptr()) };

    // disable memory-mapped flash
    spi.fctrl.write(|w| w.enable().clear_bit());

    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn ProgramPage(adr: u32, sz: u32, buf: *const u8) -> i32 {
    let spi = unsafe { &(*QSPI0::ptr()) };

    let data = unsafe { slice::from_raw_parts(buf, sz as usize) };

    write_enable(spi);

    spi.csmode.write(|w| w.mode().hold());

    transfer_byte(spi, 0x02);

    let address_bytes = adr.to_be_bytes();

    transfer_byte(spi, address_bytes[1]);
    transfer_byte(spi, address_bytes[2]);
    transfer_byte(spi, address_bytes[3]);

    for byte in data {
        transfer_byte(spi, *byte);
    }

    spi.csmode.write(|w| w.mode().auto());

    wait_for_wip_clear(spi);

    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn UnInit(_fnc: u32) -> i32 {
    // Nothing to de-init
    0
}

const fn sectors() -> [FlashSector; 512] {
    let mut sectors = [FlashSector::default(); 512];

    sectors[0] = FlashSector {
        size: 0x1000,
        address: 0x0,
    };
    sectors[1] = SECTOR_END;

    sectors
}

#[allow(non_upper_case_globals)]
#[no_mangle]
#[used]
#[link_section = "DeviceData"]
pub static FlashDevice: FlashDeviceDescription = FlashDeviceDescription {
    vers: 0x0101,
    dev_name: [0u8; 128],
    dev_type: 5,
    dev_addr: 0x2000_0000,
    device_size: 0x200_0000,
    page_size: 256,
    _reserved: 0,
    empty: 0xff,
    program_time_out: 1000,
    erase_time_out: 2000,
    flash_sectors: sectors(),
};

#[repr(C)]
pub struct FlashDeviceDescription {
    vers: u16,
    dev_name: [u8; 128],
    dev_type: u16,
    dev_addr: u32,
    device_size: u32,
    page_size: u32,
    _reserved: u32,
    empty: u8,
    program_time_out: u32,
    erase_time_out: u32,

    flash_sectors: [FlashSector; 512],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FlashSector {
    size: u32,
    address: u32,
}

impl FlashSector {
    const fn default() -> Self {
        FlashSector {
            size: 0,
            address: 0,
        }
    }
}

const SECTOR_END: FlashSector = FlashSector {
    size: 0xffff_ffff,
    address: 0xffff_ffff,
};
