use super::invoke::{Invoke, Identity};
use super::code::{self, SignaturesBuilder, FunctionBuilder};
use super::import;
use elements;

/// Module builder
pub struct ModuleBuilder<F=Identity> {
    callback: F,
    module: ModuleScaffold,
}

/// Location of the internal module function
pub struct CodeLocation {
    /// Location (index in 'functions' section) of the signature
    pub signature: u32,
    /// Location (index in the 'code' section) of the body
    pub body: u32,
}

#[derive(Default)]
struct ModuleScaffold {
    pub functions: elements::FunctionsSection,
    pub types: elements::TypeSection,
    pub import: elements::ImportSection,
    pub code: elements::CodeSection,
    pub other: Vec<elements::Section>,
}

impl From<elements::Module> for ModuleScaffold {
    fn from(module: elements::Module) -> Self {
        let mut funcs: Option<elements::FunctionsSection> = None;
        let mut types: Option<elements::TypeSection> = None;
        let mut import: Option<elements::ImportSection> = None;
        let mut code: Option<elements::CodeSection> = None;

        let mut sections = module.into_sections();
        while let Some(section) = sections.pop() {
            match section {
                elements::Section::Type(sect) => { types = Some(sect); }
                elements::Section::Function(sect) => { funcs = Some(sect); }
                elements::Section::Import(sect) => { import = Some(sect); }
                elements::Section::Code(sect) => { code = Some(sect); }
                _ => {}
            }
        }

        ModuleScaffold {
            functions: funcs.unwrap_or_default(),
            types: types.unwrap_or_default(),
            import: import.unwrap_or_default(),
            code: code.unwrap_or_default(),
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
        let import = module.import;
        if import.entries().len() > 0 {
            sections.push(elements::Section::Import(import));
        }                
        let functions = module.functions;
        if functions.entries().len() > 0 {
            sections.push(elements::Section::Function(functions));
        }        
        let code = module.code;
        if code.bodies().len() > 0 {
            sections.push(elements::Section::Code(code));
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

    /// Builder from raw module
    pub fn with_module(mut self, module: elements::Module) -> Self {
        self.module = module.into();
        self
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
        self.push_signatures(bindings);
        self
    }

    /// Push stand-alone function definition, creating sections, signature and code blocks
    /// in corresponding sections.
    /// `FunctionDefinition` can be build using `builder::function` builder
    pub fn push_function(&mut self, func: code::FunctionDefinition) -> CodeLocation {
        let signature = func.signature;
        let body = func.code;

        let type_ref = self.resolve_type_ref(signature);

        self.module.functions.entries_mut().push(elements::Func::new(type_ref));
        let signature_index = self.module.functions.entries_mut().len() as u32 - 1;
        self.module.code.bodies_mut().push(body);
        let body_index = self.module.code.bodies_mut().len() as u32 - 1;

        CodeLocation {
            signature: signature_index,
            body: body_index,
        }
    }

    fn resolve_type_ref(&mut self, signature: code::Signature) -> u32 {
        match signature {
            code::Signature::Inline(func_type) => {
                // todo: maybe search for existing type
                self.module.types.types_mut().push(elements::Type::Function(func_type));
                self.module.types.types().len() as u32 - 1
            }
            code::Signature::TypeReference(type_ref) => {
                type_ref
            }
        }
    }

    /// Push one function signature, returning it's calling index.
    /// Can create corresponding type in type section.
    pub fn push_signature(&mut self, signature: code::Signature) -> u32 {
        self.resolve_type_ref(signature)
    }

    /// Push signatures in the module, returning corresponding indices of pushed signatures
    pub fn push_signatures(&mut self, signatures: code::SignatureBindings) -> Vec<u32> {
        signatures.into_iter().map(|binding|
            self.resolve_type_ref(binding)
        ).collect()
    }

    /// Push import entry to module. Not that it does not update calling indices in
    /// function bodies.
    pub fn push_import(&mut self, import: elements::ImportEntry) -> u32 {
        self.module.import.entries_mut().push(import);
        // todo: actually update calling addresses in function bodies
        // todo: also batch push

        self.module.import.entries_mut().len() as u32 - 1
    }

    /// Add new function using dedicated builder
    pub fn function(self) -> FunctionBuilder<Self> {
        FunctionBuilder::with_callback(self)
    }

    /// Define functions section
    pub fn functions(self) -> SignaturesBuilder<Self> {
        SignaturesBuilder::with_callback(self)
    }

    /// With inserted import entry
    pub fn with_import(mut self, entry: elements::ImportEntry) -> Self {
        self.module.import.entries_mut().push(entry);
        self
    }

    /// Import entry builder
    pub fn import(self) -> import::ImportBuilder<Self> {
        import::ImportBuilder::with_callback(self)
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


impl<F> Invoke<code::FunctionDefinition> for ModuleBuilder<F>
    where F: Invoke<elements::Module> 
{
    type Result = Self;

    fn invoke(self, def: code::FunctionDefinition) -> Self {
        let mut b = self;
        b.push_function(def);
        b
    }
}

impl<F> Invoke<elements::ImportEntry> for ModuleBuilder<F>
    where F: Invoke<elements::Module> 
{
    type Result = Self;

    fn invoke(self, entry: elements::ImportEntry) -> Self::Result {
        self.with_import(entry)
    }
}

/// Start new module builder
pub fn module() -> ModuleBuilder {
    ModuleBuilder::new()
}

/// Start builder to extend existing module
pub fn from_module(module: elements::Module) -> ModuleBuilder {
    ModuleBuilder::new().with_module(module)
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
            .function()
                .signature().param().i32().build()
                .body().build()
                .build()
            .build();

        assert_eq!(module.type_section().expect("type section to exist").types().len(), 1);
        assert_eq!(module.functions_section().expect("function section to exist").entries().len(), 1);
        assert_eq!(module.code_section().expect("code section to exist").bodies().len(), 1);
    }

}
