use std::io::{Read, Error};
use std::thread::JoinHandle;
use std::sync::{Mutex, Arc, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvError;
use std::borrow::Borrow;
use std::collections::VecDeque;

pub struct Skipper<R>{
	thread: Option<JoinHandle<()>>,
	slider: Arc<Mutex<VecDeque<u8>>>,
	stop:   Arc<AtomicBool>,
	done:   Arc<AtomicBool>,
	cond:   Arc<(Mutex<bool>, Condvar)>,
	_bind0: std::marker::PhantomData<R>
}
impl<R> Skipper<R>
	where R: Read + Send + 'static {

	pub fn new_with_capacity(source: R, capacity: usize) -> Self {
		let slider0 = Arc::new(Mutex::new(VecDeque::with_capacity(capacity)));
		let slider1 = slider0.clone();

		let stop0 = Arc::new(AtomicBool::new(false));
		let stop1 = stop0.clone();

		let done0 = Arc::new(AtomicBool::new(false));
		let done1 = done0.clone();

		let cond0 = Arc::new((Mutex::new(false), Condvar::new()));
		let cond1 = cond0.clone();

		let thread = std::thread::spawn(
			move || Self::handle(
				slider1,
				source,
				stop1,
				done1,
				capacity,
				cond1)
		);

		Self {
			slider: slider0,
			stop: stop0,
			done: done0,
			cond: cond0,
			thread: Some(thread),
			_bind0: Default::default()
		}
	}

	fn handle(
		slider: Arc<Mutex<VecDeque<u8>>>,
		mut source: R,
		stop: Arc<AtomicBool>,
		done: Arc<AtomicBool>,
		cap:  usize,
		cond: Arc<(Mutex<bool>, Condvar)>) {

		while !stop.load(Ordering::Relaxed) {
			let mut buffer = [0; 1024];
			let read = source.read(&mut buffer[..]).unwrap();
			if read == 0 {
				/* End of file. */
				done.store(true, Ordering::Relaxed);
				break;
			}

			let mut edit = slider.lock().unwrap();
			if edit.len() + read > cap {
				let len = edit.len();
				edit.drain(.. len + read - cap);
			}
			edit.extend(&buffer[..read]);

			std::mem::drop(edit);

			/* Now we have to wait for the  */
			let mut data = cond.0.lock().unwrap();
			*data = true;

			cond.1.notify_all();
		}

		done.store(true, Ordering::Relaxed);
	}
}
impl<R> Drop for Skipper<R> {
	fn drop(&mut self) {
		self.stop.store(true, Ordering::Relaxed);
		self.thread
			.take()
			.unwrap()
			.join()
			.unwrap();
	}
}
impl<R> Read for Skipper<R>
	where R: Read {

	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		if self.done.load(Ordering::Relaxed) {
			/* End of file. */
			return Ok(0);
		}

		let lock = self.slider.lock().unwrap();
		let mut lock = if lock.len() == 0 {
			/* We have no data to draw from, we're gonna have to wait for more.
			 *
			 * In order to do that we drop the lock on the buffer we are holding
			 * so that the reader thread can do work, then we wait for either a
			 * notification that some data is ready or the knowledge that the
			 * thread has stopped. */
			std::mem::drop(lock);

			/* Wait for some data to present itself. */
			let mut cond = self.cond.0.lock().unwrap();
			*cond = false;

			while !*cond {
				cond = self.cond.1.wait(cond).unwrap();
			}
			std::mem::drop(cond);

			/* Now that we aren't waiting on further progress from the reader
			 * thread we can reacquire the lock we let go of earlier. */
			self.slider.lock().unwrap()
		} else {
			/* Don't change the lock. */
			lock
		};

		/* Copy the data over. */
		let mut copied = 0;
		let (a, b) = lock.as_slices();

		let len = usize::min(buf.len(), b.len());
		let off_t = buf.len() - len;
		let off_s = b.len() - len;
		if len > 0 {
			buf[off_t..].copy_from_slice(&b[off_s..]);
		}

		copied += len;
		if copied == b.len() {
			let len = usize::min(buf.len() - copied, a.len());
			let off_t = buf.len() - copied - len;
			let off_s = a.len() - len;

			if len > 0 {
				buf[off_t..off_t + len].copy_from_slice(&a[off_s..]);
			}

			copied += len;
		}

		Ok(copied)
	}
}

