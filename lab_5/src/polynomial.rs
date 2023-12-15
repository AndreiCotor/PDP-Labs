use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::{ops, thread};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use async_recursion::async_recursion;

#[derive(Clone)]
pub struct Polynomial {
    coef: Vec<i64>
}

impl Display for Polynomial {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for (power, coef) in self.coef.iter().enumerate() {
            result = result + &*format!("{}x^{} ", coef, power);
            if power < self.coef.len() - 1 {
                result += " + ";
            }
        }

        write!(f, "{}", result)
    }
}

impl Polynomial {
    pub fn new(coef: Vec<i64>) -> Self {
        Polynomial {
            coef
        }
    }

    pub fn with_size(size: usize) -> Self {
        let mut vec = vec![];
        for i in 0..size {
            vec.push(i as i64);
        }

        Polynomial::new(vec)
    }

    fn split(&self, low_half_size: usize) -> (Polynomial, Polynomial) {
        let boundary = min(low_half_size, self.coef.len());

        let mut low_half = vec![0; boundary];
        let mut high_half = vec![0; self.coef.len() - boundary];

        for i in 0..boundary {
            low_half[i] = self.coef[i];
        }

        for i in boundary..self.coef.len() {
            high_half[i - boundary] = self.coef[i];
        }

        (Polynomial::new(low_half), Polynomial::new(high_half))
    }

    pub fn multiply(&self, other: &Polynomial) -> Polynomial {
        let res_degree = self.coef.len() - 1 + other.coef.len() - 1;
        let mut res = vec![0; res_degree + 1];

        for (power1, coef1) in self.coef.iter().enumerate() {
            for (power2, coef2) in other.coef.iter().enumerate() {
                res[power1 + power2] += coef1 * coef2;
            }
        }

        Polynomial::new(res)
    }

    fn multiply_parallel_batch(&self, other: Arc<Polynomial>, left: usize, right: usize, res: Vec<Arc<AtomicI64>>) {
        for i in left..right {
            let self_left = max(0, (i as i64) - (other.coef.len() as i64) + 1);
            let self_right = min(self.coef.len() as i64, i as i64 + 1);

            for j in self_left..self_right {
                let other_index = i as i64 - j;
                let add = self.coef[j as usize] * other.coef[other_index as usize];
                res[i].fetch_add(add, Ordering::Relaxed);
            }
        }
    }

    pub fn multiply_parallel(&self, other: &Polynomial, num_threads: usize) -> Polynomial {
        let res_degree = self.coef.len() - 1 + other.coef.len() - 1;
        let res_size = res_degree + 1;
        let mut res = vec![];
        for _ in 0..res_size {
            res.push(Arc::new(AtomicI64::new(0)));
        }

        let poly1 = Arc::new(self.clone());
        let poly2 = Arc::new(other.clone());

        let mut handles = vec![];
        for thread in 0..num_threads {
            let res_copy = res.clone();
            let poly1_copy = poly1.clone();
            let poly2_copy = poly2.clone();

            let handle = thread::spawn(move || {
                let left = (thread * res_size) / num_threads;
                let right = ((thread + 1) * res_size) / num_threads;

                poly1_copy.multiply_parallel_batch(poly2_copy, left, right, res_copy);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        Polynomial::new(res.iter().map(|el| el.load(Ordering::Relaxed)).collect())
    }

    pub fn multiply_karatsuba(&self, other: &Polynomial) -> Polynomial {
        if self.coef.len() <= 16 || other.coef.len() <= 16 {
            return self.multiply(other);
        }

        let res_degree = self.coef.len() - 1 + other.coef.len() - 1;
        let mut res = vec![0; res_degree + 1];

        let n = max(self.coef.len(), other.coef.len()) / 2;

        let (x_low, x_high) = self.split(n);
        let (y_low, y_high) = other.split(n);

        let a = x_high.multiply_karatsuba(&y_high);
        let d = x_low.multiply_karatsuba(&y_low);
        let e = (x_high + &x_low).multiply_karatsuba(&(y_high + &y_low)) - &a - &d;

        for i in 0..a.coef.len() {
            res[i + 2*n] += a.coef[i];
        }

        for i in 0..e.coef.len() {
            res[i + n] += e.coef[i];
        }

        for i in  0..d.coef.len() {
            res[i] += d.coef[i];
        }

        Polynomial::new(res)
    }


    #[async_recursion]
    pub async fn multiply_karatsuba_parallel(&self, other: &Polynomial) -> Polynomial {
        if self.coef.len() <= 16 || other.coef.len() <= 16 {
            return self.multiply(other);
        }

        let res_degree = self.coef.len() - 1 + other.coef.len() - 1;
        let mut res = vec![0; res_degree + 1];

        let n = max(self.coef.len(), other.coef.len()) / 2;

        let (x_low, x_high) = self.split(n);
        let (y_low, y_high) = other.split(n);

        let a_future = x_high.multiply_karatsuba_parallel(&y_high);
        let d_future = x_low.multiply_karatsuba_parallel(&y_low);

        let a = a_future.await;
        let d = d_future.await;

        let x_sum = x_high + &x_low;
        let y_sum = y_high + &y_low;
        let e_future = x_sum.multiply_karatsuba_parallel(&y_sum);

        let e = e_future.await - &a - &d;

        for i in 0..a.coef.len() {
            res[i + 2*n] += a.coef[i];
        }

        for i in 0..e.coef.len() {
            res[i + n] += e.coef[i];
        }

        for i in  0..d.coef.len() {
            res[i] += d.coef[i];
        }

        Polynomial::new(res)
    }
}

impl ops::Add<&Polynomial> for Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: &Polynomial) -> Self::Output {
        let mut result = vec![0; max(self.coef.len(), rhs.coef.len())];

        for i in 0..max(self.coef.len(), rhs.coef.len()) {
            let x = self.coef.get(i).unwrap_or(&0);
            let y = rhs.coef.get(i).unwrap_or(&0);

            result[i] = x + y;
        }

        Polynomial::new(result)
    }
}

impl ops::Sub<&Polynomial> for Polynomial {
    type Output = Polynomial;

    fn sub(self, rhs: &Polynomial) -> Self::Output {
        let mut result = vec![0; max(self.coef.len(), rhs.coef.len())];

        for i in 0..max(self.coef.len(), rhs.coef.len()) {
            let x = self.coef.get(i).unwrap_or(&0);
            let y = rhs.coef.get(i).unwrap_or(&0);

            result[i] = x - y;
        }

        Polynomial::new(result)
    }
}
