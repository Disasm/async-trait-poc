#![allow(dead_code)]

use async_trait_poc::spi::*;
use embedded_async_sandbox::spi::AsyncTransfer;

struct AsyncDriver<SPI> {
    spi: SPI
}

impl<SPI: AsyncTransfer> AsyncDriver<SPI> {
    pub fn new(spi: SPI) -> Self {
        Self {
            spi
        }
    }

    async fn check_loopback(&mut self) -> Result<(), SPI::Error> {
        let mut buf = [0; 32];
        for (i, b) in buf.iter_mut().enumerate() {
            *b = (i+1) as u8;
        }

        self.spi.async_transfer(&mut buf).await?;

        for (i, b) in buf.iter().enumerate() {
            assert_eq!(*b, !((i+1) as u8));
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spi = DummySpi::new();

    let mut driver = AsyncDriver::new(spi);
    driver.check_loopback().await.unwrap();

    Ok(())
}
