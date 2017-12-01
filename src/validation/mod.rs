use elements::{Module, ResizableLimits, MemoryType};

pub struct Error(pub String);

pub fn validate_module(module: &Module) -> Result<(), Error> {
	if let Some(mem_section) = module.memory_section() {
		mem_section
			.entries()
			.iter()
			.map(MemoryType::validate)
			.collect::<Result<_, _>>()?
	}

	Ok(())
}

impl ResizableLimits {
	fn validate(&self) -> Result<(), Error> {
		if let Some(maximum) = self.maximum() {
			if self.initial() > maximum {
				return Err(Error(format!(
					"maximum limit {} is lesser than minimum {}",
					maximum,
					self.initial()
				)));
			}
		}
		Ok(())
	}
}

impl MemoryType {
	fn validate(&self) -> Result<(), Error> {
		self.limits().validate()
	}
}

#[cfg(test)]
mod tests {
	use super::validate_module;
	use builder::module;
	use elements::{BlockType, ExportEntry, External, FunctionType, GlobalEntry, GlobalType,
	               ImportEntry, InitExpr, Internal, MemoryType, Opcode, Opcodes, TableType,
	               ValueType};

	#[test]
	fn empty_is_valid() {
		let module = module().build();
		assert!(validate_module(&module).is_ok());
	}

	#[test]
	fn mem_limits() {
		// min > max
		let m = module()
			.memory()
				.with_min(10)
				.with_max(Some(9))
				.build()
			.build();
		assert!(validate_module(&m).is_err());

		// min = max
		let m = module()
			.memory()
				.with_min(10)
				.with_max(Some(10))
				.build()
			.build();
		assert!(validate_module(&m).is_ok());

		// mod is always valid without max.
		let m = module()
			.memory()
				.with_min(10)
				.build()
			.build();
		assert!(validate_module(&m).is_ok());
	}
}
