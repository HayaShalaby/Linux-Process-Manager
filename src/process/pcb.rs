
/// This holds key metrics retrieved from the Linux kernel via /proc.
#[derive(Debug, Clone)]
pub struct PcbData {
    pub cpu_percent: f32, 
    pub memory_rss_mb: u64, 
    pub state: char,      
    pub priority: i32,
    pub uptime_seconds: u64, // Process runtime in seconds
}