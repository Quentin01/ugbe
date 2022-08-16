use std::{fs, io, path::Path};

use thiserror::Error;

pub const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

#[derive(Clone, Error, Debug)]
pub enum HeaderError {
    #[error("rom file is too small to contain a header (got '{0}', expected at least '{1}')")]
    RomTooSmall(usize, usize),

    #[error("title was not a valid UTF-8 string: {0:?}")]
    TitleNotUtf8(Vec<u8>),

    #[error("manufacturer code was not a valid UTF-8 string: {0:?}")]
    ManufacturerCodeNotUtf8(Vec<u8>),

    #[error("licensee code is using the old version ({0}) even with SGB support")]
    OldLicenseeCodeForSGBSupport(u8),

    #[error("bad header checksum (got '0x{0:02x}', expected '0x{1:02x}')")]
    BadHeaderChecksum(u8, u8),

    #[error("bad global checksum (got '0x{0:04x}', expected '0x{1:04x}')")]
    BadGlobalChecksum(u16, u16),

    #[error("the nintendo logo is invalid")]
    InvalidNintendoLogo,

    #[error("unknown rom size specified (0x{0:02x})")]
    UnknownRomSize(u8),

    #[error("rom file isn't the same size as the specified size in the header (got '{0}', expected '{1}')")]
    InvalidRomSize(usize, usize),

    #[error("unknown ram size specified (0x{0:02x})")]
    UnknownRamSize(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CGBSupport {
    Unsupported,
    Supported,
    Required,
}

impl std::fmt::Display for CGBSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Unsupported => "no",
                Self::Supported => "yes",
                Self::Required => "required",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SGBSupport {
    Unsupported,
    Supported,
}

impl std::fmt::Display for SGBSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Unsupported => "no",
                Self::Supported => "yes",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManufacturerCode(Option<String>);

impl std::fmt::Display for ManufacturerCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self.0 {
                Some(str) => str,
                None => "none",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LicenseeCode {
    New([u8; 2]),
    Old(u8),
}

impl std::fmt::Display for LicenseeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // List of licensee strings taken from https://raw.githubusercontent.com/gb-archive/salvage/master/txt-files/gbrom.txt
        write!(
            f,
            "{}",
            match self {
                LicenseeCode::New(code) => match code {
                    [0x30, 0x30] => "none".into(),
                    [0x30, 0x31] => "nintendo".into(),
                    [0x30, 0x38] => "capcom".into(),
                    [0x31, 0x33] => "electronic arts".into(),
                    [0x31, 0x38] => "hudsonsoft".into(),
                    [0x31, 0x39] => "b-ai".into(),
                    [0x32, 0x30] => "kss".into(),
                    [0x32, 0x32] => "pow".into(),
                    [0x32, 0x34] => "pcm complete".into(),
                    [0x32, 0x35] => "san-x".into(),
                    [0x32, 0x38] => "kemco japan".into(),
                    [0x32, 0x39] => "seta".into(),
                    [0x33, 0x30] => "viacom".into(),
                    [0x33, 0x31] => "nintendo".into(),
                    [0x33, 0x32] => "bandia".into(),
                    [0x33, 0x33] => "ocean/acclaim".into(),
                    [0x33, 0x34] => "konami".into(),
                    [0x33, 0x35] => "hector".into(),
                    [0x33, 0x37] => "taito".into(),
                    [0x33, 0x38] => "hudson".into(),
                    [0x33, 0x39] => "banpresto".into(),
                    [0x34, 0x31] => "ubi soft".into(),
                    [0x34, 0x32] => "atlus".into(),
                    [0x34, 0x34] => "malibu".into(),
                    [0x34, 0x36] => "angel".into(),
                    [0x34, 0x37] => "pullet-proof".into(),
                    [0x34, 0x39] => "irem".into(),
                    [0x35, 0x30] => "absolute".into(),
                    [0x35, 0x31] => "acclaim".into(),
                    [0x35, 0x32] => "activision".into(),
                    [0x35, 0x33] => "american sammy".into(),
                    [0x35, 0x34] => "konami".into(),
                    [0x35, 0x35] => "hi tech entertainment".into(),
                    [0x35, 0x36] => "ljn".into(),
                    [0x35, 0x37] => "matchbox".into(),
                    [0x35, 0x38] => "mattel".into(),
                    [0x35, 0x39] => "milton bradley".into(),
                    [0x36, 0x30] => "titus".into(),
                    [0x36, 0x31] => "virgin".into(),
                    [0x36, 0x34] => "lucasarts".into(),
                    [0x36, 0x37] => "ocean".into(),
                    [0x36, 0x39] => "electronic arts".into(),
                    [0x37, 0x30] => "infogrames".into(),
                    [0x37, 0x31] => "interplay".into(),
                    [0x37, 0x32] => "broderbund".into(),
                    [0x37, 0x33] => "sculptured".into(),
                    [0x37, 0x35] => "sci".into(),
                    [0x37, 0x38] => "t*hq".into(),
                    [0x37, 0x39] => "accolade".into(),
                    [0x38, 0x30] => "misawa".into(),
                    [0x38, 0x33] => "lozc".into(),
                    [0x38, 0x36] => "tokuma shoten i*".into(),
                    [0x38, 0x37] => "tsukuda ori*".into(),
                    [0x39, 0x31] => "chun soft".into(),
                    [0x39, 0x32] => "video system".into(),
                    [0x39, 0x33] => "ocean/acclaim".into(),
                    [0x39, 0x35] => "varie".into(),
                    [0x39, 0x36] => "yonezawa/s'pal".into(),
                    [0x39, 0x37] => "kaneko".into(),
                    [0x39, 0x39] => "pack in soft".into(),
                    _ => format!("unknown new code (0x{:02x} 0x{:02x})", code[0], code[1]),
                },
                LicenseeCode::Old(code) => match code {
                    0x00 => "none".into(),
                    0x01 => "nintendo".into(),
                    0x08 => "capcom".into(),
                    0x09 => "hot-b".into(),
                    0x0A => "jaleco".into(),
                    0x0B => "coconuts".into(),
                    0x0C => "elite systems".into(),
                    0x13 => "electronic arts".into(),
                    0x18 => "hudsonsoft".into(),
                    0x19 => "itc entertainment".into(),
                    0x1A => "yanoman".into(),
                    0x1D => "clary".into(),
                    0x1F => "virgin".into(),
                    0x24 => "pcm complete".into(),
                    0x25 => "san-x".into(),
                    0x28 => "kotobuki systems".into(),
                    0x29 => "seta".into(),
                    0x30 => "infogrames".into(),
                    0x31 => "nintendo".into(),
                    0x32 => "bandai".into(),
                    0x34 => "konami".into(),
                    0x35 => "hector".into(),
                    0x38 => "capcom".into(),
                    0x39 => "banpresto".into(),
                    0x3C => "*entertainment i".into(),
                    0x3E => "gremlin".into(),
                    0x41 => "ubi soft".into(),
                    0x42 => "atlus".into(),
                    0x44 => "malibu".into(),
                    0x46 => "angel".into(),
                    0x47 => "spectrum holoby".into(),
                    0x49 => "irem".into(),
                    0x4A => "virgin".into(),
                    0x4D => "malibu".into(),
                    0x4F => "u.s. gold".into(),
                    0x50 => "absolute".into(),
                    0x51 => "acclaim".into(),
                    0x52 => "activision".into(),
                    0x53 => "american sammy".into(),
                    0x54 => "gametek".into(),
                    0x55 => "park place".into(),
                    0x56 => "ljn".into(),
                    0x57 => "matchbox".into(),
                    0x59 => "milton bradley".into(),
                    0x5A => "mindscape".into(),
                    0x5B => "romstar".into(),
                    0x5C => "naxat soft".into(),
                    0x5D => "tradewest".into(),
                    0x60 => "titus".into(),
                    0x61 => "virgin".into(),
                    0x67 => "ocean".into(),
                    0x69 => "electronic arts".into(),
                    0x6E => "elite systems".into(),
                    0x6F => "electro brain".into(),
                    0x70 => "infogrames".into(),
                    0x71 => "interplay".into(),
                    0x72 => "broderbund".into(),
                    0x73 => "sculptered soft".into(),
                    0x75 => "the sales curve".into(),
                    0x78 => "t*hq".into(),
                    0x79 => "accolade".into(),
                    0x7A => "triffix entertainment".into(),
                    0x7C => "microprose".into(),
                    0x7F => "kemco".into(),
                    0x80 => "misawa entertainment".into(),
                    0x83 => "lozc".into(),
                    0x86 => "*tokuma shoten i".into(),
                    0x8B => "bullet-proof software".into(),
                    0x8C => "vic tokai".into(),
                    0x8E => "ape".into(),
                    0x8F => "i'max".into(),
                    0x91 => "chun soft".into(),
                    0x92 => "video system".into(),
                    0x93 => "tsuburava".into(),
                    0x95 => "varie".into(),
                    0x96 => "yonezawa/s'pal".into(),
                    0x97 => "kaneko".into(),
                    0x99 => "arc".into(),
                    0x9A => "nihon bussan".into(),
                    0x9B => "tecmo".into(),
                    0x9C => "imagineer".into(),
                    0x9D => "banpresto".into(),
                    0x9F => "nova".into(),
                    0xA1 => "hori electric".into(),
                    0xA2 => "bandai".into(),
                    0xA4 => "konami".into(),
                    0xA6 => "kawada".into(),
                    0xA7 => "takara".into(),
                    0xA9 => "technos japan".into(),
                    0xAA => "broderbund".into(),
                    0xAC => "toei animation".into(),
                    0xAD => "toho".into(),
                    0xAF => "namco".into(),
                    0xB0 => "acclaim".into(),
                    0xB1 => "ascii or nexoft".into(),
                    0xB2 => "bandai".into(),
                    0xB4 => "enix".into(),
                    0xB6 => "hal".into(),
                    0xB7 => "snk".into(),
                    0xB9 => "pony canyon".into(),
                    0xBA => "*culture brain o".into(),
                    0xBB => "sunsoft".into(),
                    0xBD => "sony imagesoft".into(),
                    0xBF => "sammy".into(),
                    0xC0 => "taito".into(),
                    0xC2 => "kemco".into(),
                    0xC3 => "squaresoft".into(),
                    0xC4 => "*tokuma shoten i".into(),
                    0xC5 => "data east".into(),
                    0xC6 => "tonkin house".into(),
                    0xC8 => "koei".into(),
                    0xC9 => "ufl".into(),
                    0xCA => "ultra".into(),
                    0xCB => "vap".into(),
                    0xCC => "use".into(),
                    0xCD => "meldac".into(),
                    0xCE => "*pony canyon or".into(),
                    0xCF => "angel".into(),
                    0xD0 => "taito".into(),
                    0xD1 => "sofel".into(),
                    0xD2 => "quest".into(),
                    0xD3 => "sigma enterprises".into(),
                    0xD4 => "ask kodansha".into(),
                    0xD6 => "naxat soft".into(),
                    0xD7 => "copya systems".into(),
                    0xD9 => "banpresto".into(),
                    0xDA => "tomy".into(),
                    0xDB => "ljn".into(),
                    0xDD => "ncs".into(),
                    0xDE => "human".into(),
                    0xDF => "altron".into(),
                    0xE0 => "jaleco".into(),
                    0xE1 => "towachiki".into(),
                    0xE2 => "uutaka".into(),
                    0xE3 => "varie".into(),
                    0xE5 => "epoch".into(),
                    0xE7 => "athena".into(),
                    0xE8 => "asmik".into(),
                    0xE9 => "natsume".into(),
                    0xEA => "king records".into(),
                    0xEB => "atlus".into(),
                    0xEC => "epic/sony records".into(),
                    0xEE => "igs".into(),
                    0xF0 => "a wave".into(),
                    0xF3 => "extreme entertainment".into(),
                    0xFF => "ljn".into(),
                    _ => format!("unknown old code (0x{:02x})", code),
                },
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationCode {
    Japan,
    Overseas,
    Invalid(u8),
}

impl std::fmt::Display for DestinationCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Japan => "japan".into(),
                Self::Overseas => "overseas".into(),
                Self::Invalid(code) => format!("invalid (0x{:02x}", code),
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    NoMBC {
        ram: bool,
        battery: bool,
    },
    MBC1 {
        ram: bool,
        battery: bool,
        multi_cart: bool,
    },
    MBC2 {
        ram: bool,
        battery: bool,
    },
    MBC3 {
        ram: bool,
        battery: bool,
        timer: bool,
    },
    MBC5 {
        ram: bool,
        battery: bool,
        rumble: bool,
    },
    MBC6,
    MBC7 {
        ram: bool,
        battery: bool,
        rumble: bool,
        sensor: bool,
    },
    MMM01 {
        ram: bool,
        battery: bool,
    },
    HuC1 {
        ram: bool,
        battery: bool,
    },
    HuC3,
    PocketCamera,
    BandaiTama5,
    Unknown(u8),
}

impl Kind {
    pub fn ram(&self) -> bool {
        match self {
            Self::NoMBC { ram, .. }
            | Self::MBC1 { ram, .. }
            | Self::MBC2 { ram, .. }
            | Self::MBC3 { ram, .. }
            | Self::MBC5 { ram, .. }
            | Self::MBC7 { ram, .. }
            | Self::MMM01 { ram, .. }
            | Self::HuC1 { ram, .. } => *ram,
            _ => false,
        }
    }

    pub fn battery(&self) -> bool {
        match self {
            Self::NoMBC { battery, .. }
            | Self::MBC1 { battery, .. }
            | Self::MBC2 { battery, .. }
            | Self::MBC3 { battery, .. }
            | Self::MBC5 { battery, .. }
            | Self::MBC7 { battery, .. }
            | Self::MMM01 { battery, .. }
            | Self::HuC1 { battery, .. } => *battery,
            _ => false,
        }
    }

    pub fn timer(&self) -> bool {
        match self {
            Self::MBC3 { timer, .. } => *timer,
            _ => false,
        }
    }

    pub fn rumble(&self) -> bool {
        match self {
            Self::MBC5 { rumble, .. } | Self::MBC7 { rumble, .. } => *rumble,
            _ => false,
        }
    }

    pub fn sensor(&self) -> bool {
        match self {
            Self::MBC7 { sensor, .. } => *sensor,
            _ => false,
        }
    }

    pub fn is_multi_cart(rom: &[u8]) -> bool {
        let nintendo_logo_count = (0..4)
            .map(|idx| {
                let start = idx * 0x40000 + 0x0104;
                let end = start + NINTENDO_LOGO.len();

                &rom[start..end]
            })
            .filter(|&possible_logo| possible_logo == NINTENDO_LOGO)
            .count();

        // From mooneye: A multicart should have at least two games + a menu with valid logo data
        nintendo_logo_count >= 3
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Kind::NoMBC { .. } => "No MBC".into(),
            Kind::MBC1 { multi_cart, .. } => {
                if *multi_cart {
                    "MMBC1".into()
                } else {
                    "MBC1".into()
                }
            }
            Kind::MBC2 { .. } => "MBC2".into(),
            Kind::MBC3 { .. } => "MBC3".into(),
            Kind::MBC5 { .. } => "MBC5".into(),
            Kind::MBC6 => "MBC6".into(),
            Kind::MBC7 { .. } => "MBC7".into(),
            Kind::MMM01 { .. } => "MMM01".into(),
            Kind::HuC1 { .. } => "Huc1".into(),
            Kind::HuC3 => "HuC3".into(),
            Kind::PocketCamera => "PocketCamera".into(),
            Kind::BandaiTama5 => "BandaiTama5".into(),
            Kind::Unknown(value) => format!("Unknown (0x{:02x})", value),
        };

        let hardwares = {
            let mut hardwares = vec![];

            if self.ram() {
                hardwares.push("RAM");
            }

            if self.battery() {
                hardwares.push("BATTERY");
            }

            if self.timer() {
                hardwares.push("TIMER");
            }

            if self.rumble() {
                hardwares.push("RUMBLE");
            }

            if self.sensor() {
                hardwares.push("SENSOR");
            }

            hardwares.join(" + ")
        };

        if hardwares.is_empty() {
            write!(f, "{}", kind)
        } else {
            write!(f, "{} ({})", kind, hardwares)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RomSize(usize);

impl RomSize {
    const BANK_SIZE: usize = 16 * 1024;

    pub fn bank_count(&self) -> usize {
        self.0 / Self::BANK_SIZE
    }
}

impl std::fmt::Display for RomSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size_in_kb = self.0 / 1024;

        if size_in_kb % 1024 == 0 {
            write!(
                f,
                "{} MiB ({} banks of 16 KiB)",
                size_in_kb / 1024,
                self.bank_count()
            )
        } else if size_in_kb > 1024 {
            write!(
                f,
                "{}.{} MiB ({} banks of 16 KiB)",
                size_in_kb / 1024,
                (size_in_kb % 1024) / 128,
                self.bank_count()
            )
        } else {
            write!(
                f,
                "{} KiB ({} banks of 16 KiB)",
                size_in_kb,
                self.bank_count()
            )
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RamSize(usize);

impl RamSize {
    const BANK_SIZE: usize = 8 * 1024;

    pub fn bank_count(&self) -> usize {
        self.0 / Self::BANK_SIZE
    }
}

impl std::fmt::Display for RamSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} KiB ({} banks of 8 KiB)",
            self.0 / 1024,
            self.bank_count()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub title: String,
    pub kind: Kind,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
    pub manufacturer_code: ManufacturerCode,
    pub licensee_code: LicenseeCode,
    pub destination_code: DestinationCode,
    pub cgb_suppport: CGBSupport,
    pub sgb_suppport: SGBSupport,
    pub rom_version: u8,
    pub checksum: u8,
    pub global_checksum: u16,
}

impl Header {
    // TODO: Allow some failure (like checksums, nintendo logo, etc) if asked
    pub fn from_rom(rom: &[u8]) -> Result<Self, HeaderError> {
        if rom.len() < 0x150 {
            return Err(HeaderError::RomTooSmall(rom.len(), 0x150));
        }

        let checksum = {
            let mut checksum: u8 = 0;

            for value in rom.iter().take(0x14D).skip(0x134) {
                checksum = checksum.wrapping_add(!value);
            }

            checksum
        };

        let rom_checksum = rom[0x14D];
        if checksum != rom_checksum {
            return Err(HeaderError::BadHeaderChecksum(checksum, rom_checksum));
        }

        let global_checksum = {
            let mut global_checksum: u16 = 0;

            for (idx, byte) in rom.iter().enumerate() {
                if (0x14E..=0x14F).contains(&idx) {
                    continue;
                }

                global_checksum = global_checksum.wrapping_add(*byte as u16);
            }

            global_checksum
        };

        let rom_global_checksum = u16::from_be_bytes([rom[0x14E], rom[0x14F]]);
        if global_checksum != rom_global_checksum {
            return Err(HeaderError::BadGlobalChecksum(
                global_checksum,
                rom_global_checksum,
            ));
        }

        let nintendo_logo = &rom[0x104..=0x133];
        if nintendo_logo != NINTENDO_LOGO {
            return Err(HeaderError::InvalidNintendoLogo);
        }

        // In old cartbridge, this byte was part of the title, but to decide whether a cartbridge supports the CGB mode
        // this byte was used as a flag and to not conflict with the title, it is not a valid ASCII character.
        let cgb_suppport = match rom[0x143] {
            0x80 => CGBSupport::Supported,
            0xC0 => CGBSupport::Required,
            _ => CGBSupport::Unsupported,
        };

        // In old cartbridge, the title was 16 bytes long, but for CGB compatible games it is 11 bytes long with the next 4
        // bytes being the manufacturer code and the next byte the CGB flag parsed above.
        let (title, manufacturer_code) = {
            let (title_range, manufacturer_code_range) = if cgb_suppport == CGBSupport::Unsupported
            {
                (0x134..=0x143, None)
            } else {
                (0x134..=0x13E, Some(0x13F..=0x142))
            };

            (
                std::str::from_utf8(&rom[title_range.clone()])
                    .map_err(|_| HeaderError::TitleNotUtf8(rom[title_range].to_vec()))?
                    .trim_matches(char::from(0))
                    .to_string(),
                match manufacturer_code_range {
                    Some(manufacturer_code_range) => Some(
                        std::str::from_utf8(&rom[manufacturer_code_range.clone()])
                            .map_err(|_| {
                                HeaderError::ManufacturerCodeNotUtf8(
                                    rom[manufacturer_code_range].to_vec(),
                                )
                            })?
                            .trim_matches(char::from(0))
                            .to_string(),
                    ),
                    None => None,
                },
            )
        };

        // Supports of SGB functions
        let sgb_suppport = match rom[0x146] {
            0x03 => SGBSupport::Supported,
            _ => SGBSupport::Unsupported,
        };

        // The licensee code pre-SGB cartbridge is one byte at 0x14B, however, on post-SGB cartbridge, the licensee code is two bytes
        // located at 0x144 and the old licensee code should be 0x33!
        let licensee_code = if sgb_suppport == SGBSupport::Supported || rom[0x14B] == 0x33 {
            if rom[0x14B] != 0x33 {
                return Err(HeaderError::OldLicenseeCodeForSGBSupport(rom[0x14B]));
            }

            // We unwrap as we know that our range is of the right size
            LicenseeCode::New(rom[0x144..=0x145].try_into().unwrap())
        } else {
            LicenseeCode::Old(rom[0x14B])
        };

        let destination_code = match rom[0x14A] {
            0x00 => DestinationCode::Japan,
            0x01 => DestinationCode::Overseas,
            code => DestinationCode::Invalid(code),
        };

        let kind = match rom[0x147] {
            0x00 => Kind::NoMBC {
                ram: false,
                battery: false,
            },
            0x08 => Kind::NoMBC {
                ram: true,
                battery: false,
            },
            0x09 => Kind::NoMBC {
                ram: true,
                battery: true,
            },

            0x01 => Kind::MBC1 {
                ram: false,
                battery: false,
                multi_cart: Kind::is_multi_cart(rom),
            },
            0x02 => Kind::MBC1 {
                ram: true,
                battery: false,
                multi_cart: Kind::is_multi_cart(rom),
            },
            0x03 => Kind::MBC1 {
                ram: true,
                battery: true,
                multi_cart: Kind::is_multi_cart(rom),
            },

            0x05 => Kind::MBC2 {
                ram: false,
                battery: false,
            },
            0x06 => Kind::MBC2 {
                ram: true,
                battery: true,
            },

            0x0F => Kind::MBC3 {
                ram: false,
                battery: true,
                timer: true,
            },
            0x10 => Kind::MBC3 {
                ram: true,
                battery: true,
                timer: true,
            },
            0x11 => Kind::MBC3 {
                ram: false,
                battery: false,
                timer: false,
            },
            0x12 => Kind::MBC3 {
                ram: true,
                battery: false,
                timer: false,
            },
            0x13 => Kind::MBC3 {
                ram: true,
                battery: true,
                timer: false,
            },

            0x19 => Kind::MBC5 {
                ram: false,
                battery: false,
                rumble: false,
            },
            0x1A => Kind::MBC5 {
                ram: true,
                battery: false,
                rumble: false,
            },
            0x1B => Kind::MBC5 {
                ram: true,
                battery: true,
                rumble: false,
            },
            0x1C => Kind::MBC5 {
                ram: false,
                battery: false,
                rumble: true,
            },
            0x1D => Kind::MBC5 {
                ram: true,
                battery: false,
                rumble: true,
            },
            0x1E => Kind::MBC5 {
                ram: true,
                battery: true,
                rumble: true,
            },

            0x20 => Kind::MBC6,

            0x22 => Kind::MBC7 {
                ram: true,
                battery: true,
                rumble: true,
                sensor: true,
            },

            0x0B => Kind::MMM01 {
                ram: false,
                battery: false,
            },
            0x0C => Kind::MMM01 {
                ram: true,
                battery: false,
            },
            0x0D => Kind::MMM01 {
                ram: true,
                battery: true,
            },

            0xFF => Kind::HuC1 {
                ram: true,
                battery: true,
            },

            0xFE => Kind::HuC3,

            0xFC => Kind::PocketCamera,

            0xFD => Kind::BandaiTama5,

            n => Kind::Unknown(n),
        };

        let rom_size = match rom[0x148] {
            value @ 0x00..=0x08 => RomSize((32 * 1024) << value),
            value @ 0x52..=0x54 => RomSize(((32 * 1024) << (value & 0xF)) + (1024 * 1024)),
            value => return Err(HeaderError::UnknownRomSize(value)),
        };

        if rom_size.0 != rom.len() {
            return Err(HeaderError::InvalidRomSize(rom.len(), rom_size.0));
        }

        let ram_size = match rom[0x149] {
            0x00 => RamSize(0),
            0x02 => RamSize(8 * 1024),
            0x03 => RamSize(32 * 1024),
            0x04 => RamSize(128 * 1024),
            0x05 => RamSize(64 * 1024),
            value => return Err(HeaderError::UnknownRamSize(value)),
        };

        Ok(Self {
            title,
            kind,
            rom_size,
            ram_size,
            manufacturer_code: ManufacturerCode(manufacturer_code),
            licensee_code,
            destination_code,
            cgb_suppport,
            sgb_suppport,
            rom_version: rom[0x14C],
            checksum: rom_checksum,
            global_checksum: rom_global_checksum,
        })
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse header")]
    ParseHeaderError(#[from] HeaderError),

    #[error("failed to read the file")]
    ReadError(#[from] io::Error),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cartridge {
    header: Header,
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
}

impl Cartridge {
    pub fn from_rom_path<P: ?Sized + AsRef<Path>>(rom_path: &P) -> Result<Self, Error> {
        let rom_file = fs::File::open(rom_path)?;
        let mut rom_reader = io::BufReader::new(rom_file);
        let mut rom_buffer = Vec::new();

        io::Read::read_to_end(&mut rom_reader, &mut rom_buffer)?;

        let header = Header::from_rom(&rom_buffer)?;

        let ram = match &header.ram_size.0 {
            0 => None,
            size => Some(vec![0; *size]),
        };

        Ok(Self {
            header,
            rom: rom_buffer,
            ram,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn rom(&self) -> &[u8] {
        &self.rom
    }

    pub fn ram(&self) -> Option<&[u8]> {
        match &self.ram {
            Some(ram) => Some(ram),
            None => todo!(),
        }
    }

    pub fn mut_ram(&mut self) -> Option<&mut [u8]> {
        match &mut self.ram {
            Some(ram) => Some(ram),
            None => todo!(),
        }
    }
}
