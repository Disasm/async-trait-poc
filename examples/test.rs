#![allow(dead_code)]

use async_trait_poc::*;

struct AsyncDriver<UART> {
    uart: UART
}

impl<UART: AsyncWrite> AsyncDriver<UART> {
    pub fn new(uart: UART) -> Self {
        Self {
            uart
        }
    }

    async fn send_hello(&mut self) -> Result<(), UART::Error> {
        self.uart.write(b"Hello!").await?;
        self.uart.flush().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uart = Uart::new();
    let serial = Serial::new(uart);

    //serial.write(b"Hello, world").await.unwrap();
    //serial.write(b"Hello, world\xff").await.unwrap();

    let mut driver = AsyncDriver::new(serial);
    driver.send_hello().await.unwrap();

    Ok(())
}
