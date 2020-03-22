use core::future::Future;

/// Read half of a serial interface
pub trait AsyncRead {
    /// Read error
    type Error;
    /// Read byte future for polling on completion
    type ReadByteFuture<'t>: Future<Output=Result<u8, Self::Error>>;
    /// Read future for polling on completion
    type ReadFuture<'t>: Future<Output=Result<(), Self::Error>>;

    /// Reads a single byte from the serial interface
    fn read_byte(&mut self) -> Self::ReadByteFuture<'_>;

    /// Reads an array of bytes from the serial interface
    fn read<'a>(&'a mut self, data: &'a mut [u8]) -> Self::ReadFuture<'a>;
}

/// Write half of a serial interface
pub trait AsyncWrite {
    /// Write error
    type Error;
    /// Write future for polling on completion
    type WriteFuture<'t>: Future<Output=Result<(), Self::Error>>;
    /// Flush future for polling on completion
    type FlushFuture<'t>: Future<Output=Result<(), Self::Error>>;

    /// Writes an array of bytes to the serial interface
    /// When the future completes, data may not be fully transmitted.
    /// Call `flush` to ensure that no data is left buffered.
    fn write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a>;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> Self::FlushFuture<'_>;
}
