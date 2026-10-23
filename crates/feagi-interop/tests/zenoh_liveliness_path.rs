//! Writing and parsing the liveliness token path of a node.

use feagi_basis::feagi_message::feagi_message_description::FeagiMessagePathElementDescription;
use feagi_basis::feagi_message::feagi_message_path::FeagiMessagePath;
use feagi_interop::node::feagi_node_flags::FeagiNodeFlags;
use feagi_interop::node::feagi_node_id::FeagiNodeID;
use feagi_interop::node::feagi_role::{FeagiRole, FeagiRoles};
use feagi_interop::transport::zenoh::zenoh_liveliness_path::{
    FeagiNodeLivelinessPath, FeagiNodeLivelinessPathConfigurables, LIVELINESS_KEY_LEN, LIVELINESS_WILDCARD_KEY_LEN,
};

fn node_id() -> FeagiNodeID {
    FeagiNodeID::from_bytes_const([0, 1, 2, 3, 4, 5, 6, 7])
}

fn other_node_id() -> FeagiNodeID {
    FeagiNodeID::from_bytes_const([7, 6, 5, 4, 3, 2, 1, 0])
}

fn flags() -> FeagiNodeFlags {
    let mut roles = FeagiRoles::new();
    roles.set_role(FeagiRole::FeagiBrainEmbodimentEndpoint, true);
    roles.set_role(FeagiRole::FeagiViewerEditor, true);
    let mut flags = FeagiNodeFlags::new();
    flags.set_roles(roles);
    flags
}

#[test]
fn key_is_root_node_id_and_flags() {
    let configurables = FeagiNodeLivelinessPathConfigurables::new(node_id());
    let key = FeagiNodeLivelinessPath::key(&configurables, flags());
    let expected = format!("feagi/{}/{}", node_id(), flags());
    assert_eq!(key.as_str(), expected);
    assert_eq!(key.len(), LIVELINESS_KEY_LEN);
}

#[test]
fn key_parses_back_into_its_flags() {
    let configurables = FeagiNodeLivelinessPathConfigurables::new(node_id());
    let key = FeagiNodeLivelinessPath::key(&configurables, flags());
    assert_eq!(FeagiNodeLivelinessPath::parse_key(&configurables, &key), Some(flags()));
}

#[test]
fn parse_rejects_the_key_of_another_node() {
    let configurables = FeagiNodeLivelinessPathConfigurables::new(node_id());
    let other = FeagiNodeLivelinessPathConfigurables::new(other_node_id());
    let key = FeagiNodeLivelinessPath::key(&other, flags());
    assert_eq!(FeagiNodeLivelinessPath::parse_key(&configurables, &key), None);
}

#[test]
fn parse_rejects_malformed_keys() {
    let configurables = FeagiNodeLivelinessPathConfigurables::new(node_id());
    let id = node_id().to_string();
    let flags = flags().to_string();
    for key in [
        format!("other/{id}/{flags}"),
        format!("feagi/{id}"),
        format!("feagi/{id}/{flags}/extra"),
        format!("feagi/{id}/not-flags"),
        format!("/feagi/{id}/{flags}"),
    ] {
        assert_eq!(FeagiNodeLivelinessPath::parse_key(&configurables, &key), None, "{key}");
    }
}

#[test]
fn wildcard_key_replaces_the_flags() {
    let configurables = FeagiNodeLivelinessPathConfigurables::new(node_id());
    let key = FeagiNodeLivelinessPath::wildcard_key(&configurables);
    assert_eq!(key.as_str(), format!("feagi/{}/*", node_id()));
    assert_eq!(key.len(), LIVELINESS_WILDCARD_KEY_LEN);
}

#[test]
fn path_elements_name_the_same_slots_as_the_full_path() {
    let (root, node_id, flags) = FeagiNodeLivelinessPath::FULL_MESSAGE_PATH;
    match FeagiNodeLivelinessPath::PATH_ELEMENTS {
        [
            FeagiMessagePathElementDescription::Static(static_root),
            FeagiMessagePathElementDescription::Configurable { name: configurable },
            FeagiMessagePathElementDescription::Parameter { name: parameter, .. },
        ] => {
            assert_eq!(*static_root, root);
            assert_eq!(*configurable, node_id.name);
            assert_eq!(*parameter, flags.name);
        }
        elements => panic!("unexpected path elements {elements:?}"),
    }
}
