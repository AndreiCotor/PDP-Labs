use std::time::Instant;
use crate::polynomial::Polynomial;

mod polynomial;

#[tokio::main]
async fn main() {
    let x = Polynomial::with_size(100000);
    let y = Polynomial::with_size(100000);
    {
        let start = Instant::now();
        x.multiply(&y);
        println!("Time taken for regular n^2: {} ms", start.elapsed().as_millis())
    }
    {
        let start = Instant::now();
        x.multiply_karatsuba(&y);
        println!("Time taken for regular karatsuba: {} ms", start.elapsed().as_millis())
    }
    {
        let start = Instant::now();
        x.multiply_parallel(&y, 20);
        println!("Time taken for parallel n^2: {} ms", start.elapsed().as_millis())
    }
    {
        let start = Instant::now();
        x.multiply_karatsuba_parallel(&y).await;
        println!("Time taken for Karatsuba parallel: {} ms", start.elapsed().as_millis())
    }
}
