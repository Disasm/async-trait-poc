use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;
use pin_project::pin_project;

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

    fn progress(&mut self) -> nb::Result<(), UartError> {
        match self.state {
            UartState::Idle => {
                Ok(())
            },
            UartState::Sending(byte, counter) => {
                if counter > 0 {
                    self.state = UartState::Sending(byte, counter - 1);
                } else {
                    self.state = UartState::Idle;
                    if byte == 0xff {
                        return Err(nb::Error::Other(UartError::InvalidData));
                    }
                }
                Err(nb::Error::WouldBlock)
            },
        }
    }

    pub fn write_byte(&mut self, byte: u8) -> nb::Result<(), UartError> {
        match self.state {
            UartState::Idle => {
                self.state = UartState::Sending(byte, 5);
                println!("write_byte({:02x}) - Ok", byte);
                Ok(())
            },
            UartState::Sending(_, _) => {
                println!("write_byte({:02x}) - WoudlBlock", byte);
                self.progress()
            },
        }
    }

    pub fn flush(&mut self) -> nb::Result<(), UartError> {
        match self.state {
            UartState::Idle => {
                println!("flush() - Ok");
                Ok(())
            },
            UartState::Sending(_, _) => {
                println!("flush() - WouldBlock");
                self.progress()
            }
        }
    }

    async fn write_byte_async(&mut self, byte: u8) -> Result<(), UartError> {
        struct Write<'a> {
            uart: &'a mut Uart,
            byte: u8,
        }

        impl Future for Write<'_> {
            type Output = Result<(), UartError>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.uart.state {
                    UartState::Idle => {
                        self.uart.state = UartState::Sending(self.byte, 5);
                        println!("write_byte({:02x}) - Ok", self.byte);
                        Poll::Ready(Ok(()))
                    },
                    UartState::Sending(byte, counter) => {
                        println!("write_byte({:02x}) - WoudlBlock", self.byte);
                        if counter > 0 {
                            self.uart.state = UartState::Sending(byte, counter - 1);
                        } else {
                            self.uart.state = UartState::Idle;
                            if byte == 0xff {
                                return Poll::Ready(Err(UartError::InvalidData));
                            }
                        }
                        Poll::Pending
                    },
                }
            }
        }
        
        Write {
            uart: self,
            byte
        }.await
    }

    async fn flush_async(&mut self) -> Result<(), UartError> {
        struct WaitIdle<'a> {
            uart: &'a mut Uart,
        }

        impl Future for WaitIdle<'_> {
            type Output = Result<(), UartError>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.uart.state {
                    UartState::Idle => {
                        Poll::Ready(Ok(()))
                    },
                    UartState::Sending(byte, counter) => {
                        if counter > 0 {
                            self.uart.state = UartState::Sending(byte, counter - 1);
                        } else {
                            self.uart.state = UartState::Idle;
                            if byte == 0xff {
                                return Poll::Ready(Err(UartError::InvalidData));
                            }
                        }
                        Poll::Pending
                    },
                }
            }
        }

        WaitIdle {
            uart: self
        }.await
    }
}

pub struct Serial<'a> {
    uart: &'a mut Uart,
}

impl<'a> Serial<'a> {
    pub fn new(uart: &'a mut Uart) -> Serial<'a> {
        Self {
            uart
        }
    }
}

pub trait AsyncWrite<'a> {
    /// Transmit error
    type Error;
    /// Transmit future for polling on completion
    type WriteFuture: 'a + Future<Output=Result<(), Self::Error>>;

    /// Transmit the provided data on the specified channel
    fn try_write(&'a mut self, data: &'a [u8]) -> Self::WriteFuture;
}

impl<'a> AsyncWrite<'a> for Serial<'a> {
    type Error = UartError;
    type WriteFuture = SerialWriteFuture<'a>;

    fn try_write(&'a mut self, data: &'a [u8]) -> SerialWriteFuture<'a> {
        SerialWriteFuture {
            uart: self.uart,
            data,
            offset: 0
        }
    }
}

pub struct SerialWriteFuture<'a> {
    uart: &'a mut Uart,
    data: &'a [u8],
    offset: usize,
}

impl Future for SerialWriteFuture<'_> {
    type Output = Result<(), UartError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.get_mut();

        if this.offset == this.data.len() {
            match this.uart.flush() {
                Ok(()) => Poll::Ready(Ok(())),
                Err(nb::Error::WouldBlock) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                },
                Err(nb::Error::Other(e)) => Poll::Ready(Err(e))
            }
        } else {
            let byte = this.data[this.offset];
            match this.uart.write_byte(byte) {
                Ok(()) => {
                    this.offset += 1;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(nb::Error::WouldBlock) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                },
                Err(nb::Error::Other(e)) => Poll::Ready(Err(e))
            }
        }
    }
}
