pub use beas_bsl::ClientConfig;

pub mod ff01;

#[derive(Debug, Clone)]
pub struct TargetRange
{
    pub min:     f64,
    pub max:     f64,
    pub desired: f64
}