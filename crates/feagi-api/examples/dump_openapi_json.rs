// Dump OpenAPI JSON to verify tags
use feagi_api::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let spec = ApiDoc::openapi();
    let json = spec.to_pretty_json().unwrap();
    
    // Check if tags section exists
    if json.contains("\"tags\"") {
        println!("✓ Tags section found in JSON");
    } else {
        println!("❌ Tags section NOT found in JSON");
    }
    
    // Print tags section
    if let Some(tags_start) = json.find("\"tags\"") {
        let tags_section = &json[tags_start..tags_start.min(json.len()).min(tags_start + 500)];
        println!("\nTags section preview:\n{}", tags_section);
    }
    
    // Count tag definitions
    let tag_count = json.matches("\"name\"").count();
    println!("\nTotal 'name' fields (approximate tag count): {}", tag_count);
    
    // Write full JSON to file for inspection
    std::fs::write("/tmp/openapi_dump.json", &json).unwrap();
    println!("\nFull OpenAPI JSON written to /tmp/openapi_dump.json");
    println!("You can inspect it to see if tags are present.");
}


