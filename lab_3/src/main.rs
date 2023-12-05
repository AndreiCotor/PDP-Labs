use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use threadpool::ThreadPool;

type Matrix<T> = Vec<Vec<T>>;

fn compute_element(
    line: usize,
    column: usize,
    matrix1: &Arc<Matrix<i32>>,
    matrix2: &Arc<Matrix<i32>>,
) -> i32 {
    let mut result = 0;
    for intermediary in 0..matrix2.len() {
        let x = matrix1.get(line).unwrap().get(intermediary).unwrap();
        let y = matrix2.get(intermediary).unwrap().get(column).unwrap();

        result += x * y;
    }

    result
}

fn compute_consecutive_row(
    group: usize,
    total: usize,
    matrix1: Arc<Matrix<i32>>,
    matrix2: Arc<Matrix<i32>>,
    result: Matrix<Arc<AtomicI32>>,
) {
    let n = matrix1.len();
    let m = matrix2.get(0).unwrap().len();
    let mut steps_per_group = (n * m) / total;
    let steps_until_now = steps_per_group * group;
    let mut line = steps_until_now / m;
    let mut column = steps_until_now % m;

    if group + 1 == total {
        steps_per_group = n * m - steps_until_now;
    }

    for _ in 0..steps_per_group {
        let element = compute_element(line, column, &matrix1, &matrix2);
        result[line][column]
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| Some(element))
            .unwrap();

        column += 1;
        if column == m {
            line += 1;
            column = 0;
        }
    }
}

fn compute_consecutive_column(
    group: usize,
    total: usize,
    matrix1: Arc<Matrix<i32>>,
    matrix2: Arc<Matrix<i32>>,
    result: Matrix<Arc<AtomicI32>>,
) {
    let n = matrix1.len();
    let m = matrix2.get(0).unwrap().len();
    let mut steps_per_group = (n * m) / total;
    let steps_until_now = steps_per_group * group;
    let mut line = steps_until_now % n;
    let mut column = steps_until_now / n;

    if group + 1 == total {
        steps_per_group = n * m - steps_until_now;
    }

    for _ in 0..steps_per_group {
        let element = compute_element(line, column, &matrix1, &matrix2);
        result[line][column]
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| Some(element))
            .unwrap();

        line += 1;
        if line == n {
            column += 1;
            line = 0;
        }
    }
}

fn compute_consecutive_kth(
    group: usize,
    total: usize,
    matrix1: Arc<Matrix<i32>>,
    matrix2: Arc<Matrix<i32>>,
    result: Matrix<Arc<AtomicI32>>,
) {
    let mut index = group;
    let n = matrix1.len();
    let m = matrix2.get(0).unwrap().len();
    while index < n * m {
        let line = index / m;
        let column = index % m;

        let element = compute_element(index / m, index % m, &matrix1, &matrix2);
        result[line][column]
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| Some(element))
            .unwrap();
        index += total;
    }
}

fn build_matrix(n: usize, m: usize) -> Matrix<Arc<AtomicI32>> {
    let mut result = Vec::new();

    for _ in 0..n {
        let mut line = Vec::new();
        for _ in 0..m {
            line.push(Arc::new(AtomicI32::new(0)));
        }
        result.push(line);
    }

    result
}

fn compute_classic_threads<F>(
    threads: usize,
    matrix1: Arc<Matrix<i32>>,
    matrix2: Arc<Matrix<i32>>,
    strategy: F,
) where
    F: Fn(usize, usize, Arc<Matrix<i32>>, Arc<Matrix<i32>>, Matrix<Arc<AtomicI32>>)
        + Sync
        + Send
        + 'static,
{
    let now = Instant::now();

    let mut threads_vec = Vec::with_capacity(threads);
    let n = matrix1.len();
    let m = matrix2.get(0).unwrap().len();
    let strategy_arc = Arc::new(strategy);
    let result = build_matrix(n, m);

    for thread in 0..threads {
        let matrix1_copy = Arc::clone(&matrix1);
        let matrix2_copy = Arc::clone(&matrix2);
        let result_copy = result.clone();
        let strategy_copy = Arc::clone(&strategy_arc);

        threads_vec.push(thread::spawn(move || {
            strategy_copy(thread, threads, matrix1_copy, matrix2_copy, result_copy);
        }));
    }

    threads_vec.into_iter().for_each(|thread| {
        thread.join().expect("Thread failed!");
    });

    //println!("{:?}", result);
    let elapsed_time = now.elapsed();
    println!("Running took {} ms.", elapsed_time.as_millis());
}

fn compute_thread_pool<F>(
    threads: usize,
    matrix1: Arc<Matrix<i32>>,
    matrix2: Arc<Matrix<i32>>,
    strategy: F,
) where
    F: Fn(usize, usize, Arc<Matrix<i32>>, Arc<Matrix<i32>>, Matrix<Arc<AtomicI32>>)
        + Sync
        + Send
        + 'static,
{
    let now = Instant::now();

    let n = matrix1.len();
    let m = matrix2.get(0).unwrap().len();
    let strategy_arc = Arc::new(strategy);
    let result = build_matrix(n, m);

    let pool = ThreadPool::new(16);

    for thread in 0..threads {
        let matrix1_copy = Arc::clone(&matrix1);
        let matrix2_copy = Arc::clone(&matrix2);
        let result_copy = result.clone();
        let strategy_copy = Arc::clone(&strategy_arc);

        pool.execute(move || {
            strategy_copy(thread, threads, matrix1_copy, matrix2_copy, result_copy);
        });
    }

    pool.join();
    //println!("{:?}", result);

    let elapsed_time = now.elapsed();
    println!("Running took {} ms.", elapsed_time.as_millis());
}

fn main() {
    const NUM_THREADS: usize = 16;

    let mut opt = Vec::new();
    let mut opt2 = Vec::new();
    for _ in 0..500 {
        let mut line1 = Vec::new();
        let mut line2 = Vec::new();
        for _ in 0..500 {
            line1.push(1);
            line2.push(2);
        }

        opt.push(line1);
        opt2.push(line2);
    }

    let matrix1 = Arc::new(opt);
    let matrix2 = Arc::new(opt2);

    println!("Classic - consecutive rows");
    compute_classic_threads(
        NUM_THREADS,
        matrix1.clone(),
        matrix2.clone(),
        compute_consecutive_row,
    );
    println!("Classic - consecutive column");
    compute_classic_threads(
        NUM_THREADS,
        matrix1.clone(),
        matrix2.clone(),
        compute_consecutive_column,
    );
    println!("Classic - kth");
    compute_classic_threads(
        NUM_THREADS,
        matrix1.clone(),
        matrix2.clone(),
        compute_consecutive_kth,
    );

    println!("Thread pool - consecutive rows");
    compute_thread_pool(
        NUM_THREADS,
        matrix1.clone(),
        matrix2.clone(),
        compute_consecutive_row,
    );

    println!("Thread pool - consecutive column");
    compute_thread_pool(
        NUM_THREADS,
        matrix1.clone(),
        matrix2.clone(),
        compute_consecutive_column,
    );

    println!("Thread pool - kth");
    compute_thread_pool(NUM_THREADS, matrix1, matrix2, compute_consecutive_kth);
}
