use std::collections::VecDeque;

pub struct Ring<T> where T: Clone + Default {
    pub data: VecDeque<T>,
    size: usize,
}

impl<T> Ring<T> where T: Clone + Default {
    /// Create a new, empty buffer of the given size.
    pub fn new(size: usize) -> Self {
        Self { data: VecDeque::new(), size }
    }

    /// Return the number of available elements to take.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Take an element from the tail, if possible.
    pub fn recv(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    /// Drop a single element.
    /// Returns true if an element was dropped.
    pub fn drop_one(&mut self) -> bool {
        self.data.pop_front().is_some()
    }

    /// Add an element to the buffer, and advance the head.
    /// Returns true if an element from the tail was dropped to make room.
    pub fn send(&mut self, v: T) -> bool {
        let dropped = self.len() == self.size;
        if dropped {
            self.drop_one();
        }

        self.data.push_back(v);

        dropped
    }

    /// Take many elements at once. Might be faster than recv().
    pub fn recv_many(&mut self, requested: usize) -> VecDeque<T> {
        if requested == 0 {
            Default::default()
        } else if requested > self.len() {
            self.recv_many(self.len())
        } else {
            let mut remainder = self.data.split_off(requested);

            std::mem::swap(&mut self.data, &mut remainder);
            // remainder now contains the front elements

            remainder
        }
    }

    /// Drop many elements at once. May be faster than drop_one().
    pub fn drop_many(&mut self, requested: usize) -> usize {
        if requested == 0 {
            0
        } else if requested > self.len() {
            self.drop_many(self.len())
        } else {
            self.data.rotate_left(requested);
            self.data.truncate(self.len() - requested);
            requested
        }
    }

    /// Send many elements at once. May be faster than send().
    pub fn send_many<I>(&mut self, vs: I) -> usize where I: IntoIterator<Item = T> + ExactSizeIterator {
        let dropped = if vs.len() + self.len() > self.size {
            self.drop_many(vs.len() + self.len() - self.size)
        } else { 0 };

        self.data.extend(vs);

        dropped
    }

}

