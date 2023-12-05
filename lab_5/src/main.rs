use crate::polynomial::Polynomial;

mod polynomial;

#[tokio::main]
async fn main() {
    let x = Polynomial::new(vec![1, 2, 3]);
    let y = Polynomial::new(vec![2, 2]);
    println!("{}", x.multiply_parallel(&y, 3));
}
