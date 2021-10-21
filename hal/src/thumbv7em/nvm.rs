//! Non-volatile Memory Controller
//! TODO: this might be a good fit for generalizing over multiple chips later
//! on, but for now it will only be tested on the SAME54

pub mod smart_eeprom;

pub use crate::target_device::nvmctrl::ctrla::PRM_A;
use crate::target_device::nvmctrl::ctrlb::CMD_AW;
use crate::target_device::NVMCTRL;
use core::ops::Range;

/// Size of a page in bytes
pub const PAGESIZE: u32 = 512;

/// Size of the internal flash memory
pub const FLASHSIZE: u32 = 1024 * 1024;

/// Number of words for one bank
/// Because of the memory layout, this is also the offset from memory start
pub const BANKSIZE: u32 = FLASHSIZE / 2;

/// Size of one block
pub const BLOCKSIZE: u32 = 512 * 16;

/// Non-colatile memory controller
pub struct Nvm {
    /// PAC peripheral
    nvm: NVMCTRL,
}

/// Errors generated by the NVM peripheral
#[derive(Debug)]
pub enum PeripheralError {
    /// NVM error
    NvmError,
    /// Single ECC error
    EccSingleError,
    /// Dual ECC error
    EccDualError,
    /// Locked error
    LockError,
    /// Programming error
    ProgrammingError,
    /// Address error
    AddressError,
}

/// Driver errors
#[derive(Debug)]
pub enum Error {
    /// Target sector is protected
    Protected,
    /// Boot protection is already set
    AlreadyBootProtected,
    /// Errors generated by hardware
    Peripheral(PeripheralError),
    /// The DSU failed in some way
    Dsu(super::dsu::Error),
    /// An alignment requirement was not fulfilled
    Alignment,
}

/// Physical flash banks
#[derive(PartialEq, Debug)]
pub enum PhysicalBank {
    /// Flash bank A
    A,
    /// Flash bank B
    B,
}

#[derive(PartialEq, Debug)]
/// Flash banks identified by which one we boot from.
pub enum Bank {
    /// Bank that is mapped to 0x0000_0000
    Active,
    /// Bank that is not mapped to 0x0000_0000
    Inactive,
}

impl Bank {
    /// Provides the address of the bank
    pub fn address(&self) -> u32 {
        match self {
            Bank::Active => 0,
            Bank::Inactive => BANKSIZE,
        }
    }
}

/// NVM result type
pub type Result<T> = core::result::Result<T, Error>;

impl Nvm {
    /// Create a new NVM controller or handle failure from DSU
    pub fn new(nvm: NVMCTRL) -> Self {
        Self { nvm }
    }

    /// Swap the flash banks. The processor will be reset, after which the
    /// inactive bank will become the active bank.
    ///
    /// # Safety
    /// Ensure there is a working, memory safe program in place in the inactive
    /// bank before calling.
    pub unsafe fn bank_swap(&mut self) -> ! {
        self.command_sync(CMD_AW::BKSWRST);
        // The reset will happen atomically with the rest of the command, so getting
        // here is an error.
        unreachable!();
    }

    /// Configure the wait state
    /// Safety: the implementor must guarantee (empirically) that the amount of
    /// wait states satisfy their requirements
    pub unsafe fn wait_state(&mut self, ws: u8) {
        assert!(ws < 16, "invalid wait state");
        self.nvm.ctrla.modify(|_, w| w.rws().bits(ws));
    }

    /// Set the power reduction mode
    pub fn power_reduction_mode(&mut self, prm: PRM_A) {
        self.nvm.ctrla.modify(|_, w| w.prm().variant(prm));
    }

    /// Check if the flash is boot protected
    pub fn is_boot_protected(&self) -> bool {
        !self.nvm.status.read().bpdis().bit()
    }

    /// Get first bank
    pub fn first_bank(&self) -> PhysicalBank {
        if self.nvm.status.read().afirst().bit() {
            PhysicalBank::A
        } else {
            PhysicalBank::B
        }
    }

    /// Set address for reading/writing
    fn set_address(&mut self, address: u32) {
        // Safety: the entire register is used for address, invalid addresses are
        // signalled by the hardware
        unsafe {
            self.nvm
                .addr
                .write(|w| w.addr().bits(address & 0x00ff_ffff));
        }
    }

    /// Determine if the controller is busy writing or erasing
    pub fn is_ready(&self) -> bool {
        self.nvm.status.read().ready().bit()
    }

    /// Run a flash command
    fn command(&mut self, command: CMD_AW) {
        self.nvm
            .ctrlb
            .write(|w| w.cmdex().key().cmd().variant(command));
    }

    /// Check if flash command done
    fn command_done(&self) -> bool {
        self.nvm.intflag.read().done().bit()
    }

    /// Run a flash, wait for done
    fn command_sync(&mut self, command: CMD_AW) {
        self.command(command);

        while !self.command_done() {
            cortex_m::asm::nop();
        }
        self.nvm.intflag.write(|w| w.done().set_bit());
    }

    /// Check if there was a programming fault
    fn programming_error(&self) -> Result<()> {
        if self.nvm.intflag.read().proge().bit() {
            // Clear the error flag
            self.nvm.intflag.write(|w| w.proge().set_bit());

            Err(Error::Peripheral(PeripheralError::ProgrammingError))
        } else {
            Ok(())
        }
    }

    /// Clear the programming error flag
    fn programming_error_clear(&self) -> Result<()> {
        // Clear the error flag
        self.nvm.intflag.write(|w| w.proge().set_bit());
        Ok(())
    }

    /// Check if there was a section lock fault
    fn lock_error(&self) -> Result<()> {
        if self.nvm.intflag.read().locke().bit() {
            Err(Error::Peripheral(PeripheralError::LockError))
        } else {
            Ok(())
        }
    }

    /// Clear the lock error flag
    fn lock_error_clear(&self) -> Result<()> {
        // Clear the error flag
        self.nvm.intflag.write(|w| w.locke().set_bit());
        Ok(())
    }

    /// Read the user page
    pub fn user_page(&self) -> [u32; PAGESIZE as usize / 4] {
        let mut output = [0u32; PAGESIZE as usize / 4];
        let base_addr: *const u32 = 0x80_4000 as *const u32;

        for (i, o) in output.iter_mut().enumerate() {
            *o = unsafe { core::ptr::read_volatile(base_addr.offset(i as isize)) };
        }

        output
    }

    /// Turn boot protection on/off
    pub fn boot_protection(&mut self, protect: bool) -> Result<()> {
        // Check if requested state differs from current state
        if self.is_boot_protected() != protect {
            // Wait until ready
            while !self.is_ready() {
                cortex_m::asm::nop();
            }

            // Requires both command and key so the command is allowed to execute
            if !protect {
                // Issue Set boot protection disable (disable bootprotection)
                self.command_sync(CMD_AW::SBPDIS);
            } else {
                // Issue Clear boot protection disable (enable bootprotection)
                self.command_sync(CMD_AW::CBPDIS);
            }

            self.programming_error()
        } else {
            Err(Error::AlreadyBootProtected)
        }
    }

    /// `offset` is the word-aligned offset (in bytes) from the start of the
    /// bank where the write should start.
    pub fn flash_write(&mut self, bank: &Bank, offset: u32, data: &[u32]) -> Result<()> {
        let address = bank.address() + offset;
        self.write(address, data)
    }

    /// Write to flash memory
    /// If `address` is not word-aligned, an error is returned.
    pub fn write(&mut self, address: u32, data: &[u32]) -> Result<()> {
        // Length of memory step
        let step_size: u32 = core::mem::size_of::<u32>() as u32;
        // Length of data to flash
        let length = data.len() as u32 * step_size;
        let write_addrs = address..(address + length);

        if address % step_size != 0 {
            return Err(Error::Alignment);
        }

        if self.contains_bootprotected(&write_addrs) {
            Err(Error::Protected)
        } else {
            while !self.is_ready() {
                cortex_m::asm::nop();
            }
            self.command_sync(CMD_AW::PBC);
            // Track whether we have unwritten data in the page buffer
            let mut dirty = false;
            // Zip two iterators, one counter and one with the addr word aligned
            for (counter, addr) in (0..).zip(write_addrs.step_by(step_size as usize)) {
                // Write to memory, 32 bits, 1 word.
                // The data is placed in the page buffer and ADDR is updated automatically.
                // Memory is not written until the write page command is issued later.
                unsafe { core::ptr::write_volatile(addr as *mut u32, data[counter] as u32) }
                dirty = true;

                // If we are about to cross a page boundary (and run out of page buffer), write
                // to flash
                if addr % PAGESIZE >= PAGESIZE - step_size {
                    // Wait until ready
                    while !self.is_ready() {
                        cortex_m::asm::nop();
                    }

                    dirty = false;
                    // Perform a write
                    self.command_sync(CMD_AW::WP);
                }
            }

            // Wait until the last write operation is finished
            while !self.is_ready() {
                cortex_m::asm::nop();
            }

            if dirty {
                // The dirty flag has fulfilled its role here, so we don't bother to maintain
                // its invariant anymore. Otherwise, the compiler would warn of
                // unused assignments. Write last page
                self.command_sync(CMD_AW::WP);
            }

            // Check if there was a programming fault
            if let Err(e) = self.programming_error() {
                self.lock_error()?;

                // Clear the error flags
                self.programming_error_clear()?;
                self.lock_error_clear()?;

                Err(e)
            } else {
                Ok(())
            }
        }
    }

    /// Erase a page in the auxilliary space (user page, calibration page,
    /// factory and signature page). Page erase is not supported for main
    /// memory.
    pub fn erase_aux_page(&mut self, address: u32, num_pages: u32) -> Result<()> {
        // TODO since there are few pages that are erased this way, we could enumerate
        // them instead of taking an address.
        self.erase(address, num_pages, EraseGranularity::Page)
    }

    /// Erase a block of main memory
    /// `address` is the address *within* the bank.
    pub fn erase_block(&mut self, bank: &Bank, address: u32, num_blocks: u32) -> Result<()> {
        let address = bank.address() + address;
        self.erase(address, num_blocks, EraseGranularity::Block)
    }

    /// length is in units depending on the erase granularity.
    pub fn erase(
        &mut self,
        address: u32,
        length: u32,
        granularity: EraseGranularity,
    ) -> Result<()> {
        // Align to block/page boundary
        // While the NVM will accept any address in the block, we need to compute the
        // aligned address to check for boot protection.
        let flash_address = address - address % granularity.size();
        let range_to_erase = flash_address..(flash_address + length * granularity.size());

        if self.contains_bootprotected(&range_to_erase) {
            Err(Error::Protected)
        } else {
            for address in range_to_erase.step_by(granularity.size() as usize) {
                // Set target address to current block/page offset
                self.set_address(address);

                // Wait until ready
                while !self.is_ready() {
                    cortex_m::asm::nop();
                }

                // Erase block/page, wait for completion
                self.command_sync(granularity.command());

                // Check if there was a programming fault
                if let Err(e) = self.programming_error() {
                    self.lock_error()?;

                    // Clear the error flags
                    self.programming_error_clear()?;
                    self.lock_error_clear()?;

                    return Err(e);
                }
            }

            Ok(())
        }
    }

    fn contains_bootprotected(&self, inp: &Range<u32>) -> bool {
        // Calculate size that is protected for bootloader
        //   * 15 = no bootprotection, default value
        //   * 0 = max bootprotection, 15 * 8Kibyte = 120KiB
        //   * (15 - bootprot) * 8KiB = protected size
        let bootprot = self.nvm.status.read().bootprot().bits();
        let bp_space = 8 * 1024 * (15 - bootprot) as u32;

        let boot = &(Bank::Active.address()..(Bank::Active.address() + bp_space));
        self.is_boot_protected() && range_overlap(inp, boot)
    }

    pub fn smart_eeprom(&mut self) -> smart_eeprom::Result {
        smart_eeprom::SmartEepromMode::retrieve(self)
    }
}

#[derive(Copy, Clone, Debug)]
/// Data erased per command
pub enum EraseGranularity {
    /// One block. This erase type is supported by main memory
    Block,
    /// One page. This erase type is supported for the AUX memory
    Page,
}

impl EraseGranularity {
    fn command(&self) -> CMD_AW {
        match self {
            EraseGranularity::Block => CMD_AW::EB,
            EraseGranularity::Page => CMD_AW::EP,
        }
    }

    fn size(&self) -> u32 {
        match self {
            EraseGranularity::Block => BLOCKSIZE,
            EraseGranularity::Page => PAGESIZE,
        }
    }
}

fn range_overlap(a: &Range<u32>, b: &Range<u32>) -> bool {
    // When start == end, the range includes no points
    a.start != a.end && b.start != b.end && a.start <= b.end && b.start <= a.end
}
