use async_trait_poc::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uart = Uart::new();
    let mut serial = Serial::new(uart);

    serial.try_write(b"Hello, world").await.unwrap();
    serial.try_write(b"Hello, world\xff").await.unwrap();

    Ok(())
}
