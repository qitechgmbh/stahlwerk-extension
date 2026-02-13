use beas_bsl::ClientError;

mod worker;
mod worker_x;
mod client_proxy;

pub use client_proxy::ProxyClient;
pub use client_proxy::TransactionError;

use crate::Bounds;

#[derive(Debug, Clone)]
pub(crate) enum Request
{
    GetNextEntry,
    GetScrapQuantity(i32),
    Backflush,
    Terminate,
}

#[derive(Debug, Clone)]
pub(crate) enum Response
{ 
    GetNextEntry(Entry),
    GetScrapQuantity(f64),
    Backflush(),
    Terminate,
}

#[derive(Debug)]
pub enum ResponseError
{
    Client(ClientError),
    InvalidData(String),
}

impl From<ClientError> for ResponseError
{
    fn from(err: ClientError) -> Self
    {
        ResponseError::Client(err)
    }
}

#[derive(Debug, Clone)]
pub struct Entry
{
    pub doc_entry: i32,
    
    /// Used to check WorkorderPos for updates in 
    pub scrap_quantity: f64,
    
    pub weight_bounds: Bounds
}