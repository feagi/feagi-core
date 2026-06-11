

/// Calculates the size of a padding to ensure that the struct is aligned to hardware channels
pub(crate) const fn calculate_byte_alignment_padding(size_in_bytes_of_prior_members: usize) -> usize
{
    if size_in_bytes_of_prior_members <= 4 {
        return 4 - size_in_bytes_of_prior_members
    } else if size_in_bytes_of_prior_members <= 8 {
        return 8 - size_in_bytes_of_prior_members
    } else if size_in_bytes_of_prior_members <= 16 {
        return 16 - size_in_bytes_of_prior_members
    } else if size_in_bytes_of_prior_members <= 32 {
        return 32 - size_in_bytes_of_prior_members
    } else if size_in_bytes_of_prior_members <= 64 {
        return 64 - size_in_bytes_of_prior_members
    }
    128 - size_in_bytes_of_prior_members // What are you even doing to end up here?
}