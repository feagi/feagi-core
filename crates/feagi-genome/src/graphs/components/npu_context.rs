/// What data is used to communicate / translate with the NPU?
pub trait GraphNPUContext {
    type CorticalAreaContext;
}

/// No NPU context, as this graph does not interact with the NPU
pub struct GraphNPUContextNone;

impl GraphNPUContext for GraphNPUContextNone {
    type CorticalAreaContext = ();
}

// TODO NPU context that has types for things like neuron, mapping index stuff