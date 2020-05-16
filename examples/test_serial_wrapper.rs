#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![allow(dead_code)]

use async_trait_poc::serial::*;
use embedded_async_sandbox::serial::AsyncWrite;
use std::future::Future;

struct SerialWrapper<S>(S);

impl<S: AsyncWrite> AsyncWrite for SerialWrapper<S> {
    type Error = S::Error;
    type WriteByteFuture<'t> = impl Future<Output=Result<(), S::Error>>;
    type WriteFuture<'t> = impl Future<Output=Result<(), S::Error>>;
    type FlushFuture<'t> = impl Future<Output=Result<(), S::Error>>;

    fn async_write_byte(&mut self, byte: u8) -> Self::WriteByteFuture<'_> {
        async move {
            if byte == b'\n' {
                self.0.async_write_byte(b'\r').await?;
            }
            self.0.async_write_byte(byte).await?;
            Ok(())
        }
    }

    fn async_write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            for b in data {
                self.async_write_byte(*b).await?;
            }
            Ok(())
        }
    }

    fn async_flush(&mut self) -> Self::FlushFuture<'_> {
        self.0.async_flush()
    }
}

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
        self.uart.async_write(b"Hello!\n").await?;
        self.uart.async_flush().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uart = Uart::new();
    let serial = Serial::new(uart);

    //serial.write(b"Hello, world").await.unwrap();
    //serial.write(b"Hello, world\xff").await.unwrap();

    let serial_wrapper = SerialWrapper(serial);

    let mut driver = AsyncDriver::new(serial_wrapper);
    driver.send_hello().await.unwrap();

    Ok(())
}
