#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stack {
    pub stack: Vec<Vec<u8>>,
}

impl Stack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, item: Vec<u8>) {
        self.stack.push(item);
    }

    pub fn pop(&mut self) -> Result<Vec<u8>, &'static str> {
        self.stack.pop().ok_or("there is an overflow of the stack")
    }

    pub fn top(&self) -> Result<&[u8], &'static str> {
        self.stack
            .last()
            .map(|v| v.as_slice())
            .ok_or("there is an overflow of the stack")
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn last_item(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
