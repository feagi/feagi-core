use delegate::delegate;


#[derive(Copy, Clone, Debug, Hash, serde::Serialize, serde::Deserialize)]
pub struct UuidV4(uuid::Uuid);

impl UuidV4 {

    #[cfg(feature = "std")]
    pub fn new_std() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn new_with_rng<E: core::error::Error>
    (mut fill_random: impl FnMut(&mut [u8]) -> Result<(), E>)
        -> Result<Self, E> {
        let mut bytes = [0u8; 16];
        fill_random(&mut bytes)?;
        Ok(Self(
            uuid::Builder::from_random_bytes(bytes).into_uuid()
        ))
    }
    
    // TODO delegate more
    delegate! {
        to self.0 {
            pub fn as_bytes(&self) -> &[u8; 16];
        }
    }


}


