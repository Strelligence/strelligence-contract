use soroban_sdk::contracttype;

#[contracttype]
pub struct GasProfile {
    pub operation: u32,
    pub read_count: u32,
    pub write_count: u32,
    pub bytes_read: u32,
    pub bytes_written: u32,
}

#[contracttype]
pub struct StorageOptimization {
    pub batch_size: u32,
    pub compress: bool,
    pub cache_enabled: bool,
}
