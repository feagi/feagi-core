
// TODO extended traits with plastic support?

pub trait ConnectomeBaseTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{

    fn defragment_connectome(&mut self);

    fn process_burst(&mut self); // TODO pass through types of FCL, FQ as mutable references

    // TODO Memory Specific interactions

    // TODO get connectome properties

    // TODO Limits / statistics



}


pub trait ConnectomeStaticTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> // TODO const sizes
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{
    // TODO
}



pub trait ConnectomeAllocTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{

    fn free_unused_capacity(&mut self);

    //region Interneuron Cortical Areas

    fn create_interneuron_cortical_area_with_default_neurons(&mut self);

    fn create_interneuron_cortical_area_with_spanned_neurons(&mut self);

    fn resize_interneuron_cortical_area_with_default_neurons(&mut self);

    // TODO discuss this in regards to restablishing the synapses via morphology
    fn resize_interneuron_cortical_area(&mut self);

    fn delete_interneuron_cortical_area(&mut self);

    //endregion


}