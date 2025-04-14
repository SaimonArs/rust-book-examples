//! A simple thread pool implementation for executing tasks concurrently.
//!
//! The `ThreadPool` creates a fixed number of worker threads on initialization
//! and uses a channel to distribute tasks to these workers.

use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

#[derive(Debug)]
pub enum ThError {
    InvalidSize,
    PoolShutdown,
    SendFailed,
}

/// A thread pool that executes submitted tasks using a fixed number of threads.
///
/// # Examples
///
/// ```
/// use my_thread_pool::ThreadPool;
///
/// let pool = ThreadPool::new(4).unwrap();
///
/// pool.execute(|| {
///     println!("Task executed in worker thread");
/// });
/// ```

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of threads in the pool.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the thread pool if successful,
    /// or an error if thread creation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `size` is zero
    ///
    /// # Examples
    ///
    /// ```
    /// use my_thread_pool::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4).unwrap();
    /// ```
    pub fn new(size: usize) -> Result<ThreadPool, ThError> {
        if size < 1 {
            return Err(ThError::InvalidSize);
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Executes a task in the thread pool.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to execute.
    ///
    /// # Errors
    ///
    /// Returns an error if the task cannot be sent to a worker thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_thread_pool::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4).unwrap();
    ///
    /// pool.execute(|| {
    ///     println!("Task executed");
    /// }).unwrap();
    /// ```
    pub fn execute<F>(&self, f: F) -> Result<(), ThError>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        return match self.sender.as_ref() {
            Some(s) => match s.send(job) {
                Ok(_) => Ok(()),
                Err(_) => Err(ThError::SendFailed),
            },
            None => Err(ThError::PoolShutdown),
        };
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
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
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    use super::*;

    #[test]
    fn pool_creation() {
        assert!(ThreadPool::new(0).is_err());
        assert!(ThreadPool::new(4).is_ok());
        assert!(ThreadPool::new(64).is_ok());
    }

    #[test]
    fn test_execution() {
        let pool = ThreadPool::new(2).unwrap();

        let counter = Arc::new(AtomicUsize::new(0));
        for _ in 0..100 {
            let counter_clone = counter.clone();
            pool.execute(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        }

        thread::sleep(Duration::from_millis(200));

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn drop_pool() {
        let pool = ThreadPool::new(2).unwrap();

        let counter = Arc::new(AtomicUsize::new(0));
        for _ in 0..100 {
            let counter_clone = counter.clone();
            pool.execute(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        }

        drop(pool);

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }
}
