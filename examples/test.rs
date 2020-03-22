use async_trait_poc::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut uart = Uart::new();
    //let mut serial = Serial::new(&mut uart);
    //let mut serial = Pin::new_unchecked(&mut serial);

    uart.try_write(b"Hello, world").await.unwrap();
    //drop(serial);
    uart.try_write(b"Hello, world\xff").await.unwrap();

    Ok(())
}
