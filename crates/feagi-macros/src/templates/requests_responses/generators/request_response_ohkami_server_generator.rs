use quote::quote;
use crate::basis::GeneratorFromTemplate;
use crate::templates::requests_responses::requests_responses_structs::CompleteRequestResponsesTemplate;

pub struct OhkamiServerGenerator;

impl GeneratorFromTemplate<CompleteRequestResponsesTemplate> for OhkamiServerGenerator {
    fn generate_code_from_template(template: CompleteRequestResponsesTemplate) -> proc_macro2::TokenStream {
        quote!{

        }
    }
}