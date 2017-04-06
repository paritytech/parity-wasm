use super::invoke::{Invoke, Identity};
use elements;

pub enum Signature {
    TypeReference(u32),
    Inline(elements::FunctionType),
}

pub struct SignatureBuilder<F=Identity> {
    callback: F,
    signature: Signature,
}

impl<F> SignatureBuilder<F> where F: Invoke<Signature> {
    pub fn with_callback(callback: F) -> Self {
        SignatureBuilder { 
            callback: callback, 
            signature: Signature::TypeReference(0) 
        }
    }

    pub fn type_ref(mut self, val: u32) -> Self {
        self.signature = Signature::TypeReference(val);
        self
    }

    pub fn param(mut self, value_type: elements::ValueType) -> Self {
        {
            let signature = &mut self.signature;
            if let Signature::TypeReference(_) = *signature { 
                *signature = Signature::Inline(elements::FunctionType::default())
            }

            if let Signature::Inline(ref mut func_type) = *signature {
                func_type.params_mut().push(value_type);
            }
        }

        self
    }

    pub fn return_type(mut self, value_type: elements::ValueType) -> Self {
        {
            let signature = &mut self.signature;
            if let Signature::TypeReference(_) = *signature { 
                *signature = Signature::Inline(elements::FunctionType::default())
            }

            if let Signature::Inline(ref mut func_type) = *signature {
                *func_type.return_type_mut() = Some(value_type);
            }
        }
                
        self
    }

    pub fn build(self) -> F::Result {
        self.callback.invoke(self.signature)
    }
}

pub struct FunctionsBuilder<F=Identity> {
    callback: F,
    section: Vec<Signature>,
}

impl FunctionsBuilder {
    /// New empty functions section builder
    pub fn new() -> Self {
        FunctionsBuilder::with_callback(Identity)
    }
}

impl<F> FunctionsBuilder<F> {
    pub fn with_callback(callback: F) -> Self {
        FunctionsBuilder {
            callback: callback,
            section: Vec::new(),
        }
    }

    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.section.push(signature);
        self
    }

    pub fn signature(self) -> SignatureBuilder<Self> {
        SignatureBuilder::with_callback(self)
    }
}

impl<F> Invoke<Signature> for FunctionsBuilder<F> {
	type Result = Self;

	fn invoke(self, signature: Signature) -> Self {
		self.with_signature(signature)
    }    
}

impl<F> FunctionsBuilder<F> where F: Invoke<elements::FunctionsSection> {
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

impl<F> FunctionsBuilder<F> where F: Invoke<SignatureBindings> {
    pub fn bind(self) -> F::Result {
        self.callback.invoke(self.section)
    }
}

#[cfg(test)]
mod tests {

    use super::FunctionsBuilder;

    #[test]
    fn example() {
        let result = FunctionsBuilder::new()
            .signature().type_ref(1).build()
            .build();

        assert_eq!(result.entries().len(), 1);

        let result = FunctionsBuilder::new()
            .signature().type_ref(1).build()
            .bind();      

        assert_eq!(result.len(), 1);              
    }
}