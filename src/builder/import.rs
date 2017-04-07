use super::invoke::{Invoke, Identity};
use elements;

pub struct ImportBuilder<F: Identity> {
    callback: F,
    module: String,
    field: String,
    binding: ExternalBinding,
}

impl<F> ImportBuilder<F> {

    pub fn with_callback(callback: F) -> Self {
        ImportBuilder {
            callback: callback,
            module
        }
    }

    pub fn external(self) -> ImportExternalBuilder<Self> {

    }
}

pub struct ImportExternalBuilder<F=Identity> {
    callback: F,
    binding: ExternalBinding,
}

impl<F> ImportExternalBuilder<F> where F: Invoke<ExternalBinding> {
    pub fn with_callback(callback: F) {
        ImportExternalBuilder{
            callback: callback,
            binding: ExternalBinding::ExistingFunc(0),
        }
    }

    pub fn 
}