use std::collections::VecDeque;
use soft_shared_lib::field_types::SequenceNumber;

/// # Data Packet Send Buffer
///
/// buffers sent Data packets until they are acknowledged, because they might require retransmission
pub struct SendBuffer {
    /// keep allocated vectors for reuse
    memory_cache: VecDeque<Vec<u8>>,
    /// packets that are already transferred but not acknowledged
    packet_queue: VecDeque<Vec<u8>>,
    /// the sequence number of the element at the front of the queue
    front_sequence_number: SequenceNumber
}

impl SendBuffer {

    pub fn new() -> Self {
        return Self {
            memory_cache: VecDeque::new(),
            packet_queue: VecDeque::new(),
            front_sequence_number: 0,
        }
    }

    pub fn add(&mut self) -> &mut Vec<u8>{
        let vec = if let Some(vec) = self.memory_cache.pop_front() {
            vec
        } else {
            Vec::new()
        };
        self.packet_queue.push_back(vec);
        self.packet_queue.get_mut(self.packet_queue.len() - 1).unwrap()
    }

    pub fn get(&mut self, sequence_number: SequenceNumber) -> Option<&mut [u8]> {
        if sequence_number < self.front_sequence_number {
            return None;
        }
        self.packet_queue.get_mut((sequence_number - self.front_sequence_number) as usize).map(|v| v.as_mut_slice())
    }

    /// drop all packets below the next_sequence_number
    pub fn drop_before(&mut self, next_sequence_number: SequenceNumber) {
        while !self.packet_queue.is_empty() && next_sequence_number > self.front_sequence_number {
            let mut vec = self.packet_queue.pop_front().unwrap();
            vec.clear();
            self.memory_cache.push_front(vec);
            self.front_sequence_number += 1;
        }
    }

    pub fn len(&self) -> u64 {
        self.packet_queue.len() as u64
    }
}