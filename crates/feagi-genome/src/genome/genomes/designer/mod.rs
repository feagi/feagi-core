pub mod container_brain_region;
pub mod container_cortical_area;

use container_brain_region::{BrainRegionContainerRuntimeEnumMut, BrainRegionContainerRuntimeEnumRef};
use std::collections::HashMap;
use feagi_basis::neurons::DimensionalCorticalAreaDimensions;
use feagi_basis::voxels::CorticalAreaVoxelDimensions;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisGenerator;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayout;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use crate::basis::brain_region::BrainRegionID;
use crate::genome::genome_object::{GenomeObjectCoordinate2D, GenomeObjectCoordinate3D, GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::designer::container_brain_region::{BrainRegionContainerInnerRuntime, BrainRegionContainerRootRuntime};
use crate::genome::genomes::GenomeError;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::foundation_power::quantizations::{NMFoundationPowerNeurogenesisGeneratorQuanted, NMFoundationPowerQuantFloat32};
use feagi_connectome::neuron_models::models::NeurogenesisByGeneratorTypeOf;
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault, NeurogenesisGeneratorUniformDefaultInitParameters};
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::basis::cortical_area::{CTGeneratorTyped, CorticalSubTypeUsesDefaultCorticalGenerator, CorticalType};
use crate::basis::cortical_area::foundation::{CTGeneratorFoundationPowerModeled, CTGeneratorFoundationSubTyped, CorticalIdentifierFoundation, CorticalTypeFoundationPower};
use crate::genome::genomes::designer::container_cortical_area::{CorticalAreaContainer};

/// A dynamic Genome that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU

pub struct DesignerGenome<FIQ: FeagiIndexQuantization> {
    root_brain_region: BrainRegionContainerRootRuntime,
    inner_brain_regions: HashMap<BrainRegionID, BrainRegionContainerInnerRuntime>,
    cortical_areas: HashMap<CorticalIdentifier, CorticalAreaContainer<FIQ>>,

    // TODO NPU

    // TODO
    // connectivity rules cache (id -> rule)
    // cortical mappings cache  (id -> cortical mapping set (mapping id -> mapping) )
}

impl<FIQ: FeagiIndexQuantization> DesignerGenome<FIQ> {

    /// Creates a new barebones genome (root brain region dated to now, and a single
    /// foundation power cortical area)
    pub fn new_barebones_genome_now(root_metadata: GenomeObjectMetadata) -> Self {

        let region_root= BrainRegionContainerRootRuntime::new(
            root_metadata, BrainRegionID::new_now()
        );

        let power_area_params: NeurogenesisGeneratorUniformDefaultInitParameters<NMFoundationPowerQuantFloat32>
            = CorticalTypeFoundationPower::uniform_default_generator_defaults::<FIQ>();

        // TODO generators by hand for now, fix trait later

        let mut cortical_areas = HashMap::new();

        let power_area_layout = NeuronLayoutVoxel::new(
            DimensionalCorticalAreaDimensions::new_dimensions_from_usize([1,1,1,1]).unwrap()
        );
        let power_area_generator: CTGeneratorTyped<FIQ> = CTGeneratorTyped::Foundation(
            CTGeneratorFoundationSubTyped::Power(
                CTGeneratorFoundationPowerModeled::FoundationPower(
                    NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(
                        NeurogenesisGeneratorUniformDefault::new(
                            power_area_params,
                            power_area_layout
                        )
                    )
                )
            )
        );
        let power_area_identifiers = CorticalIdentifier::Foundation(
            CorticalIdentifierFoundation::Power(
                [0,0,0,0,0,0]
            )
        );
        let power_area_container = CorticalAreaContainer {
            metadata: GenomeObjectMetadata {
                name: "Power".to_string(),
                description: "".to_string(),
            },
            positioning: GenomeObjectPositioning {
                position_2d: GenomeObjectCoordinate2D::new_zero(),
                position_3d: GenomeObjectCoordinate3D::new_from_array_const([-1, 0, 0]),
            },
            parent_id: region_root.root_id.clone(),
            incoming_mappings: Default::default(),
            outgoing_mappings: Default::default(),
            genome_generator: power_area_generator,
        };



        cortical_areas.insert(power_area_identifiers, power_area_container);

        // TODO more for test

        Self {
            root_brain_region: region_root,
            inner_brain_regions: Default::default(),
            cortical_areas,
        }
    }

    pub fn get_brain_region(&self, id: &BrainRegionID) -> Result<BrainRegionContainerRuntimeEnumRef<'_>, GenomeError> {
        if &self.root_brain_region.root_id == id {
            return Ok(BrainRegionContainerRuntimeEnumRef::Root(&self.root_brain_region))
        }
        if let Some(brain_region) = self.inner_brain_regions.get(id) {
            return Ok(BrainRegionContainerRuntimeEnumRef::Inner(brain_region))
        }
        Err(GenomeError::InvalidRegionID)
    }

    // Some methods, while mutables, will have no dependents and thus can be executed safely independently
    //region Safe Mutable methods


    pub fn reposition_brain_region(&mut self, region_id: &BrainRegionID, new_pos: GenomeObjectPositioning) -> Result<(), GenomeError> {
        let mut region_enum = self.get_brain_region_mut(region_id)?;
        let region = region_enum.try_unwrap_as_inner_mut()?;
        region.positioning = new_pos;
        Ok(())
    }

    //endregion


    pub fn add_new_brain_region(&mut self, parent_region_id: &BrainRegionID, metadata: GenomeObjectMetadata, positioning: GenomeObjectPositioning) -> Result<(), GenomeError> {
        let (new_region, new_id) = BrainRegionContainerInnerRuntime::new_now(parent_region_id.clone(), metadata, positioning);
        self.get_brain_region_mut(parent_region_id)?.insert_contained_brain_region_id(new_id.clone());
        self.inner_brain_regions.insert(new_id, new_region);
        Ok(())
    }



    pub(crate) fn get_brain_region_mut(&mut self, id: &BrainRegionID) -> Result<BrainRegionContainerRuntimeEnumMut<'_>, GenomeError> {
        if &self.root_brain_region.root_id == id {
            return Ok(BrainRegionContainerRuntimeEnumMut::Root(&mut self.root_brain_region))
        }
        if let Some(brain_region) = self.inner_brain_regions.get_mut(id) {
            return Ok(BrainRegionContainerRuntimeEnumMut::Inner(brain_region))
        }
        Err(GenomeError::InvalidRegionID)
    }










}
