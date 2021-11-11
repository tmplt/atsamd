//! Uses RTIC with the RTC as time source to blink an LED.
//!
//! The idle task is sleeping the CPU, so in practice this gives similar power
//! figure as the "sleeping_timer_rtc" example.
#![no_std]
#![no_main]

use feather_m0 as bsp;

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;
use rtic;

#[rtic::app(device = bsp::pac, peripherals = true, dispatchers = [EVSYS])]
mod app {

    use super::*;
    use bsp::hal;
    use fugit::ExtU32;
    use hal::clock::{ClockGenId, ClockSource, GenericClockController};
    use hal::pac::Peripherals;
    use hal::prelude::*;
    use hal::rtc::{Count32Mode, Rtc};

    use hal::eic::v1::{pin::*, *};
    use hal::gpio::v2::*;

    #[local]
    struct Local {
        ei_a1: ExtInt8<Pin<PB08, Interrupt<Floating>>>,
    }

    #[shared]
    struct Shared {
        // The LED could be a local resource, since it is only used in one task
        // But we want to showcase shared resources and locking
        red_led: bsp::RedLed,
    }

    #[monotonic(binds = RTC, default = true)]
    type RtcMonotonic = Rtc<Count32Mode>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut peripherals: Peripherals = cx.device;
        let pins = bsp::Pins::new(peripherals.PORT);
        let mut core: rtic::export::Peripherals = cx.core;
        let mut clocks = GenericClockController::with_external_32kosc(
            peripherals.GCLK,
            &mut peripherals.PM,
            &mut peripherals.SYSCTRL,
            &mut peripherals.NVMCTRL,
        );
        let _gclk = clocks.gclk0();
        let rtc_clock_src = clocks
            .configure_gclk_divider_and_source(ClockGenId::GCLK2, 1, ClockSource::XOSC32K, false)
            .unwrap();
        clocks.configure_standby(ClockGenId::GCLK2, true);
        let rtc_clock = clocks.rtc(&rtc_clock_src).unwrap();
        let rtc = Rtc::count32_mode(peripherals.RTC, rtc_clock.freq(), &mut peripherals.PM);
        let red_led: bsp::RedLed = pins.d13.into();

        let eic_clock_src = clocks
            .configure_gclk_divider_and_source(ClockGenId::GCLK3, 1, ClockSource::OSCULP32K, false)
            .unwrap();
        let eic_clock = clocks.eic(&eic_clock_src).unwrap();

        let mut eic = init_with_gclk(&mut peripherals.PM, eic_clock, peripherals.EIC);

        // Pin a1, matching with EXTINT8
        let mut ei_a1 = ExtInt8::new(pins.a1.into_floating_interrupt());
        ei_a1.sense(&mut eic, Sense::HIGH);
        ei_a1.enable_interrupt(&mut eic);

        let _eic = eic.finalize();

        // We can use the RTC in standby for maximum power savings
        core.SCB.set_sleepdeep();

        // Start the blink task
        blink::spawn().unwrap();

        (Shared { red_led }, Local { ei_a1 }, init::Monotonics(rtc))
    }

    #[task(shared = [red_led])]
    fn blink(mut cx: blink::Context) {
        // If the LED were a local resource, the lock would not be necessary
        cx.shared.red_led.lock(|led| led.toggle().unwrap());
        blink::spawn_after(1_u32.secs()).ok();
    }

    #[task(binds = EIC, local = [ei_a1])]
    fn eic_01(cx: eic_01::Context) {
        let ei_a1 = cx.local.ei_a1;
        ei_a1.clear_interrupt();
        cortex_m::asm::bkpt();
    }
}
