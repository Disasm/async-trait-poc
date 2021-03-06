#[derive(Copy, Clone, Debug)]
pub enum UartError {
    InvalidData
}

pub struct Uart {
    fifo: [u8; 4],
    fifo_size: usize,
    error: bool,
    ticks_to_send: usize,
}

impl Uart {
    pub fn new() -> Self {
        Self {
            fifo: [0; 4],
            fifo_size: 0,
            error: false,
            ticks_to_send: 0,
        }
    }

    fn is_idle(&self) -> bool {
        self.fifo_size == 0
    }

    fn has_space(&self) -> bool {
        self.fifo_size < self.fifo.len()
    }

    fn write_byte(&mut self, byte: u8) {
        if self.fifo_size < self.fifo.len() {
            self.fifo[self.fifo_size] = byte;
            if self.fifo_size == 0 {
                // start sending
                self.ticks_to_send = 3;
            }
            self.fifo_size += 1;
        }
    }

    fn make_progress(&mut self) {
        if self.fifo_size > 0 {
            if self.ticks_to_send == 0 {
                let byte = self.fifo[0];
                self.fifo.rotate_left(1);
                self.fifo_size -= 1;

                println!("byte! {:02x}", byte);
                if byte == 0xff {
                    self.error = true;
                }

                if self.fifo_size > 0 {
                    // start sending next byte
                    self.ticks_to_send = 3;
                }
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

    // pub fn write_byte_nowait(&mut self, cx: &mut Context<'_>, byte: u8) -> Poll<()> {
    //     self.uart.make_progress();
    //
    //     if self.uart.has_space() {
    //         self.uart.write_byte(byte);
    //         println!("write_byte({:02x}) - Ok", byte);
    //         Poll::Ready(())
    //     } else {
    //         println!("write_byte({:02x}) - WoudlBlock", byte);
    //
    //         // TODO: save waker here and wake on interrupt
    //         cx.waker().wake_by_ref();
    //         Poll::Pending
    //     }
    // }
    //
    // pub fn flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), UartError>> {
    //     self.uart.make_progress();
    //
    //     if self.uart.error {
    //         self.uart.error = false;
    //         return Poll::Ready(Err(UartError::InvalidData));
    //     }
    //
    //     if self.uart.is_idle() {
    //         println!("flush() - Ok");
    //         Poll::Ready(Ok(()))
    //     } else {
    //         println!("flush() - WouldBlock");
    //
    //         // TODO: save waker here and wake on interrupt
    //         cx.waker().wake_by_ref();
    //         Poll::Pending
    //     }
    // }
}

impl embedded_hal::serial::Write<u8> for Serial {
    type Error = UartError;

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        self.uart.make_progress();

        if self.uart.error {
            self.uart.error = false;
            return Err(nb::Error::Other(UartError::InvalidData));
        }

        if self.uart.has_space() {
            println!("write_byte({:02x}) - Ok", byte);
            self.uart.write_byte(byte);
            Ok(())
        } else {
            println!("write_byte({:02x}) - WouldBlock", byte);
            Err(nb::Error::WouldBlock)
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.uart.make_progress();

        if self.uart.error {
            self.uart.error = false;
            return Err(nb::Error::Other(UartError::InvalidData));
        }

        if self.uart.is_idle() {
            println!("flush() - Ok");
            Ok(())
        } else {
            println!("flush() - WouldBlock");
            Err(nb::Error::WouldBlock)
        }
    }
}

impl embedded_async_sandbox::serial::write::Default for Serial {}

// impl AsyncWrite for Serial {
//     type Error = UartError;
//     type WriteByteFuture<'t> = SerialWriteByteFuture;
//     type WriteFuture<'t> = SerialWriteFuture<'t>;
//     type FlushFuture<'t> = SerialFlushFuture<'t>;
//
//     fn write_byte(&mut self, byte: u8) -> Self::WriteByteFuture {
//         unimplemented!()
//     }
//
//     fn write<'a>(&'a mut self, data: &'a [u8]) -> SerialWriteFuture<'a> {
//         SerialWriteFuture {
//             serial: self,
//             data,
//         }
//     }
//
//     fn flush(&mut self) -> SerialFlushFuture {
//         SerialFlushFuture {
//             serial: self
//         }
//     }
// }
//
// pub struct SerialWriteFuture<'a> {
//     serial: &'a mut Serial,
//     data: &'a [u8],
// }
//
// impl Future for SerialWriteFuture<'_> {
//     type Output = Result<(), UartError>;
//
//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         while let Some(byte) = self.data.first() {
//             match self.serial.write_byte_nowait(cx, *byte) {
//                 Poll::Ready(()) => {
//                     self.data = &self.data[1..];
//                     continue;
//                 },
//                 Poll::Pending => return Poll::Pending,
//             }
//         }
//         Poll::Ready(Ok(()))
//     }
// }
//
// pub struct SerialFlushFuture<'a> {
//     serial: &'a mut Serial,
// }
//
// impl Future for SerialFlushFuture<'_> {
//     type Output = Result<(), UartError>;
//
//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         self.serial.flush(cx)
//     }
// }
