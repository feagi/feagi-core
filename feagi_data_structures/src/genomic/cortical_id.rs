use crate::FeagiDataError;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorticalID {
    pub(crate) bytes: [u8; CorticalID::CORTICAL_ID_LENGTH],
}

impl CorticalID {

    pub const CORTICAL_ID_LENGTH: usize = 8; // 8 bytes -> 64 bit

    pub const NUMBER_OF_BYTES: usize = Self::CORTICAL_ID_LENGTH;

    //region Constructors

    pub fn try_from_bytes(bytes: [u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<Self, FeagiDataError> {
        todo!()
    }



    //endregion





    //region internal



    //endregion

}