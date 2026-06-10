// TODO This is sync for now, which is pretty bad! We should move these things to seperate threads
// and allow them to run independently at their own pace, passing messages for commands
// TODO while messages are fine for command and control, how do we handle motor / sensor data?
// Just a mutex reference? Or better yes, mpsc messages?

// TODO global quantization?

// TODO Traits for multiple types of interfaces?

// TODO FEAGI Error

use crate::neural_processing_unit_data_structures::dynamic_burst_engine_interface::npu_requests::enums::{NPURequestBase, NPURequestCorticalArea};
use crate::neural_processing_unit_data_structures::dynamic_burst_engine_interface::npu_requests::npu_request::{NPURequest, NPURequestID};

pub struct DynamicNPUInterface {
    // TODO
}

impl DynamicNPUInterface {

    // TODO channel for motor out
    // TODO channel for sensor in
    // motor channels should probably just be some sort of notification, we access the data via
    // reference here or something


    // TODO channel for change request responses


    pub fn request_change(&mut self, request: NPURequest) -> Option<NPURequestID>
    {
        let request_enum = request.get_request();


    }


    fn process_request_change(&mut self, request_enum: &NPURequestBase) -> Option<NPURequestID> {

        // TODO, actually, move this into the NPU processor itself, why should we be doing this here?

        match request_enum {

            NPURequestBase::CorticalArea(cortical_area_request) => {
                match cortical_area_request {

                    NPURequestCorticalArea::CreateCustomArea {
                        dimensions,
                        voxel_density,
                        neuron_model_quantization_and_device_class
                    } =>
                        {

                        }
                }
            }





        }


        None
    }





}