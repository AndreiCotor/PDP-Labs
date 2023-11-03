use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

fn producer(vec1: Vec<i32>, vec2: Vec<i32>, sync: Arc<(Mutex<Option<i32>>, Mutex<bool>, Condvar)>) {
    if vec1.len() != vec2.len() {
        return;
    }

    for i in 0..vec1.len() {
        let (last_val_mutex, _, cond_var) = &*sync;
        let mut last_val_guard = last_val_mutex.lock().unwrap();

        println!("Prod {}", i);

        *last_val_guard = Some(vec1[i] * vec2[i]);
        drop(last_val_guard);

        cond_var.notify_all();
        thread::sleep(Duration::from_secs(1));
    }

    let (_, finished_mutex, cond_var) = &*sync;
    let mut finished_guard = finished_mutex.lock().unwrap();
    *finished_guard = true;
    cond_var.notify_all();
}

fn consumer(sync: Arc<(Mutex<Option<i32>>, Mutex<bool>, Condvar)>) {
    let (last_val_mutex, finished_mutex, cond_var) = &*sync;
    let mut last_val_guard = last_val_mutex.lock().unwrap();

    println!("Entered consumer");

    let mut sum = 0;
    loop {
        last_val_guard = cond_var.wait(last_val_guard).unwrap();

        if last_val_guard.is_some() {
            sum += last_val_guard.unwrap();
            *last_val_guard = None;
        }

        println!("Cons {}", sum);

        let finished_guard = finished_mutex.lock().unwrap();
        if *finished_guard {
            break;
        }
        drop(finished_guard);
    }

    println!("{}", sum);
}

fn main() {
    let vec1 = vec![3, 4, 7, 6, 8, 10];
    let vec2 = vec![6, 5, 2, 4, 10, 4];
    let sync = Arc::new(
        (Mutex::new(None), Mutex::new(false), Condvar::new())
    );

    let sync_clone = Arc::clone(&sync);
    let consumer = thread::spawn(move || {
        consumer(sync_clone);
    });

    let producer = thread::spawn(move || {
        producer(vec1, vec2, sync);
    });

    producer.join().expect("Thread failed!");
    consumer.join().expect("Thread failed!");
}
