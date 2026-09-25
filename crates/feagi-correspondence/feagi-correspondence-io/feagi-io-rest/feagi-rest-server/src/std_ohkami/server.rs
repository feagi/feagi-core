use ohkami::prelude::*;
use ohkami::claw::{Json, status, Path};
use ohkami::openapi;

// TODO review https://github.com/ohkami-rs/ohkami (openapi section)
// macro should generate request struct, response struct, async function for what should be done
// Note that ohkami has its own JSON wrapper that needs to be used
// we need to make use of parsing path parameters and url parameters
// macro generated async function should parse the object (return 400 error if invalid), call a named async func at a given ReqRes handler


