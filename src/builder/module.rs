use super::invoke::{Invoke, Identity};
use super::code::{self, FunctionsBuilder};
use elements;

/// Module builder
pub struct ModuleBuilder<F=Identity> {
    callback: F,
    module: ModuleScaffold,
}

#[derive(Default)]
struct ModuleScaffold {
    pub functions: elements::FunctionsSection,
    pub types: elements::TypeSection,
    pub other: Vec<elements::Section>,
}

impl From<elements::Module> for ModuleScaffold {
    fn from(module: elements::Module) -> Self {
        let mut funcs: Option<elements::FunctionsSection> = None;
        let mut types: Option<elements::TypeSection> = None;

        let mut sections = module.into_sections();
        while let Some(section) = sections.pop() {
            match section {
                elements::Section::Type(sect) => { types = Some(sect); }
                elements::Section::Function(sect) => { funcs = Some(sect); }
                _ => {}
            }
        }

        ModuleScaffold {
            functions: funcs.unwrap_or_default(),
            types: types.unwrap_or_default(),
            other: sections,
        }
    }
}

impl From<ModuleScaffold> for elements::Module {
    fn from(module: ModuleScaffold) -> Self {
        let mut sections = Vec::new();

        let types = module.types;
        if types.types().len() > 0 {
            sections.push(elements::Section::Type(types));
        }
        let functions = module.functions;
        if functions.entries().len() > 0 {
            sections.push(elements::Section::Function(functions));
        }        
        sections.extend(module.other);
        elements::Module::new(sections)
    }
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
            module: Default::default(),
        }
    }

    /// Fill module with sections from iterator
    pub fn with_sections<I>(mut self, sections: I) -> Self 
        where I: IntoIterator<Item=elements::Section>
    {
        self.module.other.extend(sections);
        self
    }

    /// Add additional section
    pub fn with_section(mut self, section: elements::Section) -> Self {
        self.module.other.push(section);
        self
    }

    /// Binds to the type section, creates additional types when required
    pub fn with_signatures(mut self, bindings: code::SignatureBindings) -> Self {
        // todo bind to type section

        {
            let module = &mut self.module;

            let raw_functions: Vec<u32> = bindings.into_iter().map(|binding|
                match binding {
                    code::Signature::Inline(func_type) => {
                        module.types.types_mut().push(elements::Type::Function(func_type));
                        module.types.types().len() as u32 - 1
                    }
                    code::Signature::TypeReference(type_ref) => {
                        type_ref
                    }
                }
            ).collect();

            for function in raw_functions {
                module.functions.entries_mut().push(elements::Func::new(function));
            }
        }

        self
    }

    /// Define functions section
    pub fn functions(self) -> FunctionsBuilder<Self> {
        FunctionsBuilder::with_callback(self)
    }

    /// Build module (final step)
    pub fn build(self) -> F::Result {
        self.callback.invoke(self.module.into())
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

impl<F> Invoke<code::SignatureBindings> for ModuleBuilder<F>
    where F: Invoke<elements::Module> 
{
    type Result = Self;

    fn invoke(self, bindings: code::SignatureBindings) -> Self {
        self.with_signatures(bindings)
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

    #[test]
    fn functions() {
        let module = module()
            .functions()
                .signature().param(::elements::ValueType::I32).build()
                .bind()
            .build();

        assert_eq!(module.type_section().expect("type section to exist").types().len(), 1);
        assert_eq!(module.functions_section().expect("function section to exist").entries().len(), 1);
    }

}
