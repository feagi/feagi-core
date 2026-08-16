/// Makes it easy to have 2 directional buffers that way big data blocks can be passed back and forth
/// without having to reallocate (generate data to a big array, pass it in, read it from the other side / thread, then return it)
pub struct BiDirectionalChannelQueue<Data: Send, const A_TO_B_BUFFER_SIZE: usize, const B_TO_A_BUFFER_SIZE: usize> {
    a: heapless::spsc::Queue<Data, A_TO_B_BUFFER_SIZE>,
    b: heapless::spsc::Queue<Data, B_TO_A_BUFFER_SIZE>,
}

impl<Data: Send, const A_TO_B_BUFFER_SIZE: usize, const B_TO_A_BUFFER_SIZE: usize>
BiDirectionalChannelQueue<Data, A_TO_B_BUFFER_SIZE, B_TO_A_BUFFER_SIZE>
{
    pub fn new() -> Self {
        let a: heapless::spsc::Queue<Data, A_TO_B_BUFFER_SIZE> = heapless::spsc::Queue::new();
        let b: heapless::spsc::Queue<Data, B_TO_A_BUFFER_SIZE> = heapless::spsc::Queue::new();

        Self { a, b }
    }

    pub fn split(&mut self) -> (BidirectionalChannelSide<Data>, BidirectionalChannelSide<Data>) {
        let (a_p, a_c) = self.a.split();
        let (b_p, b_c) = self.b.split();

        let side_a = BidirectionalChannelSide { sender: a_p, receiver: a_c };
        let side_b = BidirectionalChannelSide { sender: b_p, receiver: b_c };

        (side_a, side_b)
    }
}

// TODO make a dyn version, heapless has something for it...


/// One side of a `BiDirectionChannelQueue`. Can be used to recieve data, do something with it, and then
/// return the data (all by ownership) to avoid memory allocations
pub struct BidirectionalChannelSide<'a, Data: Send> {
    sender: heapless::spsc::Producer<'a, Data>,
    receiver: heapless::spsc::Consumer<'a, Data>,
}

impl<'a, Data: Send> BidirectionalChannelSide<'a, Data> {
    // I totally did not copy and paste these doc strings from the heapless doc page

    /// Adds an Data to the end of the queue, returns back the Data if the queue is full.
    pub fn enqueue(&mut self, item: Data) -> Result<(), Data> {
        self.sender.enqueue(item)
    }

    /// Returns all items from the incoming queue to the outside
    pub fn completely_return_queue(&mut self) -> Result<(), ()> // TODO how should we handle multiple error types?
    {
        todo!()
        /*
        while let Some(item) = self.receiver.dequeue() {
            self.sender.enqueue(item).unwrap();
        }
        Ok(())

         */
    }

    /// Returns the item in the front of the queue, or None if the queue is empty.
    pub fn dequeue(&mut self) -> Option<Data> {
        self.receiver.dequeue()
    }

    /// Returns if there is any space to enqueue a new item. When this returns true, at least the first subsequent enqueue will succeed.
    pub fn sender_ready(&self) -> bool {
        self.sender.ready()
    }

    /// Returns if there are any items to dequeue. When this returns true, at least the first subsequent dequeue will succeed.
    pub fn receiver_ready(&self) -> bool {
        self.receiver.ready()
    }



    // TODO there are other methods to mirror
}

/// std::sync::mpsc-backed bidirectional queue with owned endpoints.
pub struct MpscBiDirectionalChannelQueue<Data: Send> {
    a_to_b_sender: std::sync::mpsc::SyncSender<Data>,
    a_to_b_receiver: std::sync::mpsc::Receiver<Data>,
    b_to_a_sender: std::sync::mpsc::SyncSender<Data>,
    b_to_a_receiver: std::sync::mpsc::Receiver<Data>,
}

impl<Data: Send> MpscBiDirectionalChannelQueue<Data> {
    pub fn new(a_to_b_buffer_size: usize, b_to_a_buffer_size: usize) -> Self {
        let (a_to_b_sender, a_to_b_receiver) = std::sync::mpsc::sync_channel(a_to_b_buffer_size);
        let (b_to_a_sender, b_to_a_receiver) = std::sync::mpsc::sync_channel(b_to_a_buffer_size);

        Self {
            a_to_b_sender,
            a_to_b_receiver,
            b_to_a_sender,
            b_to_a_receiver,
        }
    }

    pub fn split(self) -> (MpscBidirectionalChannelSide<Data>, MpscBidirectionalChannelSide<Data>) {
        let side_a = MpscBidirectionalChannelSide {
            sender: self.a_to_b_sender,
            receiver: self.b_to_a_receiver,
        };
        let side_b = MpscBidirectionalChannelSide {
            sender: self.b_to_a_sender,
            receiver: self.a_to_b_receiver,
        };

        (side_a, side_b)
    }
}

pub struct MpscBidirectionalChannelSide<Data: Send> {
    sender: std::sync::mpsc::SyncSender<Data>,
    receiver: std::sync::mpsc::Receiver<Data>,
}

impl<Data: Send> MpscBidirectionalChannelSide<Data> {
    pub fn enqueue(
        &self,
        item: Data,
    ) -> Result<(), std::sync::mpsc::TrySendError<Data>> {
        self.sender.try_send(item)
    }

    pub fn dequeue(&self) -> Option<Data> {
        self.receiver.try_recv().ok()
    }
}
