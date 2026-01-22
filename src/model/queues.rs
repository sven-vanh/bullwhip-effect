// src/model/queues.rs

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct TimeDelayQueue {
    buffer: VecDeque<u32>,
    delay_length: usize,
}

impl TimeDelayQueue {
    pub fn new(delay: usize) -> Self {
        let mut buffer = VecDeque::with_capacity(delay);
        // Pre-fill with 0s so items take time to traverse the pipe
        for _ in 0..delay {
            buffer.push_back(0);
        }

        Self {
            buffer,
            delay_length: delay,
        }
    }

    /// Step 1: Items arrive at the destination.
    /// Call this at the START of the turn.
    pub fn pop_arrival(&mut self) -> u32 {
        self.buffer.pop_front().unwrap_or(0)
    }

    /// Step 2: Items enter the pipeline.
    /// Call this at the END of the turn.
    pub fn push_departure(&mut self, item: u32) {
        self.buffer.push_back(item);
    }

    // Helper to see what is inside (for debugging)
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
