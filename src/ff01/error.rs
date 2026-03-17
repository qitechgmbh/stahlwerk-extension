pub use beas_bsl::TransactionError as ClientTransactionError;

use crate::ff01::{InternalRequest, TrySendError};

#[derive(Debug)]
pub enum ProxyTransactionError {
    NoPendingRequest,
    Pending,
    ChannelFull,
    ChannelClosed,
    TagMismatch,
    Response(ResponseError),
}

impl From<TrySendError<InternalRequest>> for ProxyTransactionError {
    fn from(err: TrySendError<InternalRequest>) -> Self {
        match err {
            TrySendError::Full(_) => ProxyTransactionError::ChannelFull,
            TrySendError::Closed(_) => ProxyTransactionError::ChannelClosed,
        }
    }
}

impl From<ResponseError> for ProxyTransactionError {
    fn from(err: ResponseError) -> Self {
        ProxyTransactionError::Response(err)
    }
}

#[derive(Debug)]
pub enum ResponseError {
    ClientTransaction(ClientTransactionError),
    InvalidData(String),
}

impl From<ClientTransactionError> for ResponseError {
    fn from(err: ClientTransactionError) -> Self {
        ResponseError::ClientTransaction(err)
    }
}
