use beas_bsl::ClientError;

mod worker;
mod client_proxy;

pub use client_proxy::ProxyClient;
pub use client_proxy::TransactionError;

use crate::types::WeightedItem;

#[derive(Debug, Clone)]
pub enum Request
{
    GetNextEntry,
    GetCurrentOrderPos,
    Backflush,
    Terminate,
}

#[derive(Debug, Clone)]
pub enum Response
{ 
    GetNextEntry(Option<Entry>),
    GetOrderPosCurrent(),
    Backflush(),
    Terminate,
}

#[derive(Debug)]
pub enum ResponseError
{
    Client(ClientError),
    Error(String),
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
    pub doc_entry: u32,
    
    // key for accesing WorkorderPos
    pub line_number: u32,
    
    /// Used to check WorkorderPos for updates in 
    pub quantity_scrap: u32,
    
    pub weighted_item: WeightedItem
}