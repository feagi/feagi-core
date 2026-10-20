use crate::parallelism::feagi_channel::recycling::Recyclable;

/// Marks a type that builds feagi requester / responder pairs
pub trait FeagiRequestResponseBuilder<Request: Recyclable, Response: Recyclable> {
    /// Requesting half produced by this builder
    type Requester;
    /// Responding half produced by this builder
    type Responder;
}

//region Shared Structs

/// Contents of one request channel slot
pub struct RequestSlot<Request, ReplySender> {
    pub(crate) sequence: u32,
    pub(crate) request: Request,
    /// `None` when the slot is recycled, or once the responder has taken it
    pub(crate) reply_sender: Option<ReplySender>,
}

impl<Request: Recyclable, ReplySender> Recyclable for RequestSlot<Request, ReplySender> {
    fn new_for_recycler() -> Self {
        RequestSlot {
            sequence: 0,
            request: Request::new_for_recycler(),
            reply_sender: None,
        }
    }

    fn recycle_self(&mut self) {
        self.request.recycle_self();
        self.reply_sender = None;
    }
}

/// Whether a reply slot holds a real response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyStatus {
    Responded,
    NotResponded,
}

/// Contents of one reply channel slot
pub struct ReplySlot<Response> {
    pub(crate) sequence: u32,
    pub(crate) status: ReplyStatus,
    pub(crate) response: Response,
}

impl<Response: Recyclable> Recyclable for ReplySlot<Response> {
    fn new_for_recycler() -> Self {
        ReplySlot {
            sequence: 0,
            status: ReplyStatus::NotResponded,
            response: Response::new_for_recycler(),
        }
    }

    fn recycle_self(&mut self) {
        self.status = ReplyStatus::NotResponded;
        self.response.recycle_self();
    }
}

//endregion