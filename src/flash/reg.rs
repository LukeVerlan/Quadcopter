
// Standard SPI instruction set
enum Command {
    WriteEnable = 0x06,
    WriteDisable = 0x04,

    ReadStatus1 = 0x05,
    ReadStatus2 = 0x35,
    ReadStatus3 = 0x15,

    ReadData = 0x03,
    FastRead = 0x0B,

    PageProgram = 0x02,

    SectorErase4K = 0x20,
    BlockErase32K = 0x52,
    BlockErase64K = 0xD8,
    ChipErase = 0xC7,

    JedecId = 0x9F,

    ResetEnable = 0x66,
    Reset = 0x99,
}

// Status register 1 bits
const SR1_BUSY: u8 = 1 << 0;
const SR1_WEL:  u8 = 1 << 1;

// Device constants
const PAGE_SIZE: usize = 256;
const SECTOR_SIZE: usize = 4096;
const JEDEC_ID: [u8;3] = [0xEF, 0x40, 0x18];