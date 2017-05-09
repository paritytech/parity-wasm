use super::invoke::{Invoke, Identity};
use elements;

pub struct ExportBuilder<F=Identity> {
    callback: F,
    field: String,
    binding: elements::Internal,
}

impl ExportBuilder {
    pub fn new() -> Self {
        ExportBuilder::with_callback(Identity)
    }
}

impl<F> ExportBuilder<F> {

    pub fn with_callback(callback: F) -> Self {
        ExportBuilder {
            callback: callback,
            field: String::new(),
            binding: elements::Internal::Function(0),
        }
    }

    pub fn field(mut self, field: &str) -> Self {
        self.field = field.to_owned();
        self
    }

    pub fn with_internal(mut self, external: elements::Internal) -> Self {
        self.binding = external;
        self
    }

    pub fn internal(self) -> ExportInternalBuilder<Self> {
        ExportInternalBuilder::with_callback(self)
    }
}

impl<F> ExportBuilder<F> where F: Invoke<elements::ExportEntry> {
    pub fn build(self) -> F::Result {
        self.callback.invoke(elements::ExportEntry::new(self.field, self.binding))
    }
}

impl<F> Invoke<elements::Internal> for ExportBuilder<F> {
    type Result = Self;
    fn invoke(self, val: elements::Internal) -> Self {
        self.with_internal(val)
    }
}

pub struct ExportInternalBuilder<F=Identity> {
    callback: F,
    binding: elements::Internal,
}

impl<F> ExportInternalBuilder<F> where F: Invoke<elements::Internal> {
    pub fn with_callback(callback: F) -> Self {
        ExportInternalBuilder{
            callback: callback,
            binding: elements::Internal::Function(0),
        }
    }

    pub fn func(mut self, index: u32) -> F::Result {
        self.binding = elements::Internal::Function(index);
        self.callback.invoke(self.binding)
    }

    pub fn memory(mut self, index: u32) -> F::Result {
        self.binding = elements::Internal::Memory(index);
        self.callback.invoke(self.binding)
    }

    pub fn table(mut self, index: u32) -> F::Result {
        self.binding = elements::Internal::Table(index);
        self.callback.invoke(self.binding)
    }

    pub fn global(mut self, index: u32) -> F::Result {
        self.binding = elements::Internal::Global(index);
        self.callback.invoke(self.binding)
    }
}

/// New builder for export entry
pub fn export() -> ExportBuilder {
    ExportBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::export;

    #[test]
    fn example() {
        let entry = export().field("memory").internal().memory(0).build();
        assert_eq!(entry.field(), "memory");
    }
}