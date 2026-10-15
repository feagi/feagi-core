/// Have to do this because of compiler limits
pub type FeagiRequestMessageU8 = u8;

//region Request Method
/// Represents the method of the request (equivalent to HTTP Get / Put / etc)
#[repr(u8)]
#[derive(Clone)]
pub enum FeagiRequestMessageMethod {
    Read = Self::READ,     // GET
    Create = Self::CREATE, // PUT
    Edit = Self::EDIT,     // POST
    Delete = Self::DELETE, // DELETE
    Patch = Self::PATCH,   // PATCH
}

impl FeagiRequestMessageMethod {
    pub const READ: u8 = 0;
    pub const CREATE: u8 = 1;
    pub const EDIT: u8 = 2;
    pub const DELETE: u8 = 3;
    pub const PATCH: u8 = 4;

    pub const fn had_no_payload(&self) -> bool {
        match self {
            FeagiRequestMessageMethod::Read => true,
            _ => false,
        }
    }

    pub const fn from_u8(u: u8) -> Self {
        match u {
            Self::READ => Self::Read,
            Self::CREATE => Self::Create,
            Self::EDIT => Self::Edit,
            Self::DELETE => Self::Delete,
            Self::PATCH => Self::Patch,
            _ => panic!("Invalid Method!"),
        }
    }

    pub const fn to_u8(self) -> u8 {
        match self {
            FeagiRequestMessageMethod::Read => Self::READ,
            FeagiRequestMessageMethod::Create => Self::CREATE,
            FeagiRequestMessageMethod::Edit => Self::EDIT,
            FeagiRequestMessageMethod::Delete => Self::DELETE,
            FeagiRequestMessageMethod::Patch => Self::PATCH,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            FeagiRequestMessageMethod::Read => "Read",
            FeagiRequestMessageMethod::Create => "Create",
            FeagiRequestMessageMethod::Edit => "Edit",
            FeagiRequestMessageMethod::Delete => "Delete",
            FeagiRequestMessageMethod::Patch => "Patch",
        }
    }
}

//endregion
