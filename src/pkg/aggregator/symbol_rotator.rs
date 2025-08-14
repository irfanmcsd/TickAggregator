pub struct SymbolRotator {
    all_symbols: Vec<String>,
    batch_size: usize,
    current_idx: usize,
}

impl SymbolRotator {
    pub fn new(symbols: Vec<String>, batch_size: usize) -> Self {
        Self {
            all_symbols: symbols,
            batch_size,
            current_idx: 0,
        }
    }

    pub fn next_batch(&mut self) -> Option<&[String]> {
        let n = self.all_symbols.len();
        if n == 0 || self.batch_size == 0 {
            return None;
        }

        let start = self.current_idx;
        let mut end = start + self.batch_size;

        if end > n {
            end = n;
        }

        let batch = &self.all_symbols[start..end];

        self.current_idx = end % n;

        Some(batch)
    }
}
