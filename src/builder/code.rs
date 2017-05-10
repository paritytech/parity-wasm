use elements;
use super::invoke::{Invoke, Identity};
use super::misc::{ValueTypeBuilder, ValueTypesBuilder, OptionalValueTypeBuilder};

pub enum Signature {
    TypeReference(u32),
    Inline(elements::FunctionType),
}

pub struct SignatureBuilder<F=Identity> {
    callback: F,
    signature: elements::FunctionType,
}

impl SignatureBuilder {
    pub fn new() -> Self {
        SignatureBuilder::with_callback(Identity)
    }
}

impl<F> SignatureBuilder<F> where F: Invoke<elements::FunctionType> {
    pub fn with_callback(callback: F) -> Self {
        SignatureBuilder { 
            callback: callback, 
            signature: elements::FunctionType::default(),
        }
    }

    pub fn with_param(mut self, value_type: elements::ValueType) -> Self {
        self.signature.params_mut().push(value_type);
        self
    }

    pub fn with_params(mut self, value_types: Vec<elements::ValueType>) -> Self {
        self.signature.params_mut().extend(value_types);
        self
    }

    pub fn with_return_type(mut self, return_type: Option<elements::ValueType>) -> Self {
        *self.signature.return_type_mut() = return_type;
        self
    }

    pub fn param(self) -> ValueTypeBuilder<Self> {
        ValueTypeBuilder::with_callback(self)
    }

    pub fn params(self) -> ValueTypesBuilder<Self> {
        ValueTypesBuilder::with_callback(self)
    }

    pub fn return_type(self) -> OptionalValueTypeBuilder<Self> {
        OptionalValueTypeBuilder::with_callback(self)
    }

    pub fn build(self) -> F::Result {
        self.callback.invoke(self.signature)
    }

    pub fn build_sig(self) -> Signature {
        Signature::Inline(self.signature)
    }
}

impl<F> Invoke<Vec<elements::ValueType>> for SignatureBuilder<F>
    where F: Invoke<elements::FunctionType>
{
    type Result = Self;

    fn invoke(self, args: Vec<elements::ValueType>) -> Self {
        self.with_params(args)
    }
}

impl<F> Invoke<Option<elements::ValueType>> for SignatureBuilder<F>
    where F: Invoke<elements::FunctionType>
{
    type Result = Self;

    fn invoke(self, arg: Option<elements::ValueType>) -> Self {
        self.with_return_type(arg)
    }
}

impl<F> Invoke<elements::ValueType> for SignatureBuilder<F> 
    where F: Invoke<elements::FunctionType>  
{
    type Result = Self;

    fn invoke(self, arg: elements::ValueType) -> Self {
        self.with_param(arg)
    }
}

pub struct TypeRefBuilder<F=Identity> {
    callback: F,
    type_ref: u32,
}

impl<F> TypeRefBuilder<F> where F: Invoke<u32> {
    pub fn with_callback(callback: F) -> Self {
        TypeRefBuilder { 
            callback: callback, 
            type_ref: 0
        }
    }

    pub fn val(mut self, val: u32) -> Self {
        self.type_ref = val;
        self
    }

    pub fn build(self) -> F::Result { self.callback.invoke(self.type_ref) }
}

pub struct SignaturesBuilder<F=Identity> {
    callback: F,
    section: Vec<Signature>,
}

impl SignaturesBuilder {
    /// New empty functions section builder
    pub fn new() -> Self {
        SignaturesBuilder::with_callback(Identity)
    }
}

impl<F> SignaturesBuilder<F> {
    pub fn with_callback(callback: F) -> Self {
        SignaturesBuilder {
            callback: callback,
            section: Vec::new(),
        }
    }

    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.section.push(signature);
        self
    }

    pub fn type_ref(self) -> TypeRefBuilder<Self> {
        TypeRefBuilder::with_callback(self)
    }    
}

impl<F> SignaturesBuilder<F> where F: Invoke<SignatureBindings> {
    pub fn signature(self) -> SignatureBuilder<Self> {
        SignatureBuilder::with_callback(self)
    }
}

impl<F> Invoke<elements::FunctionType> for SignaturesBuilder<F> {
	type Result = Self;

	fn invoke(self, signature: elements::FunctionType) -> Self {
		self.with_signature(Signature::Inline(signature))
    }    
}

impl<F> Invoke<u32> for SignaturesBuilder<F> {
	type Result = Self;

	fn invoke(self, type_ref: u32) -> Self {
		self.with_signature(Signature::TypeReference(type_ref))
    }    
}

impl<F> SignaturesBuilder<F> where F: Invoke<elements::FunctionsSection> {
    pub fn build(self) -> F::Result {
        let mut result = elements::FunctionsSection::new();
        for f in self.section.into_iter() {
            if let Signature::TypeReference(type_ref) = f {
                result.entries_mut().push(elements::Func::new(type_ref));
            } else {
                unreachable!(); // never possible with current generics impl-s
            }
        }
        self.callback.invoke(result)
    }
}

pub type SignatureBindings = Vec<Signature>;

impl<F> SignaturesBuilder<F> where F: Invoke<SignatureBindings> {
    pub fn bind(self) -> F::Result {
        self.callback.invoke(self.section)
    }
}

pub struct FuncBodyBuilder<F=Identity> {
    callback: F,
    body: elements::FuncBody,
}

impl<F> FuncBodyBuilder<F> {
    pub fn with_callback(callback: F) -> Self {
        FuncBodyBuilder {
            callback: callback,
            body: elements::FuncBody::new(Vec::new(), elements::Opcodes::empty()),
        }
    }
}

impl<F> FuncBodyBuilder<F> where F: Invoke<elements::FuncBody> {
    pub fn with_func(mut self, func: elements::FuncBody) -> Self {
        self.body = func;
        self
    }

    pub fn with_locals(mut self, locals: Vec<elements::Local>) -> Self {
        self.body.locals_mut().extend(locals);
        self
    }

    pub fn with_opcodes(mut self, opcodes: elements::Opcodes) -> Self {
        *self.body.code_mut() = opcodes;
        self
    }

    pub fn build(self) -> F::Result {
        self.callback.invoke(self.body)
    }
}

pub struct FunctionDefinition {
    pub is_main: bool,
    pub signature: Signature,
    pub code: elements::FuncBody,
}

impl Default for FunctionDefinition {
    fn default() -> Self {
        FunctionDefinition {
            is_main: false,
            signature: Signature::TypeReference(0),
            code: elements::FuncBody::empty(),
        }
    }
}

pub struct FunctionBuilder<F=Identity> {
    callback: F,
    func: FunctionDefinition,
}

impl FunctionBuilder {
    pub fn new() -> Self {
        FunctionBuilder::with_callback(Identity)
    }
}

impl<F> FunctionBuilder<F> where F: Invoke<FunctionDefinition> {
    pub fn with_callback(callback: F) -> Self {
        FunctionBuilder {
            callback: callback,
            func: Default::default(),
        }
    }

    pub fn main(mut self) -> Self {
        self.func.is_main = true;
        self
    }

    pub fn signature(self) -> SignatureBuilder<Self> {
        SignatureBuilder::with_callback(self)
    }

    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.func.signature = signature;
        self
    }

    pub fn body(self) -> FuncBodyBuilder<Self> {
        FuncBodyBuilder::with_callback(self)
    }

    pub fn with_body(mut self, body: elements::FuncBody) -> Self {
        self.func.code = body;
        self
    }

    pub fn build(self) -> F::Result {
        self.callback.invoke(self.func)
    }
}

impl<F> Invoke<elements::FunctionType> for FunctionBuilder<F> where F: Invoke<FunctionDefinition> {
	type Result = Self;

	fn invoke(self, signature: elements::FunctionType) -> Self {
		self.with_signature(Signature::Inline(signature))
    }    
}

impl<F> Invoke<u32> for FunctionBuilder<F> where F: Invoke<FunctionDefinition> {
	type Result = Self;

	fn invoke(self, type_ref: u32) -> Self {
		self.with_signature(Signature::TypeReference(type_ref))
    }    
}

impl<F> Invoke<elements::FuncBody> for FunctionBuilder<F> where F: Invoke<FunctionDefinition> {
    type Result = Self;

    fn invoke(self, body: elements::FuncBody) -> Self::Result {
        self.with_body(body)
    }
}

/// New builder of signature list
pub fn signatures() -> SignaturesBuilder {
    SignaturesBuilder::new()
}

/// New signature builder
pub fn signature() -> SignatureBuilder {
    SignatureBuilder::new()
}

/// New builder of function (signature & body)
pub fn function() -> FunctionBuilder {
    FunctionBuilder::new()
}

#[cfg(test)]
mod tests {

    use super::{signatures, function};
    use elements;

    #[test]
    fn example() {
        let result = signatures()
            .type_ref().val(1).build()
            .build();

        assert_eq!(result.entries().len(), 1);

        let result = signatures()
            .signature()
                .param().i32()
                .param().i32()
                .return_type().i64()
                .build()
            .bind();      

        assert_eq!(result.len(), 1);              
    }

    #[test]
    fn func_example() {
        let func = function()
            .signature()
                .param().i32()
                .return_type().i32()
                .build()
            .body()
                .with_opcodes(elements::Opcodes::empty())
                .build()
            .build();

        assert_eq!(func.code.locals().len(), 0);
        assert_eq!(func.code.code().elements().len(), 1);
    }
}