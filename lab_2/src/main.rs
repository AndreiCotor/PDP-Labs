use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

fn producer(vec1: Vec<i32>, vec2: Vec<i32>, sync: Arc<(Mutex<VecDeque<i32>>, Condvar)>) {
    if vec1.len() != vec2.len() {
        return;
    }

    for i in 0..vec1.len() {
        let (queue_mutex, cond_var) = &*sync;
        let mut queue_guard = queue_mutex.lock().unwrap();

        println!("Prod {}", i);

        queue_guard.push_back(vec1[i] * vec2[i]);
        drop(queue_guard);

        thread::sleep(Duration::from_secs(1));

        cond_var.notify_all();
    }
}

fn consumer(sync: Arc<(Mutex<VecDeque<i32>>, Condvar)>, n: i32) {
    let (queue_mutex, cond_var) = &*sync;
    let mut queue_guard = queue_mutex.lock().unwrap();

    println!("Entered consumer");

    let mut sum = 0;
    let mut consumed = 0;
    loop {
        queue_guard = cond_var.wait(queue_guard).unwrap();

        while !queue_guard.is_empty() {
            sum += queue_guard.front().unwrap();
            queue_guard.pop_front();
            consumed += 1;
        }

        if consumed == n {
            break;
        }

        println!("Cons {}: {}", consumed, sum);
    }

    println!("{}", sum);
}

fn main() {
    let vec1 = vec![3, 4, 7, 6, 8, 10];
    let vec2 = vec![6, 5, 2, 4, 10, 4];
    let sync = Arc::new(
        (Mutex::new(VecDeque::new()), Condvar::new())
    );

    let sync_clone = Arc::clone(&sync);
    let consumer = thread::spawn(move || {
        consumer(sync_clone, 6);
    });

    let producer = thread::spawn(move || {
        producer(vec1, vec2, sync);
    });

    producer.join().expect("Thread failed!");
    consumer.join().expect("Thread failed!");
}
