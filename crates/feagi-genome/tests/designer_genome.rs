#![cfg(feature = "std")]

use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_genome::basis::brain_region::BrainRegionID;
use feagi_genome::genome::genome_object::GenomeObjectMetadata;
use feagi_genome::genome::genomes::designer::DesignerGenome;

#[test]
fn barebones_designer_genome_builds() {
    let genome = DesignerGenome::<FeagiIndexQuantizationStandard>::new_barebones_genome_now(GenomeObjectMetadata {
        name: "test".to_string(),
        description: String::new(),
    });
    // Only the root region exists, so a new ID names no region
    assert!(genome.get_brain_region(&BrainRegionID::new_now()).is_err());
}
