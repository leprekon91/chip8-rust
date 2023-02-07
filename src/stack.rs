pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { data: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn length(&self) -> usize {
        self.data.len()
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.last()
    }
}
