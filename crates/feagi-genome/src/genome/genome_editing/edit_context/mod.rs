pub mod load_genome;
pub mod amalgamate_genome;
pub mod edit_genome;

use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;


/// Under what context are we editing (loading genome, editing, amalgamation?)
pub trait GenomeEditSequenceContext: Sized {
    // Cant put FIQ at top due to `GenomeEditStepSequenceStep`

    type BrainRegionStepEnum;
    type CorticalAreaStepEnum<FIQ: FeagiIndexQuantization>;

}