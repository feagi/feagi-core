use core::cmp::min;
use feagi_basis::prelude::*;

create_wrapped_quantized_decimal!(
    /// When psp uniformity is enabled, we divide the membrane potential by the number of outgoing
    /// connection, for each connection. However, since storing the connection count as an int,
    /// then turning it into a float, then dividing with it is slow, here we precompute that into
    /// this struct
    pub(crate) InverseOutgoingConnectionCount
);

impl<Q: QuantizedDecimalUnwrappedTrait> InverseOutgoingConnectionCount<Q> {
    
    /// Defines a new inverse count from a number of connections uint. If 0 is supplied, it rounds 
    /// up to one
    pub fn new_from_count<FIQ: FeagiIndexQuantization>(number_connections: FIQ::SynapseIndexCountQuant) -> Self {
        // We don't want to end up dividing by 0
        let number_connections = min(FIQ::SynapseIndexCountQuant::QUANT_ONE, number_connections);
        let as_decimal: Q = Q::from_quantized_unsigned_integer(number_connections);
        Self(as_decimal.reciprocal())
    }
}