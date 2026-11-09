use feagi_basis::feagi_basis_proc_common::feagi_id;

feagi_id! {
    /// Specifies a unique synapse mapping rule that describes the pattern of how synapses are
    /// connected between cortical areas in a mapping entry
    pub ConnectivityRuleID: 8,
    random,
    base64 = Base64EncodedConnectivityRuleID,
}