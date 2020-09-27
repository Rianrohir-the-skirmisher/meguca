use std::sync::{Arc, Mutex};

/// Atomically generates unique sequential u64 IDs
#[derive(Default, Clone)]
pub struct IDGenerator {
	counter: Arc<Mutex<u64>>,
}

impl IDGenerator {
	/// Create new IDGenerator starting count form 1
	pub fn new() -> Self {
		Default::default()
	}

	/// Return the next unique ID
	pub fn next(&self) -> u64 {
		let mut ptr = self.counter.lock().unwrap();
		*ptr += 1;
		*ptr
	}
}

/// Boxed error shorthand
pub type Err = Box<dyn std::error::Error + Send + Sync>;

/// Boxed error result type shorthand
pub type DynResult<T = ()> = Result<T, Err>;

/// Return a string as error
#[macro_export]
macro_rules! str_err {
	($msg:expr) => {
		return Err($msg.to_owned().into());
	};
	($fmt:expr, $( $args:tt )* ) => {
		str_err!(format!($fmt, $($args)*))
    };
}
