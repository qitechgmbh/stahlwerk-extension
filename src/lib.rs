pub use beas_bsl::ClientConfig;

mod ff01;
pub use ff01::Entry;
pub use ff01::ProxyClient;
pub use ff01::TransactionError;
pub use ff01::ClientTransactionError;

#[derive(Debug, Clone)]
pub struct Bounds
{
    pub min:     f64,
    pub max:     f64,
    pub desired: f64
}