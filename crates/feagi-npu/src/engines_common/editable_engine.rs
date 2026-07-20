use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// 1: append consequences of commands (deleting / remaking connections)
//  2: sort into the following order -> delete synapses -> delete areas -> resize area? -> make areas -> (re)make connections


/// Engine with an editable connectome
pub trait EditableEngine<FIQ: FeagiIndexQuantization> {

    //region Cortical Area

    fn add_cortical_area(&mut self); // TODO

    fn remove_cortical_area(&mut self); // TODO

    fn resize_cortical_area(&mut self); // TODO

    //endregion

    fn add_connections(&mut self); // TODO

    fn remove_connections(&mut self); // TODO


    //region Dynamic Metrics

    // TODO

    //endregion
}