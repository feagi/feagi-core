//! The path of the liveliness token every node declares while it is on the network. Written by
//! hand rather than by macro, as it is one universal path and not part of a request category
//!
//! The path is `feagi/$node_id/:flags`. The node ID is a configurable as every use already knows
//! which node it is about (announcing itself, checking an ID is free, reading a node's roles),
//! while the flags are the parameter read back out of the key

use crate::node::feagi_node_flags::{FeagiNodeFlags, NUMBER_OF_BASE_64_BYTES_NODE_FLAGS};
use crate::node::feagi_node_id::{Base64EncodedFeagiNodeID, FeagiNodeID, NUMBER_OF_BASE_64_BYTES_NODE_ID};
use feagi_basis::feagi_message::feagi_message_description::{FeagiFieldKind, FeagiMessagePathElementDescription};
use feagi_basis::feagi_message::feagi_message_path::configurables::{ConfigurableSlot, FeagiMessagePathConfigurables, FeagiMessagePathStaticElement};
use feagi_basis::feagi_message::feagi_message_path::parameters::{FeagiMessagePathParameterElement, FeagiMessagePathParameters, MessagePathSlotElement};
use feagi_basis::feagi_message::feagi_message_path::{write_wildcard_message_path, FeagiMessagePath};

/// First segment of every FEAGI key
pub const FEAGI_KEY_ROOT: &str = "feagi";

/// Length of a liveliness key
pub const LIVELINESS_KEY_LEN: usize = FEAGI_KEY_ROOT.len() + 1 + NUMBER_OF_BASE_64_BYTES_NODE_ID + 1 + NUMBER_OF_BASE_64_BYTES_NODE_FLAGS;

/// Length of a liveliness key with the flags replaced by `*`
pub const LIVELINESS_WILDCARD_KEY_LEN: usize = FEAGI_KEY_ROOT.len() + 1 + NUMBER_OF_BASE_64_BYTES_NODE_ID + 2;

const NODE_ID_CONFIGURABLE: &str = "node_id";
const FLAGS_PARAMETER: &str = "flags";

//region Configurables and Parameters

/// Which node a liveliness path is about
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeagiNodeLivelinessPathConfigurables {
    pub node_id: Base64EncodedFeagiNodeID,
}

impl FeagiNodeLivelinessPathConfigurables {
    pub fn new(node_id: FeagiNodeID) -> Self {
        Self { node_id: node_id.into() }
    }
}

impl FeagiMessagePathConfigurables for FeagiNodeLivelinessPathConfigurables {
    fn configurable_str(&self, name: &str) -> Option<&str> {
        match name {
            NODE_ID_CONFIGURABLE => Some(self.node_id.to_str_slice()),
            _ => None,
        }
    }
}

/// What a node's liveliness key says about it
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeagiNodeLivelinessPathParameters {
    pub flags: FeagiNodeFlags,
}

impl FeagiMessagePathParameters for FeagiNodeLivelinessPathParameters {}

//endregion

//region Path

/// The liveliness token path of a node, `feagi/$node_id/:flags`
#[derive(Debug, Clone, Copy)]
pub struct FeagiNodeLivelinessPath;

impl FeagiNodeLivelinessPath {
    /// The key the node declares its liveliness token on
    pub fn key(configurables: &FeagiNodeLivelinessPathConfigurables, flags: FeagiNodeFlags) -> heapless::String<LIVELINESS_KEY_LEN> {
        let mut key = heapless::String::new();
        Self::write_message_path(configurables, &FeagiNodeLivelinessPathParameters { flags }, &mut key)
            .expect("liveliness keys are LIVELINESS_KEY_LEN long");
        key
    }

    /// The key matching the node's liveliness token whatever its flags. Querying it tells whether
    /// the node is on the network, and with which flags
    pub fn wildcard_key(configurables: &FeagiNodeLivelinessPathConfigurables) -> heapless::String<LIVELINESS_WILDCARD_KEY_LEN> {
        let mut key = heapless::String::new();
        write_wildcard_message_path::<Self, _>(configurables, &mut key).expect("liveliness wildcard keys are LIVELINESS_WILDCARD_KEY_LEN long");
        key
    }

    /// The flags in `key`, or `None` if `key` is not the liveliness key of the node
    pub fn parse_key(configurables: &FeagiNodeLivelinessPathConfigurables, key: &str) -> Option<FeagiNodeFlags> {
        Self::parse_message_path(configurables, key.split('/')).map(|parameters| parameters.flags)
    }
}

impl FeagiMessagePath for FeagiNodeLivelinessPath {
    type Configurables = FeagiNodeLivelinessPathConfigurables;
    type Parameters = FeagiNodeLivelinessPathParameters;
    type FullMessagePathBaseTuple = (&'static str, ConfigurableSlot, MessagePathSlotElement<FeagiNodeFlags>);

    const FULL_MESSAGE_PATH: Self::FullMessagePathBaseTuple = (
        FEAGI_KEY_ROOT,
        ConfigurableSlot::new(NODE_ID_CONFIGURABLE),
        MessagePathSlotElement::new(FLAGS_PARAMETER),
    );

    const PATH_ELEMENTS: &'static [FeagiMessagePathElementDescription] = &[
        FeagiMessagePathElementDescription::Static(FEAGI_KEY_ROOT),
        FeagiMessagePathElementDescription::Configurable { name: NODE_ID_CONFIGURABLE },
        FeagiMessagePathElementDescription::Parameter {
            name: FLAGS_PARAMETER,
            description: Some("Version, roles, and state of the node, as padded URL-safe base64"),
            kind: FeagiFieldKind::String,
        },
    ];

    type SlottedParameterMessagePath = (&'static str, &'static str, MessagePathSlotElement<FeagiNodeFlags>);

    fn to_slotted_parameter_message_path(configurables: &'static Self::Configurables) -> Self::SlottedParameterMessagePath {
        let full = Self::FULL_MESSAGE_PATH;
        (full.0, configurables.node_id.to_str_slice(), full.2)
    }

    fn write_message_path<W: core::fmt::Write>(configurables: &Self::Configurables, parameters: &Self::Parameters, out: &mut W) -> core::fmt::Result {
        out.write_str(FEAGI_KEY_ROOT)?;
        out.write_str("/")?;
        out.write_str(configurables.node_id.to_str_slice())?;
        out.write_str("/")?;
        parameters.flags.write_path_segment(out)
    }

    fn parse_message_path<'a, I: Iterator<Item = &'a str>>(configurables: &Self::Configurables, mut segments: I) -> Option<Self::Parameters> {
        if segments.next()? != FEAGI_KEY_ROOT {
            return None;
        }
        if segments.next()? != configurables.node_id.to_str_slice() {
            return None;
        }
        let flags: FeagiNodeFlags = core::str::FromStr::from_str(segments.next()?).ok()?;
        if segments.next().is_some() {
            return None;
        }
        Some(FeagiNodeLivelinessPathParameters { flags })
    }
}

//endregion
