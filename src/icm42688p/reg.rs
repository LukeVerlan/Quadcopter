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
    AccelConfig0 = 0x50,
    AccelConfig1 = 0x53,

    // Gyro
    GyroDataX1 = 0x25,
    GyroDataX0 = 0x26,
    GyroDataY1 = 0x27,
    GyroDataY0 = 0x28,
    GyroDataZ1 = 0x29,
    GyroDataZ0 = 0x2A,
    GyroConfig0 = 0x4F,
    GyroConfig1 = 0x51,

    // TMST
    TMSTFsyncH = 0x2B,
    TMSTFsyncL = 0x2C,
    TMSTConfig = 0x54,

    // FIFO
    FIFOCountH = 0x2E,
    FIFOCountL = 0x2F,
    FIFOData = 0x30,
    FIFOConfig = 0x16,
    FIFOConfig1 = 0x5F,
    FIFOConfig2 = 0x60,
    FIFOConfig3 = 0x61,
    FIFOLostPKT0 = 0x6C,
    FIFOLostPKT1 = 0x6D,

    // Apex
    ApexData0 = 0x31,
    ApexData1 = 0x32,
    ApexData2 = 0x33,
    ApexData3 = 0x34,
    ApexData4 = 0x35,
    ApexData5 = 0x36,
    ApexConfig = 0x56,

    // Int
    IntStatus = 0x2D,
    IntStatus2 = 0x37,
    IntStatus3 = 0x38,
    IntConfig = 0x14,
    IntConfig0 = 0x63,
    IntConfig1 = 0x64,
    IntSource0 = 0x65,
    IntSource1 = 0x66,
    IntSource3 = 0x68,
    IntSource4 = 0x69,

    // Int F
    IntFConfig0 = 0x4C,
    IntFConfig1 = 0x4D,

    // Signal Path Reset
    SignalPathReset = 0x4B,

    // Power
    PwrMGMT0 = 0x4E,

    // Gyro accel config 0
    GyroAccelConfig0 = 0x52,

    // SMD Config
    SMDConfig = 0x57,

    // Fsync config
    FSyncConfig  = 0x62,

    // Self Test Config
    SelfTestConfig = 0x70,

    // Who am I reg
    WhoAmI = 0x75,

    // Reg bank select
    RegBankSel = 0x76

}

/** Bank 1 Registers */
pub enum Bank1 {

    // Sensor Config
    SensorConfig0 = 0x03,

    // Gyro Config Static
    GyroConfigStatic2 = 0x0B,
    GyroConfigStatic3 = 0x0C,
    GyroConfigStatic4 = 0x0D,
    GyroConfigStatic5 = 0x0E,
    GyroConfigStatic6 = 0x0F,
    GyroConfigStatic7 = 0x10,
    GyroConfigStatic8 = 0x11,
    GyroConfigStatic9 = 0x12,
    GyroConfigStatic10 = 0x13,

    // Gyro statics
    XGSTData = 0x5F,
    YGSTData = 0x60,
    ZGSTData = 0x61,

    // TMST
    TMSTVal0 = 0x62,
    TMSTVal1 = 0x63,
    TMSTVal2 = 0x64,

    // INTF Config
    INTFConfig4 = 0x7A,
    INTFConfig5 = 0x7B,
    INTFConfig6 = 0x7C

}

/** Bank 2 Registers */
pub enum Bank2 {

    // Accel Config Statics
    AccelConfigStatic2 = 0x03,
    AccelConfigStatic3 = 0x04,
    AccelConfigStatic4 = 0x05,

    // Accel statics
    XASTData = 0x3B,
    YASTData = 0x3C,
    ZASTData = 0x3D,

}

/** Bank 3 Registers */
pub enum Bank3 {
    CLKDIV = 0x2A
}

/** Bank 4 Registers */
pub enum Bank4 {

    // Apex Config
    ApexConfig1 = 0x40,
    ApexConfig2 = 0x41,
    ApexConfig3 = 0x42,
    ApexConfig4 = 0x43,
    ApexConfig5 = 0x44,
    ApexConfig6 = 0x45,
    ApexConfig7 = 0x46,
    ApexConfig8 = 0x47,
    ApexConfig9 = 0x48,

    // Accel Wom
    AccelWomXTHR = 0x4A,
    AccelWomYTHR = 0x4B,
    AccelWomZTHR = 0x4C,

    // Int Source
    IntSource6 = 0x4D,
    IntSource7 = 0x4E,
    IntSource8 = 0x4F,
    IntSource9 = 0x50,
    IntSource10 = 0x51,

    // Offset
    OffsetUser0 = 0x77,
    OffsetUser1 = 0x78,
    OffsetUser2 = 0x79,
    OffsetUser3 = 0x7A,
    OffsetUser4 = 0x7B,
    OffsetUser5 = 0x7C,
    OffsetUser6 = 0x7D,
    OffsetUser7 = 0x7E,
    OffsetUser8 = 0x7F

}
