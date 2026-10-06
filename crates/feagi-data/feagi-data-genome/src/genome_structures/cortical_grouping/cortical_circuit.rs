




pub trait CorticalGrouping<
    CorticalGroupContext,
    CircuitInternals
>
{
    type IdentifierType;
    
}



// cortical context -> root circuit, open circuit, obscured circuit, amalgamating circuit, alagamating censored circuit (use different internals)

// cortical internals -> client sparse, client in depth, client censored, server, saved geneome, 




pub struct CorticalCircuita {
    parent_circuit: Option<()>,
    containing_cortical_ids: (),
    containing_cortical_circuits: (),
    genome_positions: (),
    name: (),
    description: (),
    has_visibility_restrictions: bool,

}