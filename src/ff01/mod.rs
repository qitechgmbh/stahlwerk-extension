pub use beas_bsl::TransactionError as ClientTransactionError;
use beas_bsl::api::BackflushRequest;

use crate::Bounds;

mod worker;
mod requests;
mod client_proxy;

use worker::Worker;
pub use client_proxy::ProxyClient;

#[derive(Debug)]
pub enum TransactionError
{
    Pending,
    ChannelFull,
    ChannelClosed,
    TagMismatch,
    Response(ResponseError)
}

#[derive(Debug, Clone)]
pub(crate) enum Request
{
    GetNextEntry,
    GetScrapQuantity(i32, i32),
    Backflush(BackflushRequest),
    Terminate,
}

#[derive(Debug, Clone)]
pub(crate) enum Response
{ 
    GetNextEntry(Entry),
    GetScrapQuantity(f64),
    Backflush,
    Terminate,
}

#[derive(Debug)]
pub enum ResponseError
{
    ClientTransaction(ClientTransactionError),
    InvalidData(String),
}

impl From<ClientTransactionError> for ResponseError
{
    fn from(err: ClientTransactionError) -> Self
    {
        ResponseError::ClientTransaction(err)
    }
}

#[derive(Debug, Clone)]
pub struct Entry
{
    pub doc_entry: i32,
    pub line_number: i32,
    pub scrap_quantity: f64,
    pub item_code: String,
    pub whs_code: String,
    pub weight_bounds: Bounds,
}