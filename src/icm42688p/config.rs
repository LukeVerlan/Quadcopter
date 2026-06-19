

/// Constants
pub const GYRO_ODR: GyroODR = GyroODR::Khz8;
pub const GYRO_FSR: GyroFSR = GyroFSR::Dps2000;

pub const ACCEL_ODR: AccelODR = AccelODR::Khz8;
pub const ACCEL_FSR: AccelFSR = AccelFSR::G16;
pub const INTERRUPTS_ENABLED: bool = false;

/// Commands
pub const ACCEL_PWR_CMD: u8 = AccelPWR::LowNoise as u8;
pub const GYRO_PWR_CMD : u8 = GyroPWR::LowNoise as u8;

pub const ACTIVE_CMD: u8 = 0b1;

/// Device Details
pub const WHO_AM_I: u8 = 0x47;

/// FOR DPS2000 (update if changes)
pub const GYRO_SENS_FACTOR: f32 = 1.0 / 16.4;

/// FOR G16 (update if changes)
pub const ACCEL_SENS_FACTOR: f32 = 1.0 / 2048.0;


/// Config Bit Masks
/// -----------------

// Gyro Sampling rate
pub enum GyroODR {
    Khz32 = 0b0001,
    Khz16 = 0b0010,
    Khz8 = 0b0011,
    Khz4 = 0b0100,
    Khz2 = 0b0101,
    Khz1 = 0b0110,
    Hz500 = 0b1111,
    Hz200 = 0b0111,
    Hz100 = 0b1000,
    Hz50 = 0b1001,
    Hz25 = 0b1010,
    Hz12_5 = 0b1011,
}

// Gyro Resolution
pub enum GyroFSR {
    Dps2000 = 0b000,
    Dps1000 = 0b001,
    Dps500 = 0b010,
    Dps250 = 0b011,
    Dps125 = 0b100,
    Dps62_5 = 0b101,
    Dps31_25 = 0b110,
    Dps15_625 = 0b111,
}

// Accel Sampling rate
pub enum AccelODR {
    Khz32 = 0b0001,
    Khz16 = 0b0010,
    Khz8 = 0b0011,
    Khz4 = 0b0100,
    Khz2 = 0b0101,
    Khz1 = 0b0110,
    Hz500 = 0b1111,
    Hz200 = 0b0111,
    Hz100 = 0b1000,
    Hz50 = 0b1001,
    Hz25 = 0b1010,
    Hz12_5 = 0b1011,
    Hz6_25 = 0b1100,
    Hz3_125 = 0b1101,
    Hz1_5625 = 0b1110,
}

// Accel resolution
pub enum AccelFSR {
    G16 = 0b000,
    G8 = 0b001,
    G4 = 0b010,
    G2 = 0b011,
}

pub enum GyroPWR {
    Off = 0b00,
    Standby = 0b01,
    LowNoise = 0b11
}

pub enum AccelPWR {
    Off = 0b00,
    Off2 = 0b01,
    LowPower = 0b10,
    LowNoise = 0b11
}