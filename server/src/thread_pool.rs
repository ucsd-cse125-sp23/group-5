use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use log::debug;

/// A thread-safe, fixed-size pool of worker threads.
///
/// The `ThreadPool` struct manages a group of worker threads that can execute tasks concurrently.
/// It creates a specified number of threads and maintains a channel for sending tasks to the worker
/// threads.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// A job is a boxed closure that can be executed by a worker thread.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Represents a worker thread in the thread pool.
///
/// A `Worker` holds an identifier and a join handle for its associated thread.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    /// Creates a new ThreadPool with the specified number of worker threads.
    ///
    /// # Panics
    ///
    /// Panics if the size is zero.
    ///
    /// # Example
    ///
    /// ```
    /// use server::thread_pool::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4);
    /// ```
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Executes a function on one of the worker threads.
    ///
    /// The function is executed in a separate thread managed by the thread pool.
    ///
    /// # Example
    ///
    /// ```
    /// use server::thread_pool::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4);
    /// pool.execute(|| println!("Hello from a worker thread!"));
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Cleans up the resources associated with the thread pool.
    ///
    /// It takes ownership of the sender, drops it, and then joins all worker threads.
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            debug!("Shutting down worker {id}.", id = worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    /// Creates a new Worker with the given identifier and receiver.
    ///
    /// A worker waits for incoming jobs through the receiver and executes them.
    /// The worker terminates when it receives an error from the receiver.
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    debug!("Worker {id} got a job; executing.", id = id);
                    job();
                }
                Err(_) => {
                    debug!("Worker {id} was told to terminate.", id = id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_thread_pool_creation() {
        let pool = ThreadPool::new(4);
        assert_eq!(pool.workers.len(), 4);
    }

    #[test]
    #[should_panic]
    fn test_thread_pool_creation_with_zero_size() {
        ThreadPool::new(0);
    }

    #[test]
    fn test_thread_pool_execute() {
        use std::sync::mpsc;
        use std::time::Duration;

        let pool = ThreadPool::new(2);
        let (tx, rx) = mpsc::channel();

        for _ in 0..4 {
            let tx_clone = tx.clone();
            pool.execute(move || {
                tx_clone.send(1).unwrap();
                thread::sleep(Duration::from_millis(100));
            });
        }

        let sum: u32 = rx.iter().take(4).sum();
        assert_eq!(sum, 4);
    }
}
