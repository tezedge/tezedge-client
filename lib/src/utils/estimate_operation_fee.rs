pub fn estimate_operation_fee(
    base_fee: u64,
    ntez_per_byte: u64,
    ntez_per_gas: u64,

    estimated_gas: u64,
    estimated_bytes: u64,
) -> u64
{
    // add 32 bytes for the branch block hash.
    let estimated_bytes = estimated_bytes + 32;

    base_fee
        + ntez_per_byte * estimated_bytes / 1000
        + ntez_per_gas * (estimated_gas) / 1000
        // correct rounding error for above two divisions
        + 2
}
