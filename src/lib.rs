#![allow(dead_code)]

use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;

// Proposed approach: Generics with associated type bounds
// Implementers create device-specific TransmitFuture objects to poll to completion

#[derive(Copy, Clone, Debug)]
pub enum UartError {
    InvalidData
}

pub struct Uart {
    busy: bool,
    error: bool,
    data: u8,
    ticks_to_send: usize,
}

impl Uart {
    pub fn new() -> Self {
        Self {
            busy: false,
            error: false,
            data: 0,
            ticks_to_send: 0
        }
    }

    fn is_idle(&self) -> bool {
        !self.busy
    }

    fn has_space(&self) -> bool {
        !self.busy
    }

    fn write_byte(&mut self, byte: u8) {
        if !self.busy {
            self.data = byte;
            self.busy = true;
            self.ticks_to_send = 3;
        }
    }

    fn make_progress(&mut self) {
        if self.busy {
            if self.ticks_to_send == 0 {
                println!("byte! {:02x}", self.data);
                if self.data == 0xff {
                    self.error = true;
                }
                self.busy = false;
            } else {
                self.ticks_to_send -= 1;
            }
        }
    }
}

pub struct Serial {
    uart: Uart,
}

impl Serial {
    pub fn new(uart: Uart) -> Serial {
        Self {
            uart
        }
    }

    pub fn write_byte_nowait(&mut self, cx: &mut Context<'_>, byte: u8) -> Poll<Result<(), UartError>> {
        self.uart.make_progress();

        if self.uart.has_space() {
            self.uart.write_byte(byte);
            println!("write_byte({:02x}) - Ok", byte);
            Poll::Ready(Ok(()))
        } else {
            println!("write_byte({:02x}) - WoudlBlock", byte);

            // TODO: save waker here and wake on interrupt
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    pub fn flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), UartError>> {
        self.uart.make_progress();

        if self.uart.error {
            self.uart.error = false;
            return Poll::Ready(Err(UartError::InvalidData));
        }

        if self.uart.is_idle() {
            println!("flush() - Ok");
            Poll::Ready(Ok(()))
        } else {
            println!("flush() - WouldBlock");

            // TODO: save waker here and wake on interrupt
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    fn write_buf_async<'a>(&'a mut self, data: &'a [u8]) -> SerialWriteFuture<'a> {
        SerialWriteFuture {
            serial: self,
            data,
        }
    }
}

pub trait AsyncWrite<'a> {
    /// Transmit error
    type Error;
    /// Transmit future for polling on completion
    type WriteFuture: Future<Output=Result<(), Self::Error>>;

    /// Transmit the provided data on the specified channel
    fn try_write(&'a mut self, data: &'a [u8]) -> Self::WriteFuture;
}

impl<'a> AsyncWrite<'a> for Serial {
    type Error = UartError;
    type WriteFuture = SerialWriteFuture<'a>;

    fn try_write(&'a mut self, data: &'a [u8]) -> SerialWriteFuture<'a> {
        self.write_buf_async(data)
    }
}

pub struct SerialWriteFuture<'a> {
    serial: &'a mut Serial,
    data: &'a [u8],
}

impl Future for SerialWriteFuture<'_> {
    type Output = Result<(), UartError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        while let Some(byte) = self.data.first() {
            match self.serial.write_byte_nowait(cx, *byte) {
                Poll::Ready(Ok(())) => {
                    self.data = &self.data[1..];
                    continue;
                },
                other => return other,
            }
        }
        self.serial.flush(cx)
    }
}
