use unicode_segmentation::UnicodeSegmentation;
pub struct LineBuffer {
    buffer: String,
    insertion_point: usize,
}

impl LineBuffer {
    pub fn new() -> LineBuffer {
        LineBuffer {
            buffer: String::new(),
            insertion_point: 0,
        }
    }

    pub fn set_insertion_pos(&mut self, pos: usize) {
        self.insertion_point = pos;
    }

    pub fn get_insertion_pos(&self) -> usize {
        self.insertion_point
    }

    pub fn get_buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear()
    }

    pub fn get_grapheme_indices(&self) -> Vec<(usize, &str)> {
        UnicodeSegmentation::grapheme_indices(self.buffer.as_str(), true).collect()
    }

    pub fn inc_insertion_pos(&mut self) {
        let grapheme_indices = self.get_grapheme_indices();
        //eprintln!("{:?}, {}",grapheme_indices,self.get_buffer_len());
        for i in 0..grapheme_indices.len() {
            if grapheme_indices[i].0 == self.insertion_point && i < grapheme_indices.len() - 1 {
                self.insertion_point = grapheme_indices[i + 1].0;
                return;
            }
        }
        self.insertion_point = self.get_buffer_len();
    }

    pub fn dec_insertion_pos(&mut self) {
        let grapheme_indices = self.get_grapheme_indices();

        if self.get_insertion_pos() == self.get_buffer_len() && grapheme_indices.len() > 0 {
            if let Some((index, _)) = grapheme_indices.last() {
                self.insertion_point = *index
            } else {
                self.insertion_point = 0
            }
        } else {
            for i in 0..grapheme_indices.len() {
                if grapheme_indices[i].0 == self.insertion_point && i > 1 {
                    self.insertion_point = grapheme_indices[i - 1].0;
                    return;
                }
            }
            self.insertion_point = 0;
        }
    }

    pub fn get_grapheme_idx_left(&self) -> usize {
        let grapheme_indices = self.get_grapheme_indices();
        let mut prev = 0;

        for idx in 0..grapheme_indices.len() {
            if grapheme_indices[idx].0 >= self.insertion_point {
                return prev;
            }
            prev = grapheme_indices[idx].0;
        }
        prev
    }

    pub fn get_grapheme_idx_right(&self) -> usize {
        let grapheme_indices = self.get_grapheme_indices();
        let mut next = self.get_buffer_len();

        for (idx, _) in grapheme_indices.iter().rev() {
            if *idx <= self.insertion_point {
                return next;
            }
            next = *idx;
        }
        next
    }

    pub fn insert_char(&mut self, insertion_point: usize, c: char) {
        self.buffer.insert(insertion_point, c);
    }

    pub fn slice_buffer(&mut self, insertion_point: usize) -> &str {
        &self.buffer[insertion_point..]
    }

    pub fn is_buffer_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        self.buffer.pop()
    }

    pub fn remove_char(&mut self, remove_idx: usize) -> char {
        self.buffer.remove(remove_idx)
    }

    pub fn move_word_left(&mut self) -> usize {
        match self
            .buffer
            .rmatch_indices(&[' ', '\t'][..])
            .find(|(index, _)| index < &(self.insertion_point - 1))
        {
            Some((index, _)) => {
                self.insertion_point = index + 1; // as index is index of space, so we need to incr by 1
            }
            None => {
                self.insertion_point = 0;
            }
        }
        self.insertion_point
    }

    pub fn move_word_right(&mut self) -> usize {
        match self
            .buffer
            .match_indices(&[' ', '\t'][..])
            .find(|(index, _)| index > &(self.insertion_point))
        {
            Some((index, _)) => {
                self.insertion_point = index + 1; // as index is index of space, so we need to incr by 1
            }
            None => {
                self.insertion_point = self.get_buffer_len();
            }
        }
        self.insertion_point
    }

    pub fn set_buffer(&mut self, other: String) {
        self.buffer = other;
    }

    pub fn move_to_end(&mut self) {
        self.insertion_point = self.get_buffer_len();
    }
}
