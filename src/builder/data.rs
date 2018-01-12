use super::invoke::{Identity, Invoke};
use elements;

pub struct DataSegmentBuilder<F=Identity> {
    callback: F,
    // todo: add mapper once multiple memory refs possible
    mem_index: u32,
    offset: elements::InitExpr,
    value: Vec<u8>,
}

impl DataSegmentBuilder {
    pub fn new() -> Self {
        DataSegmentBuilder::with_callback(Identity)
    }
}

impl<F> DataSegmentBuilder<F> {
    pub fn with_callback(callback: F) -> Self {
        DataSegmentBuilder {
            callback: callback,
            mem_index: 0,
            offset: elements::InitExpr::empty(),
            value: Vec::new(),
        }
    }

    pub fn offset(mut self, opcode: elements::Opcode) -> Self {
        self.offset = elements::InitExpr::new(vec![opcode, elements::Opcode::End]);
        self
    }

    pub fn value(mut self, value: Vec<u8>) -> Self {
        self.value = value;
        self
    }
}

impl<F> DataSegmentBuilder<F> where F: Invoke<elements::DataSegment> {
    pub fn build(self) -> F::Result {
        self.callback.invoke(
            elements::DataSegment::new(
                self.mem_index,
                self.offset,
                self.value,
            )
        )
    }
}