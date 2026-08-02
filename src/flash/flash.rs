// ----------- HAL interface -----------
use embedded_hal::spi::{Operation, SpiDevice};
use embedded_hal_async::delay::DelayNs;


// ----------- Error types -----------
pub enum FlashError<E> {
    Spi(E),
    InvalidDevice
}

impl<E> From<E> for FlashError<E> {
    fn from(e: E) -> Self {
        FlashError::Spi(e)
    }
}

// ----------- SPI -----------

pub struct W25Q128<SPI> {
    spi: SPI,
}

impl<SPI> W25Q128<SPI>
where
    SPI: SpiDevice,
{
    pub fn new(spi: SPI) -> Self
    where
        SPI: SpiDevice + 'static,
    {
        Self { spi }
    }

    pub async fn startup(&mut self, delay: &mut impl DelayNs) -> Result<(), FlashError<SPI::Error>> {

        delay.delay_ms(10).await;

        self.reset().await?;

        delay.delay_ms(30).await;

        let id = self.jedec_id().await?;

        if id != JEDEC_ID {
            return Err(FlashError::InvalidDevice);
        }

        Ok(())
    }

    pub async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), FlashError<SPI::Error>> {

        let cmd = [
            Command::ReadData as u8,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.spi.transaction(&mut [
            Operation::Write(&cmd),
            Operation::Read(data),
        ])?;

        Ok(())
    }

    pub async fn page_program(&mut self, address: u32, data: &[u8]) -> Result<(), FlashError<SPI::Error>> {

        assert!(data.len() <= PAGE_SIZE);

        self.write_enable().await?;

        let cmd = [
            Command::PageProgram as u8,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.spi.transaction(&mut [
            Operation::Write(&cmd),
            Operation::Write(data),
        ])?;

        self.wait_ready().await?;

        Ok(())
    }

    pub async fn erase_sector(&mut self, address: u32) -> Result<(), FlashError<SPI::Error>> {
        self.write_enable().await?;

        let mut cmd = [
            Command::SectorErase4K as u8,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.spi.transfer_in_place(&mut cmd)?;

        self.wait_ready().await?;

        Ok(())
    }

    // Helper functions

    async fn write_enable(&mut self, ) -> Result<(), FlashError<SPI::Error>> {
        let mut cmd = [Command::WriteEnable as u8];

        self.spi.transfer_in_place(&mut cmd)?;

        Ok(())
    }

    async fn read_status1(&mut self) -> Result<u8, FlashError<SPI::Error>> {
        let mut buf = [Command::ReadStatus1 as u8, 0];

        self.spi.transfer_in_place(&mut buf)?;

        Ok(buf[1])
    }

    async fn wait_ready(&mut self) -> Result<(), FlashError<SPI::Error>> {
        loop {
            let status = self.read_status1().await?;

            if (status & SR1_BUSY) == 0 {
                break;
            }
        }

        Ok(())
    }

    async fn jedec_id(&mut self) -> Result<[u8;3], FlashError<SPI::Error>> {
        let cmd = [Command::JedecId as u8];

        let mut id = [0u8;3];

        self.spi.transaction(&mut [
            Operation::Write(&cmd),
            Operation::Read(&mut id),
        ])?;

        Ok(id)
    }

    async fn reset(&mut self) -> Result<(), FlashError<SPI::Error>> {
        let mut enable = [Command::ResetEnable as u8];
        self.spi.transfer_in_place(&mut enable)?;

        let mut reset = [Command::Reset as u8];
        self.spi.transfer_in_place(&mut reset)?;

        Ok(())
    }
}
