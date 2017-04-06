use super::invoke::{Invoke, Identity};
use super::code::FunctionsBuilder;
use elements;

/// Module builder
pub struct ModuleBuilder<F=Identity> {
    callback: F,
    sections: Vec<elements::Section>,
}

impl ModuleBuilder {
    /// New empty module builder
    pub fn new() -> Self {
        ModuleBuilder::with_callback(Identity)
    }
}

impl<F> ModuleBuilder<F> where F: Invoke<elements::Module> {
    /// New module builder with bound callback
    pub fn with_callback(callback: F) -> Self {
        ModuleBuilder {
            callback: callback,
            sections: Vec::new(),
        }
    }

    /// Fill module with sections from iterator
    pub fn with_sections<I>(mut self, sections: I) -> Self 
        where I: IntoIterator<Item=elements::Section>
    {
        self.sections.extend(sections);
        self
    }

    pub fn with_section(mut self, section: elements::Section) -> Self {
        self.sections.push(section);
        self
    }

    pub fn functions(self) -> FunctionsBuilder<Self> {
        FunctionsBuilder::with_callback(self)
    }

    /// Build module (final step)
    pub fn build(self) -> F::Result {
        self.callback.invoke(elements::Module::new(self.sections))
    }
}

impl<F> Invoke<elements::FunctionsSection> for ModuleBuilder<F> 
    where F: Invoke<elements::Module>
{
	type Result = Self;

	fn invoke(self, section: elements::FunctionsSection) -> Self {
		self.with_section(elements::Section::Function(section))
    }    
}

/// Start new module builder
pub fn module() -> ModuleBuilder {
    ModuleBuilder::new()
}

#[cfg(test)]
mod tests {

    use super::module;

    #[test]
    fn smoky() {
        let module = module().build();
        assert_eq!(module.sections().len(), 0);
    }

}
