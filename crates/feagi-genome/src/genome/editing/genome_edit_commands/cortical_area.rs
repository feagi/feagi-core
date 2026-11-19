use serde::{Deserialize, Serialize};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_burst::burst_basis::connectome_editing::ConnectomeEditEnum;
use feagi_connectome::connectome_burst::burst_basis::connectome_reshaping::{ConnectomeReshapeStepCorticalAreaAdd, ConnectomeReshapeStepEnum};
use feagi_connectome::connectome_models::neuron_models::models::{NMModeled, NMNeurogenesisStorageOrGenerator};
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::CorticalTypedSpawningValidated;
use crate::genome::editing::genome_edit_commands::GenomeEditCommandStep;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

/// The enum edit step for cortical areas for the load genome sequence type. Can only add cortical areas
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct GenomeEditCommandStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_region: BrainRegionID,
    pub spawner: CorticalTypedSpawningValidated<FIQ>
}

impl<FIQ: FeagiIndexQuantization> GenomeEditCommandStep<FIQ> for GenomeEditCommandStepCorticalAreaAdd<FIQ> {
    fn generate_connectome_edit_step(&self) -> ConnectomeEditEnum<FIQ> {

        // Strip the cortical type layers down to the per type neurogenesis enum, then use the
        // existing From impls (foundation.rs / processing.rs) to widen it into NMModeled
        let neurogenesis: NMModeled<NMNeurogenesisStorageOrGenerator<FIQ>> = match self.neurogenesis.clone() {
            CorticalTypedSpawningParameters::Foundation(a) => match a {
                CTFoundationSubtypedSpawningParameters::Power(CorticalTypeAreaSpawningParameters { neurogenesis, .. }) => {
                    neurogenesis.into()
                }
            },
            CorticalTypedSpawningParameters::Processing(a) => match a {
                CTProcessingSubtypedSpawningParameters::Interconnect(CorticalTypeAreaSpawningParameters { neurogenesis, .. }) => {
                    neurogenesis.into()
                }
            },
        };

        ConnectomeEditEnum::EditConnectomeStep(
            ConnectomeReshapeStepEnum::CorticalAreaAdd(
                ConnectomeReshapeStepCorticalAreaAdd {
                    cortical_area_neurogenesis: neurogenesis }
            )
        )
    }
}
