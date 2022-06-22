//! USB example that enumerates the device on on the host-side, but
//! does not expose any services.
#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use atsame54_xpro as bsp;
use bsp::hal;


use bsp::entry;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::watchdog::{Watchdog, WatchdogTimeout};
use hal::usb::{usb_device::bus::UsbBusAllocator, UsbBus};
use usb_device::prelude::*;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("!!! init");

    let mut peripherals = Peripherals::take().unwrap();

    Watchdog::new(peripherals.WDT).disable();

    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = Delay::new(core.SYST, &mut clocks);
    // delay.delay_ms(400u16);

    let mut pins = bsp::Pins::new(peripherals.PORT);
    let usb = bsp::pins::USB {
        dm: pins.usb_dm.into(),
        dp: pins.usb_dp.into(),
    };
    let usb = usb.init(peripherals.USB, &mut clocks, &mut peripherals.MCLK);
    let mut usb = UsbDeviceBuilder::new(&usb, UsbVidPid(0x2222, 0x3333))
        .manufacturer("Fake company")
        .product("Fake product")
        .serial_number("TEST")
        .device_class(0xff)
        .build();
    rprintln!("!!! UsbDeviceBuilder::build finalized");

    loop {
        let _ = usb.poll(&mut []);
    }
}
