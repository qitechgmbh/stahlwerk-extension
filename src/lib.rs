pub use beas_bsl::ClientConfig;
pub use beas_bsl::api::Date;
pub use beas_bsl::api::Time;

pub mod ff01;

#[derive(Debug, Clone)]
pub struct TargetRange
{
    pub min:     f64,
    pub max:     f64,
    pub desired: f64
}

impl TargetRange {
    pub fn in_bounds(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}