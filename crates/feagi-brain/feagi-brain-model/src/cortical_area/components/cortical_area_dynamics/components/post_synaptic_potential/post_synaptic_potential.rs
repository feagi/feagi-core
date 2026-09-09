/// This trait defines how
pub trait PostSynapticPotential {
    fn get_if_psp_is_mp_driven(&self) -> bool;
}
