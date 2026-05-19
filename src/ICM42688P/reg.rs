#![no_std]
#![no_main]

/** Register Bank number */
pub enum BankNum {
    Bank0 = 0x00,
    Bank1 = 0x01,
    Bank2 = 0x02,
    Bank3 = 0x03,
    Bank4 = 0x04,
}

/** Bank 0 Registers */
pub enum Bank0 {

    // Configs
    DeviceConfig = 0x11,
    DriveConfig = 0x13,
    IntConfig = 0x14,
    FIFOConfig = 0x16,

    // Temp
    TempData1 = 0x1D,
    TempData0 = 0x1E,

    // Accel
    AccelDataX1 = 0x1F,
    AccelDataX0 = 0x20,
    AccelDataY1 = 0x21,
    AccelDataY0 = 0x22,
    AccelDataZ1 = 0x23,
    AccelDataZ0 = 0x24,

    // Gyro
    GyroDataX1 = 0x25,
    GyroDataX0 = 0x26,
    GyroDataY1 = 0x27,
    GyroDataY0 = 0x28,
    GyroDataZ1 = 0x29,
    GyroDataZ0 = 0x2A,

    //TMST
    TMSTFsyncH = 0x2B,
    TMSTFsyncL = 0x2C
}

/** Bank 1 Registers */
pub enum Bank1 {

}

/** Bank 2 Registers */
pub enum Bank2 {

}

/** Bank 3 Registers */
pub enum Bank3 {

}

/** Bank 4 Registers */
pub enum Bank4 {

}
