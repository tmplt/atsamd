//! TODO

use crate::pac::{GCLK, MCLK, NVMCTRL, OSC32KCTRL, OSCCTRL};
use crate::pac::osc32kctrl::rtcctrl::RTCSEL_A;

pub mod sources;
pub use sources::*;

pub mod gclk;
pub use gclk::*;

pub mod pclk;
pub use pclk::*;

pub mod ahb;
pub use ahb::*;

pub mod apb;
pub use apb::*;

/// TODO
pub struct PacClocks {
    pub oscctrl: OSCCTRL,
    pub osc32kctrl: OSC32KCTRL,
    pub gclk: GCLK,
    pub mclk: MCLK,
}

/// TODO
pub struct Clocks {
    pac: Option<PacClocks>,
    pub sources: sources::Sources,
    pub gclks: gclk::Gclks,
    pub pclks: pclk::Tokens,
    pub ahbs: ahb::AhbClks,
    pub apbs: apb::ApbClks,
}

impl Clocks {
    /// TODO
    pub fn new(
        oscctrl: OSCCTRL,
        osc32kctrl: OSC32KCTRL,
        gclk: GCLK,
        mclk: MCLK,
        nvmctrl: &mut NVMCTRL,
    ) -> Clocks {
        // TODO
        unsafe {
            Clocks {
                pac: Some(PacClocks {
                    oscctrl,
                    osc32kctrl,
                    gclk,
                    mclk,
                }),
                sources: sources::Sources::new(),
                gclks: gclk::Gclks::new(nvmctrl),
                pclks: pclk::Tokens::new(),
                ahbs: ahb::AhbClks::new(),
                apbs: apb::ApbClks::new(),
            }
        }
    }

    /// TODO
    pub unsafe fn pac(&mut self) -> Option<PacClocks> {
        self.pac.take()
    }
}

/// TODO
pub trait RtcClock {
    fn enable_1k(&mut self) -> RTCSEL_A;
    fn enable_32k(&mut self) -> RTCSEL_A;
}

/// TODO
pub fn set_rtc_clock<C: RtcClock>(clock: &mut C, enable_32k: bool) {
    use crate::pac::osc32kctrl::RegisterBlock;
    let rtc_sel = if enable_32k {
        clock.enable_32k()
    } else {
        clock.enable_1k()
    };
    unsafe {
        let osc32kctrl = OSC32KCTRL::ptr() as *mut RegisterBlock;
        (*osc32kctrl).rtcctrl.write(|w| w.rtcsel().variant(rtc_sel));
    }
}
