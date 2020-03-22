#![allow(dead_code)]

use async_trait_poc::*;

struct AsyncDriver<UART> {
    uart: UART
}

impl<'a, UART: AsyncWrite<'a>> AsyncDriver<UART> {
    pub fn new(uart: UART) -> Self {
        Self {
            uart
        }
    }

    // async fn send_hello(&'a mut self) -> Result<(), UART::Error> {
    //     self.uart.try_write(b"Hello!").await
    // }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uart = Uart::new();
    let mut serial = Serial::new(uart);

    serial.try_write(b"Hello, world").await.unwrap();
    serial.try_write(b"Hello, world\xff").await.unwrap();

    Ok(())
}
