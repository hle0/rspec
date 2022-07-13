pub struct Ring<T> where T: Clone + Default {
    data: Vec<Option<T>>,
    tail: usize,
    head: usize,
}

impl<T> Ring<T> where T: Clone + Default {
    /// Create a new, empty buffer of the given size.
    pub fn new(size: usize) -> Self {
        Self { data: vec![None; size], tail: 0, head: 0 }
    }

    /// Return the number of available elements to take.
    pub fn len(&self) -> usize {
        (self.data.len() + self.head - self.tail) % self.data.len()
    }

    /// Take an element from the tail, if possible.
    pub fn recv(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        } else {
            let v = std::mem::take(&mut self.data[self.tail]);
            self.tail = (self.tail + 1) % self.data.len();
            return v;
        }
    }

    /// Drop a single element.
    /// Returns true if an element was dropped.
    pub fn drop_one(&mut self) -> bool {
        match self.recv() {
            Some(_) => true,
            None => false
        }
    }

    /// Add an element to the buffer, and advance the head.
    /// Returns true if an element from the tail was dropped to make room.
    pub fn send(&mut self, v: T) -> bool {
        self.data[self.head] = Some(v);
        self.head = self.head + 1;

        if self.len() == 0 {
            // We've looped all the way around!
            // Our head has caught up to our tail, and we've used the whole buffer.
            // We have to drop an element, otherwise the length becomes zero.
            self.tail = (self.tail + 1) % self.data.len();
            true
        } else {
            false
        }
    }

    /// Take many elements at once. Might be faster than recv().
    pub fn recv_many(&mut self, requested: usize) -> Vec<T> {
        if requested == 0 {
            return Vec::new();
        }

        if requested > self.len() {
            self.recv_many(self.len())
        } else {
            if self.tail + requested > self.data.len() {
                // we'll have to loop around

                // the easy elements
                let mut res = self.recv_many(self.data.len() - self.tail);

                // the hard elements
                res.append(&mut self.recv_many(requested - res.len()));

                res
            } else {
                let mut res = Vec::new();

                for i in 0..requested {
                    res.push(std::mem::take(&mut self.data[self.tail + i]).unwrap());
                }

                self.tail = (self.tail + requested) % self.data.len();

                res
            }
        }
    }

    /// Drop many elements at once. May be faster than drop_one().
    pub fn drop_many(&mut self, requested: usize) -> usize {
        if requested == 0 {
            return 0;
        }

        if requested > self.len() {
            self.drop_many(self.len())
        } else {
            if self.tail + requested > self.data.len() {
                // we'll have to loop around

                // the easy elements
                let dropped = self.drop_many(self.data.len() - self.tail);

                // the hard elements
                dropped + self.drop_many(requested - dropped)
            } else {
                for i in 0..requested {
                    self.data[self.tail + i] = None
                }

                self.tail = (self.tail + requested) % self.data.len();

                requested
            }
        }
    }

    /// Send many elements at once. May be faster than send().
    pub fn send_many(&mut self, vs: &mut Vec<T>) -> usize {
        if vs.len() == 0 {
            return 0;
        }

        if vs.len() + self.len() >= self.data.len() {
            self.drop_many(vs.len() + self.len() - self.data.len() + 1) + self.send_many(vs)
        } else {
            if self.head + vs.len() > self.data.len() {
                // we'll have to loop around

                // hard part
                let mut far = vs.split_off(self.data.len() - self.head);
                // easy part
                let near = vs;

                self.send_many(near) + self.send_many(&mut far)
            } else {
                for i in 0..vs.len() {
                    self.data[self.head + i] = Some(std::mem::take(&mut vs[i]));
                }

                self.head = (self.head + vs.len()) % self.data.len();

                vs.len()
            }
        }
    }

}

