use elements::ResizableLimits;
use interpreter::Error;
use interpreter::VariableType;

/// Execution context.
pub struct ExecutionParams<'a, St: 'static> {
	/// State that can be used by host functions,
	pub state: &'a mut St,
}

/// Export type.
#[derive(Debug, Clone)]
pub enum ExportEntryType {
	/// Any type.
	Any,
	/// Type of global.
	Global(VariableType),
}

/// Item index in items index space.
#[derive(Debug, Clone, Copy)]
pub enum ItemIndex {
	/// Index in index space.
	IndexSpace(u32),
	/// Internal item index (i.e. index of item in items section).
	Internal(u32),
	/// External module item index (i.e. index of item in the import section).
	External(u32),
}

pub fn check_limits(limits: &ResizableLimits) -> Result<(), Error> {
	if let Some(maximum) = limits.maximum() {
		if maximum < limits.initial() {
			return Err(Error::Validation(format!("maximum limit {} is lesser than minimum {}", maximum, limits.initial())));
		}
	}

	Ok(())
}
