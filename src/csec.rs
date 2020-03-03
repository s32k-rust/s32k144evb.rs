//! # On-board Cryptographic Service Engine (CSEc)
//!
//! This module is an interface implementation for the board's hardware-accelerated cryptographic
//! functions. A range of functions are silicon-supported, but this module currently implements
//! * random number generation,
//! * plainkey loading into RAM slot,
//! * AES-CBC-128 encryption/decryption, and
//! * MAC generation and verification.
//!
//! Hardware used in this module is documented in the reference manual, § 35.6.13, p. 847.
//!
//! ## Usage
//!
//! - Random number generation
//!
//! This module can generate a `[u8; 16]` of random bits via
//! ```rust
//! mod csec;
//!
//! let csec = csec::CSEc::init(&p.FTFC, &p.CSE_PRAM);
//! csec.init_rng().unwrap();
//! let rnd_buf: [u8; 16] = csec.generate_rnd().unwrap();
//! assert!(u128::from_be_bytes(rnd_buf) != 0); // very likely
//! ```
//!
//! - AES-CBC-128 encryption/decryption
//!
//! This module can encrypt/decrypt a `[u8]` of a size which is an integer multiple of 16
//! after a key (`[u8; 16]`) has been loaded and an initialization vector
//! (also `[u8; 16]`) has been provided.
//!
//! ```rust
//! mod csec;
//!
//! // Example key
//! const PLAINKEY: [u8; 16] = [
//!     0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
//!     0x3c,
//! ];
//!
//! let plaintext: &[u8] = "Key:0123456789ab".as_bytes();
//! let initvct: &[u8] = "1234567887654321".as_bytes();
//! let mut buffer: [u8; 16] = [0; 16];
//!
//! let csec = csec::CSEc::init(&p.FTFC, &p.CSE_PRAM);
//! let rnd_buf = csec.generate_rnd().unwrap();
//! csec.load_plainkey(&PLAINKEY).unwrap();
//! buffer.copy_from_slice(plaintext);
//! csec.encrypt_cbc(&rnd_buf, &mut buffer).unwrap();
//! csec.decrypt_cbc(&rnd_buf, &mut buffer).unwrap();
//! assert!(plaintext == &buffer[..]);
//! ```
//!
//! The provided key is loaded onto the board's RAM key slot. Multiple key slots are available, but
//! support for those are not yet implemented.
//!
//! - MAC generation/verification
//!
//! This module can generate a `[u8; 16]` containing a calculated One-key MAC (message authentication code)
//! for an `[u8]` input of a length up to 2KB, and a loaded key.
//! ```rust
//! mod csec;
//!
//! // Example key
//! const PLAINKEY: [u8; 16] = [
//!     0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
//!     0x3c,
//! ];
//!
//! let csec = csec::CSEc::init(&p.FTFC, &p.CSE_PRAM);
//! csec.load_plainkey(&PLAINKEY).unwrap();
//!
//! let plaintext: &[u8] = "Key:0123456789ab-someotherbytes".as_bytes();
//! let cmac = csec.generate_mac(&plaintext).unwrap();
//! assert!(csec.verify_mac(&plaintext, &cmac).unwrap());
//! ```
//!
//! ## Security
//! During encryption the initialization vector must be random and unpredictable (for each
//! message), and may be made public after encryption. It is then recommended to use the output of
//! `generate_rnd()` as the initialization vector for an encryption.
//!
//! The initialization vector is required for decryption, so it is recommended to prefix it to the
//! sent message. Only the key is a secret.
//!
//! ## Hardware API
//! The API for the CSEc is 7 "pages" of 128-bit each in FTFC PRAM. Prefixed to these pages is a command header.
//! To run a CSEc operation, data to be processed should first be written to these pages after
//! which the wanted operation, along with eventual operation arguments, are written to the command
//! header. See the images below.
#![allow(dead_code)]

use s32k144;

/// CSEc commands which follow the same values as the SHE command defenition.
#[derive(Debug, Clone, Copy)]
enum Command {
    EncEcb = 0x01,

    /// Implemented!
    EncCbc,

    DecEcb,

    /// Implemented!
    DecCbc,

    /// Implemented!
    GenerateMac,

    /// Implemented!
    VerifyMac,

    LoadKey,

    /// Implemented!
    LoadPlainKey,

    ExportRamKey,

    /// Implemented!
    InitRng,

    ExtendSeed,

    /// Implemented!
    Rng,

    Reserved1,
    BootFailure,
    BootOk,
    GetId,
    BootDefine,
    DbgChal,
    DbgAuth,
    Reserved2,
    Reserved3,
    MPCompress,
}

/// Specifies how the data is transferred to/from the CSE.
/// There are two use cases. One is to copy all data and the command function call method and the
/// other is a pointer and function call method.
enum Format {
    Copy = 0x0,
    Pointer,
}

/// Specifies if the information is the first of a following function call.
#[derive(PartialEq)]
enum Sequence {
    First = 0x0,
    Subsequent,
}

/// Specify the KeyID to be used to implement the requested cryptographic operation.
enum KeyID {
    SecretKey = 0x0,
    MasterEcu,
    BootMacKey,
    BootMac,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key10,
    RamKey = 0xf,
    Key11 = 0x14,
    Key12,
    Key13,
    Key14,
    Key15,
    Key16,
    Key17,
}

/// Represents the result of the execution of a command. Provides one bit for each error code as
/// per SHE specification.
#[derive(Debug)]
pub enum CommandResult {
    NoError = 0x1,
    SequenceError = 0x2,
    KeyNotAvailable = 0x4,
    KeyInvalid = 0x8,
    KeyEmpty = 0x10,
    NoSecureBoot = 0x20,
    KeyWriteProtected = 0x40,
    KeyUpdateError = 0x80,
    RngSeed = 0x100,
    NoDebugging = 0x200,
    MemoryFailure = 0x400,
    GeneralError = 0x800,
}

impl CommandResult {
    fn from_u16(value: u16) -> CommandResult {
        match value {
            0x1 => CommandResult::NoError,
            0x2 => CommandResult::SequenceError,
            0x4 => CommandResult::KeyNotAvailable,
            0x8 => CommandResult::KeyInvalid,
            0x10 => CommandResult::KeyEmpty,
            0x20 => CommandResult::NoSecureBoot,
            0x40 => CommandResult::KeyWriteProtected,
            0x80 => CommandResult::KeyUpdateError,
            0x100 => CommandResult::RngSeed,
            0x200 => CommandResult::NoDebugging,
            0x400 => CommandResult::MemoryFailure,
            0x800 => CommandResult::GeneralError,
            _ => panic!("Unknown CommandResult value: {}", value),
        }
    }
}

/// Safely transforms a `u32` to Big-Endian `[u8; 4]`.
fn u8_be_array_from_u32(x: u32) -> [u8; 4] {
    [
        ((x >> 24) & 0xff) as u8,
        ((x >> 16) & 0xff) as u8,
        ((x >> 8) & 0xff) as u8,
        ((x >> 0) & 0xff) as u8,
    ]
}

pub struct CSEc {
    ftfc: s32k144::FTFC,
    cse_pram: s32k144::CSE_PRAM,
}

const PAGE_1_OFFSET: usize = 16;
const PAGE_2_OFFSET: usize = 32;
const PAGE_LENGTH_OFFSET: usize = 14;
const PAGE_SIZE_IN_BYTES: usize = 16;
const ERROR_BITS_OFFSET: usize = 4;
const LOWER_HALF_MASK: u32 = 0xffff;
const LOWER_HALF_SHIFT: u32 = 0x0;
const UPPER_HALF_MASK: u32 = 0xffff0000;
const UPPER_HALF_SHIFT: u32 = 0x10;
const BYTES_TO_PAGES_SHIFT: u32 = 4;
const MAX_PAGES: usize = 7;
const MAC_MESSAGE_LENGTH_OFFSET: usize = 0xc;
const MAC_VERIFICATION_BITS_OFFSET: usize = PAGE_1_OFFSET + 0x4;
const MAC_LENGTH_OFFSET: usize = 0x8;

impl CSEc {
    pub fn init(ftfc: s32k144::FTFC, cse_pram: s32k144::CSE_PRAM) -> Self {
        CSEc {
            ftfc: ftfc,
            cse_pram: cse_pram,
        }
    }

    /// Initializes the seed and derive a key for the PRNG.
    /// This function must be called before `generate_rnd`.
    pub fn init_rng(&self) -> Result<(), CommandResult> {
        self.write_command_header(
            Command::InitRng,
            Format::Copy,
            Sequence::First,
            KeyID::SecretKey,
        )
    }

    /// Generates a vector of 128 random bits.
    /// This function must be called after `init_rng`.
    pub fn generate_rnd(&self) -> Result<[u8; 16], CommandResult> {
        self.write_command_header(
            Command::Rng,
            Format::Copy,
            Sequence::First,
            KeyID::SecretKey,
        )?;

        // Read the resulted random bytes
        let mut buf: [u8; 16] = [0; 16];
        self.read_command_bytes(PAGE_1_OFFSET, &mut buf);

        Ok(buf)
    }

    /// Updates the RAM key memory slot with a 128-bit plaintext.
    pub fn load_plainkey(&self, key: &[u8; PAGE_SIZE_IN_BYTES]) -> Result<(), CommandResult> {
        // Write the bytes of the key
        self.write_command_bytes(PAGE_1_OFFSET, key);

        self.write_command_header(
            Command::LoadPlainKey,
            Format::Copy,
            Sequence::First,
            KeyID::RamKey,
        )
    }

    /// Perform in-place AES-128 encryption in CBC mode of the input buffer.
    pub fn encrypt_cbc(
        &self,
        init_vec: &[u8; PAGE_SIZE_IN_BYTES],
        buffer: &mut [u8],
    ) -> Result<(), CommandResult> {
        self.handle_cbc(Command::EncCbc, init_vec, buffer)
    }

    /// Perform in-place AES-128 decryption in CBC mode of the input buffer.
    pub fn decrypt_cbc(
        &self,
        init_vec: &[u8; PAGE_SIZE_IN_BYTES],
        buffer: &mut [u8],
    ) -> Result<(), CommandResult> {
        self.handle_cbc(Command::DecCbc, init_vec, buffer)
    }

    /// Generate a 128-bit Message Authentication Code for `input`.
    pub fn generate_mac(&self, message: &[u8]) -> Result<[u8; 16], CommandResult> {
        if message.len() > u32::max_value() as usize {
            return Err(CommandResult::GeneralError);
        }

        // Write how long our message is (in bits)
        self.write_command_words(MAC_MESSAGE_LENGTH_OFFSET, &[(message.len() * 8) as u32]);

        fn process_blocks(
            cse: &CSEc,
            message: &[u8],
            sequence: Sequence,
        ) -> Result<(), CommandResult> {
            // How many bytes are we processing this round?
            let bytes = core::cmp::min(message.len(), MAX_PAGES * PAGE_SIZE_IN_BYTES);

            // Write out message bytes from `message` and process them.
            cse.write_command_bytes(PAGE_1_OFFSET, &message[..bytes]);
            cse.write_command_header(Command::GenerateMac, Format::Copy, sequence, KeyID::RamKey)?;

            // Process remaining bytes, if any
            if message.len() - bytes != 0 {
                process_blocks(cse, &message[bytes..], Sequence::Subsequent)
            } else {
                Ok(())
            }
        }

        process_blocks(self, message, Sequence::First)?;

        // Read out calculated MAC
        let mut cmac: [u8; 16] = [0; 16];
        self.read_command_bytes(PAGE_2_OFFSET, &mut cmac);

        Ok(cmac)
    }

    /// Verify a message against a 128-bit Message Authentication Code.
    pub fn verify_mac(&self, message: &[u8], cmac: &[u8; 16]) -> Result<bool, CommandResult> {
        // A length of 0 is interpreted by SHE to compare all bits of `mac`.
        if message.len() == 0 || message.len() > u32::max_value() as usize {
            return Err(CommandResult::GeneralError);
        }

        // Write how long our message is (in bits)
        self.write_command_words(MAC_MESSAGE_LENGTH_OFFSET, &[(message.len() * 8) as u32]);

        // Write the number of bits of the CMAC to be compared
        self.write_command_halfword(MAC_LENGTH_OFFSET, (cmac.len() * 8) as u16);

        // Write all n data blocks first, and write expected CMAC on page n + 1.
        fn process_blocks(
            cse: &CSEc,
            message: &[u8],
            cmac: &[u8],
            sequence: Sequence,
            mut mac_written: bool,
        ) -> Result<bool, CommandResult> {
            // How many bytes are we processing this round?
            let bytes = core::cmp::min(message.len(), MAX_PAGES * PAGE_SIZE_IN_BYTES);

            // Write our `message` bytes
            cse.write_command_bytes(PAGE_1_OFFSET, &message[..bytes]);

            // Which page is the next, rounded up?
            let next_page = (PAGE_1_OFFSET + bytes + PAGE_SIZE_IN_BYTES - 1) / PAGE_SIZE_IN_BYTES;
            if !mac_written && next_page < MAX_PAGES {
                // All data blocks has been written, append the expected CMAC.
                cse.write_command_bytes(next_page * PAGE_SIZE_IN_BYTES, &cmac);
                mac_written = true;
            }

            cse.write_command_header(Command::VerifyMac, Format::Copy, sequence, KeyID::RamKey)?;

            // Read verification status bits
            let success = cse.read_command_halfword(MAC_VERIFICATION_BITS_OFFSET) == 0;

            // Process remaining blocks until expected CMAC has been written.
            if !mac_written {
                process_blocks(
                    cse,
                    &message[bytes..],
                    cmac,
                    Sequence::Subsequent,
                    mac_written,
                )
            } else {
                Ok(success)
            }
        }

        process_blocks(self, message, cmac, Sequence::First, false)
    }

    fn handle_cbc(
        &self,
        command: Command,
        init_vec: &[u8; PAGE_SIZE_IN_BYTES],
        buffer: &mut [u8],
    ) -> Result<(), CommandResult> {
        if buffer.len() % 16 != 0
            || (buffer.len() >> BYTES_TO_PAGES_SHIFT) > u16::max_value() as usize
        {
            return Err(CommandResult::GeneralError);
        }

        // Write the initialization vector and how many pages we are going to process
        self.write_command_bytes(PAGE_1_OFFSET, init_vec);
        self.write_command_halfword(
            PAGE_LENGTH_OFFSET,
            // At least one page has to be processed.
            core::cmp::max((buffer.len() >> BYTES_TO_PAGES_SHIFT) as u16, 1),
        );

        fn process_blocks(
            cse: &CSEc,
            buffer: &mut [u8],
            sequence: Sequence,
            command: Command,
        ) -> Result<(), CommandResult> {
            // On first call page 1 is occupied by the initialization vector, so we have one less.
            // On Subsequent calls we have all at our disposal.
            let (page_offset, avail_pages) = if sequence == Sequence::First {
                (PAGE_2_OFFSET, MAX_PAGES - 1)
            } else {
                (PAGE_1_OFFSET, MAX_PAGES)
            };

            // How many bytes are we processing this round? At least one page of bytes must be
            // processed.
            let bytes = core::cmp::min(buffer.len() >> BYTES_TO_PAGES_SHIFT, avail_pages)
                * PAGE_SIZE_IN_BYTES;

            // Write our input bytes from `input`, process them, and read the processed bytes into
            // `output`.
            cse.write_command_bytes(page_offset, &buffer[..bytes]);
            cse.write_command_header(command, Format::Copy, sequence, KeyID::RamKey)?;
            cse.read_command_bytes(page_offset, &mut buffer[..bytes]);

            // Process remaining blocks, if any
            if buffer.len() - bytes != 0 {
                process_blocks(cse, &mut buffer[bytes..], Sequence::Subsequent, command)
            } else {
                Ok(())
            }
        }

        process_blocks(self, buffer, Sequence::First, command)
    }

    /// Writes the command header to `CSE_PRAM`, triggering the CSEc operation.
    /// Blocks until the operation has finished.
    fn write_command_header(
        &self,
        cmd: Command,
        cmd_format: Format,
        callseq: Sequence,
        key: KeyID,
    ) -> Result<(), CommandResult> {
        match cmd {
            Command::InitRng
            | Command::Rng
            | Command::LoadPlainKey
            | Command::EncCbc
            | Command::DecCbc
            | Command::GenerateMac
            | Command::VerifyMac => (),
            _ => unimplemented!("Command {:?}", cmd),
        };

        #[rustfmt::skip]
        self.cse_pram.embedded_ram0.write(|w| unsafe {
            w.byte_0().bits(cmd as u8)
                .byte_1().bits(cmd_format as u8)
                .byte_2().bits(callseq as u8)
                .byte_3().bits(key as u8)
        });

        // Wait until the operation has finished
        while self.ftfc.fstat.read().ccif().bit_is_clear() {}

        let status = CommandResult::from_u16(self.read_command_halfword(ERROR_BITS_OFFSET));
        match status {
            CommandResult::NoError => Ok(()),
            _ => Err(status),
        }
    }

    /// Write 32-bit words to `CSE_PRAM` starting at an offset.
    fn write_command_words(&self, offset: usize, words: &[u32]) {
        for i in 0..words.len() {
            let upper = ((words[i] & 0xffff0000) >> 16) as u16;
            let lower = ((words[i] & 0xffff) >> 0) as u16;
            self.write_command_halfword(offset, upper);
            self.write_command_halfword(offset + 2, lower);
        }
    }

    /// Reads a command half word from `CSE_PRAM` from a 16-bit aligned offset.
    fn read_command_halfword(&self, offset: usize) -> u16 {
        let page = self.read_pram(offset >> 2);
        let halfword: [u8; 2] = match (offset & 2) != 0 {
            true => [page[2], page[3]],
            false => [page[0], page[1]],
        };

        u16::from_be_bytes(halfword)
    }

    /// Writes a command half word to `CSE_PRAM` at a 16-bit aligned offset.
    /// Ported verbatim from reference code.
    fn write_command_halfword(&self, offset: usize, halfword: u16) {
        let mut page = u32::from_be_bytes(self.read_pram(offset >> 2));
        if (offset & 2) != 0 {
            page &= !LOWER_HALF_MASK;
            page |= ((halfword as u32) << LOWER_HALF_SHIFT) & LOWER_HALF_MASK;
        } else {
            page &= !UPPER_HALF_MASK;
            page |= ((halfword as u32) << LOWER_HALF_SHIFT) & UPPER_HALF_MASK;
        }

        let newpage = u8_be_array_from_u32(page);
        self.write_pram(offset >> 2, &newpage);
    }

    /// Reads a single byte from `CSE_PRAM`.
    /// Ported verbatim from reference code.
    fn read_command_byte(&self, offset: usize) -> u8 {
        let page = self.read_pram(offset >> 2);

        match offset & 0x3 {
            0x0 => page[0], // LL
            0x1 => page[1], // LU
            0x2 => page[2], // HL
            0x3 => page[3], // HU
            _ => unreachable!(),
        }
    }

    /// Writes a single byte from `CSE_PRAM`.
    /// Ported verbatim from reference code.
    fn write_command_byte(&self, offset: usize, byte: u8) {
        let page = self.read_pram(offset >> 2);
        let page: [u8; 4] = match offset & 0x3 {
            0x0 => [byte, page[1], page[2], page[3]], // LL
            0x1 => [page[0], byte, page[2], page[3]], // LU
            0x2 => [page[0], page[1], byte, page[3]], // HL
            0x3 => [page[0], page[1], page[2], byte], // HU
            _ => unreachable!(),
        };

        self.write_pram(offset >> 2, &page)
    }

    /// Reads command bytes from `CSE_PRAM` from a 32-bit aligned offset.
    /// Ported verbatim from reference code.
    fn read_command_bytes(&self, offset: usize, buf: &mut [u8]) {
        // TODO: ensure we don't read past available pages

        let mut i = 0;
        while (i + 3) < buf.len() {
            let page = self.read_pram((offset + i) >> 2);

            buf[i] = page[0];
            buf[i + 1] = page[1];
            buf[i + 2] = page[2];
            buf[i + 3] = page[3];
            i += 4;
        }

        while i < buf.len() {
            buf[i] = self.read_command_byte(offset + i);
            i += 1;
        }
    }

    /// Writes command bytes from `CSE_PRAM` from a 32-bit aligned offset.
    /// Ported verbatim from reference code.
    fn write_command_bytes(&self, offset: usize, buf: &[u8]) {
        // TODO: ensure we don't write past available pages

        let mut i = 0;
        while (i + 3) < buf.len() {
            self.write_pram(
                (offset + i) >> 2,
                &[buf[i], buf[i + 1], buf[i + 2], buf[i + 3]],
            );
            i += 4;
        }

        while i < buf.len() {
            self.write_command_byte(offset + i, buf[i]);
            i += 1;
        }
    }

    fn read_pram(&self, n: usize) -> [u8; 4] {
        let page = match n {
            0 => self.cse_pram.embedded_ram0.read().bits(),
            1 => self.cse_pram.embedded_ram1.read().bits(),
            2 => self.cse_pram.embedded_ram2.read().bits(),
            3 => self.cse_pram.embedded_ram3.read().bits(),
            4 => self.cse_pram.embedded_ram4.read().bits(),
            5 => self.cse_pram.embedded_ram5.read().bits(),
            6 => self.cse_pram.embedded_ram6.read().bits(),
            7 => self.cse_pram.embedded_ram7.read().bits(),
            8 => self.cse_pram.embedded_ram8.read().bits(),
            9 => self.cse_pram.embedded_ram9.read().bits(),
            10 => self.cse_pram.embedded_ram10.read().bits(),
            11 => self.cse_pram.embedded_ram11.read().bits(),
            12 => self.cse_pram.embedded_ram12.read().bits(),
            13 => self.cse_pram.embedded_ram13.read().bits(),
            14 => self.cse_pram.embedded_ram14.read().bits(),
            15 => self.cse_pram.embedded_ram15.read().bits(),
            16 => self.cse_pram.embedded_ram16.read().bits(),
            17 => self.cse_pram.embedded_ram17.read().bits(),
            18 => self.cse_pram.embedded_ram18.read().bits(),
            19 => self.cse_pram.embedded_ram19.read().bits(),
            20 => self.cse_pram.embedded_ram20.read().bits(),
            21 => self.cse_pram.embedded_ram21.read().bits(),
            22 => self.cse_pram.embedded_ram22.read().bits(),
            23 => self.cse_pram.embedded_ram23.read().bits(),
            24 => self.cse_pram.embedded_ram24.read().bits(),
            25 => self.cse_pram.embedded_ram25.read().bits(),
            26 => self.cse_pram.embedded_ram26.read().bits(),
            27 => self.cse_pram.embedded_ram27.read().bits(),
            28 => self.cse_pram.embedded_ram28.read().bits(),
            29 => self.cse_pram.embedded_ram29.read().bits(),
            30 => self.cse_pram.embedded_ram30.read().bits(),
            31 => self.cse_pram.embedded_ram31.read().bits(),
            _ => unreachable!(),
        };

        u8_be_array_from_u32(page)
    }

    fn write_pram(&self, n: usize, buf: &[u8; 4]) {
        let bytes = u32::from_be_bytes(*buf);

        #[rustfmt::skip]
        match n {
            0 => self.cse_pram.embedded_ram0.write(|w| unsafe { w.bits(bytes) }),
            1 => self.cse_pram.embedded_ram1.write(|w| unsafe { w.bits(bytes) }),
            2 => self.cse_pram.embedded_ram2.write(|w| unsafe { w.bits(bytes) }),
            3 => self.cse_pram.embedded_ram3.write(|w| unsafe { w.bits(bytes) }),
            4 => self.cse_pram.embedded_ram4.write(|w| unsafe { w.bits(bytes) }),
            5 => self.cse_pram.embedded_ram5.write(|w| unsafe { w.bits(bytes) }),
            6 => self.cse_pram.embedded_ram6.write(|w| unsafe { w.bits(bytes) }),
            7 => self.cse_pram.embedded_ram7.write(|w| unsafe { w.bits(bytes) }),
            8 => self.cse_pram.embedded_ram8.write(|w| unsafe { w.bits(bytes) }),
            9 => self.cse_pram.embedded_ram9.write(|w| unsafe { w.bits(bytes) }),
            10 => self.cse_pram.embedded_ram10.write(|w| unsafe { w.bits(bytes) }),
            11 => self.cse_pram.embedded_ram11.write(|w| unsafe { w.bits(bytes) }),
            12 => self.cse_pram.embedded_ram12.write(|w| unsafe { w.bits(bytes) }),
            13 => self.cse_pram.embedded_ram13.write(|w| unsafe { w.bits(bytes) }),
            14 => self.cse_pram.embedded_ram14.write(|w| unsafe { w.bits(bytes) }),
            15 => self.cse_pram.embedded_ram15.write(|w| unsafe { w.bits(bytes) }),
            16 => self.cse_pram.embedded_ram16.write(|w| unsafe { w.bits(bytes) }),
            17 => self.cse_pram.embedded_ram17.write(|w| unsafe { w.bits(bytes) }),
            18 => self.cse_pram.embedded_ram18.write(|w| unsafe { w.bits(bytes) }),
            19 => self.cse_pram.embedded_ram19.write(|w| unsafe { w.bits(bytes) }),
            20 => self.cse_pram.embedded_ram20.write(|w| unsafe { w.bits(bytes) }),
            21 => self.cse_pram.embedded_ram21.write(|w| unsafe { w.bits(bytes) }),
            22 => self.cse_pram.embedded_ram22.write(|w| unsafe { w.bits(bytes) }),
            23 => self.cse_pram.embedded_ram23.write(|w| unsafe { w.bits(bytes) }),
            24 => self.cse_pram.embedded_ram24.write(|w| unsafe { w.bits(bytes) }),
            25 => self.cse_pram.embedded_ram25.write(|w| unsafe { w.bits(bytes) }),
            26 => self.cse_pram.embedded_ram26.write(|w| unsafe { w.bits(bytes) }),
            27 => self.cse_pram.embedded_ram27.write(|w| unsafe { w.bits(bytes) }),
            28 => self.cse_pram.embedded_ram28.write(|w| unsafe { w.bits(bytes) }),
            29 => self.cse_pram.embedded_ram29.write(|w| unsafe { w.bits(bytes) }),
            30 => self.cse_pram.embedded_ram30.write(|w| unsafe { w.bits(bytes) }),
            31 => self.cse_pram.embedded_ram31.write(|w| unsafe { w.bits(bytes) }),
            _ => unreachable!(),
        };
    }
}
