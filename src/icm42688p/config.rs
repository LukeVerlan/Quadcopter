

/// Configurations
const GYRO_ODR: GyroODR = GyroODR::Khz4;
const GYRO_FSR: GyroFSR = GyroFSR::Dps2000;
const ACCEL_ODR: AccelODR = AccelODR::Khz4;
const ACCEL_FSR: AccelFSR = AccelFSR::G16; 

// Gyro Sampling rate
enum GyroODR {
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
enum GyroFSR {
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
enum AccelODR {
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
enum AccelFSR {
    G16 = 0b000,
    G8 = 0b001,
    G4 = 0b010,
    G2 = 0b011,
}