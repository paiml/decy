// Producer-consumer pattern with pthreads
// Transpiled to safe Rust using std::sync and std::thread

use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 10;

struct Queue {
    buffer: [i32; BUFFER_SIZE],
    count: usize,
    in_idx: usize,
    out_idx: usize,
}

impl Queue {
    fn new() -> Self {
        Queue {
            buffer: [0; BUFFER_SIZE],
            count: 0,
            in_idx: 0,
            out_idx: 0,
        }
    }

    fn put(&mut self, item: i32) {
        self.buffer[self.in_idx] = item;
        self.in_idx = (self.in_idx + 1) % BUFFER_SIZE;
        self.count += 1;
    }

    fn get(&mut self) -> i32 {
        let item = self.buffer[self.out_idx];
        self.out_idx = (self.out_idx + 1) % BUFFER_SIZE;
        self.count -= 1;
        item
    }

    fn is_full(&self) -> bool {
        self.count == BUFFER_SIZE
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

struct SyncQueue {
    queue: Mutex<Queue>,
    not_empty: Condvar,
    not_full: Condvar,
}

impl SyncQueue {
    fn new() -> Self {
        SyncQueue {
            queue: Mutex::new(Queue::new()),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    fn put(&self, item: i32) {
        let mut q = self.queue.lock().unwrap();

        while q.is_full() {
            q = self.not_full.wait(q).unwrap();
        }

        q.put(item);
        self.not_empty.notify_one();
    }

    fn get(&self) -> i32 {
        let mut q = self.queue.lock().unwrap();

        while q.is_empty() {
            q = self.not_empty.wait(q).unwrap();
        }

        let item = q.get();
        self.not_full.notify_one();
        item
    }
}

fn producer(queue: Arc<SyncQueue>) {
    for i in 0..20 {
        queue.put(i);
        println!("Produced: {}", i);
        thread::sleep(Duration::from_millis(100));
    }
}

fn consumer(queue: Arc<SyncQueue>) {
    for _ in 0..20 {
        let item = queue.get();
        println!("Consumed: {}", item);
        thread::sleep(Duration::from_millis(150));
    }
}

fn main() {
    let queue = Arc::new(SyncQueue::new());

    let producer_queue = Arc::clone(&queue);
    let consumer_queue = Arc::clone(&queue);

    let producer_thread = thread::spawn(move || {
        producer(producer_queue);
    });

    let consumer_thread = thread::spawn(move || {
        consumer(consumer_queue);
    });

    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();

    // No explicit cleanup needed - Rust handles it automatically
}
