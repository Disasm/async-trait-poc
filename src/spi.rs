#[derive(Copy, Clone, Debug)]
pub enum SpiError {
    InvalidData,
    RxFifoOverflow,
}

pub struct DummySpi {
    tx_fifo: [u8; 4],
    tx_fifo_size: usize,
    rx_fifo: [u8; 4],
    rx_fifo_size: usize,
    error_fifo: bool,
    ticks_to_send: usize,
}

impl DummySpi {
    pub fn new() -> Self {
        Self {
            tx_fifo: [0; 4],
            tx_fifo_size: 0,
            rx_fifo: [0; 4],
            rx_fifo_size: 0,
            error_fifo: false,
            ticks_to_send: 0
        }
    }

    fn make_progress(&mut self) {
        if self.tx_fifo_size > 0 {
            if self.ticks_to_send == 0 {
                let byte = self.tx_fifo[0];
                self.tx_fifo[0] = 0;
                self.tx_fifo.rotate_left(1);
                self.tx_fifo_size -= 1;

                let byte = !byte;

                if self.rx_fifo_size < self.rx_fifo.len() {
                    self.rx_fifo[self.rx_fifo_size] = byte;
                    self.rx_fifo_size += 1;
                } else {
                    self.error_fifo = true;
                }

                if self.tx_fifo_size > 0 {
                    // start sending next byte
                    self.ticks_to_send = 3;
                }
            } else {
                self.ticks_to_send -= 1;
            }
        }
    }
}

impl embedded_hal::spi::FullDuplex<u8> for DummySpi {
    type Error = SpiError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.make_progress();

        if self.error_fifo {
            self.error_fifo = false;
            println!("read(): RxFifoOverflow");
            return Err(nb::Error::Other(SpiError::RxFifoOverflow));
        }

        if self.rx_fifo_size > 0 {
            let byte = self.rx_fifo[0];
            self.rx_fifo[0] = 0;
            self.rx_fifo.rotate_left(1);
            self.rx_fifo_size -= 1;

            if byte == 0x42 {
                println!("read(): InvalidData");
                return Err(nb::Error::Other(SpiError::InvalidData));
            }

            println!("read(): Ok({:02x})", byte);
            Ok(byte)
        } else {
            println!("read(): WouldBlock");
            Err(nb::Error::WouldBlock)
        }
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        if self.tx_fifo_size < self.tx_fifo.len() {
            self.tx_fifo[self.tx_fifo_size] = byte;
            if self.tx_fifo_size == 0 {
                // start sending
                self.ticks_to_send = 3;
            }
            self.tx_fifo_size += 1;

            println!("send({:02x}): Ok", byte);

            Ok(())
        } else {
            println!("send({:02x}): WouldBlock", byte);
            Err(nb::Error::WouldBlock)
        }
    }
}

impl embedded_async_sandbox::spi::transfer::Default for DummySpi {}
