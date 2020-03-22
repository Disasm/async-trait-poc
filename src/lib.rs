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

#[derive(Copy, Clone)]
enum UartState {
    Idle,
    Sending(u8, usize),
}

pub struct Uart {
    state: UartState
}

impl Uart {
    pub fn new() -> Self {
        Self {
            state: UartState::Idle
        }
    }

    fn progress(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), UartError>> {
        match self.state {
            UartState::Idle => {
                Poll::Ready(Ok(()))
            },
            UartState::Sending(byte, counter) => {
                if counter > 0 {
                    self.state = UartState::Sending(byte, counter - 1);
                } else {
                    self.state = UartState::Idle;
                    if byte == 0xff {
                        return Poll::Ready(Err(UartError::InvalidData));
                    }
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            },
        }
    }

    pub fn write_byte(&mut self, cx: &mut Context<'_>, byte: u8) -> Poll<Result<(), UartError>> {
        match self.state {
            UartState::Idle => {
                self.state = UartState::Sending(byte, 5);
                println!("write_byte({:02x}) - Ok", byte);
                Poll::Ready(Ok(()))
            },
            UartState::Sending(_, _) => {
                println!("write_byte({:02x}) - WoudlBlock", byte);
                self.progress(cx)
            },
        }
    }

    pub fn flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), UartError>> {
        match self.state {
            UartState::Idle => {
                println!("flush() - Ok");
                Poll::Ready(Ok(()))
            },
            UartState::Sending(_, _) => {
                println!("flush() - WouldBlock");
                self.progress(cx)
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
        if let Some(byte) = self.data.first() {
            match self.serial.uart.write_byte(cx, *byte) {
                Poll::Ready(Ok(())) => {
                    self.data = &self.data[1..];
                    cx.waker().wake_by_ref();
                    Poll::Pending
                },
                other => other,
            }
        } else {
            self.serial.uart.flush(cx)
        }
    }
}
