use quote::quote;
use crate::basis::GeneratorFromTemplate;
use crate::templates::requests_responses::requests_responses_structs::TemplateRequestCategory;

pub struct OhkamiServerGenerator;

impl GeneratorFromTemplate<TemplateRequestCategory> for OhkamiServerGenerator {
    fn generate_code_from_template(template: TemplateRequestCategory) -> proc_macro2::TokenStream {
        quote!{

        }
    }
}