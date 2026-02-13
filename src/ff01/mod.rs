use beas_bsl::ClientError;

mod worker;
mod worker_x;
mod client_proxy;

pub use client_proxy::ProxyClient;
pub use client_proxy::TransactionError;

#[derive(Debug, Clone)]
pub enum Request
{
    GetNextEntry,
    GetScrapQuantity(i32),
    Backflush,
    Terminate,
}

#[derive(Debug, Clone)]
pub enum Response
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
pub enum GetNextEntryError
{
    NoWorkorder,
}


#[derive(Debug, Clone)]
pub struct Entry
{
    pub doc_entry: i32,
    
    /// Used to check WorkorderPos for updates in 
    pub scrap_quantity: f64,
    
    pub bounds: Bounds
}

#[derive(Debug, Clone)]
pub struct Bounds
{
    pub min:     f64,
    pub max:     f64,
    pub desired: f64
}