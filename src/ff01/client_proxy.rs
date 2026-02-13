use std::thread;
use std::mem::discriminant;

use beas_bsl::Client;
use beas_bsl::ClientConfig;
use beas_bsl::ClientError;

use smol::channel::TryRecvError;
use smol::channel::TrySendError;
use smol::channel::{ Receiver, Sender, unbounded };

use crate::ff01::Entry;
use crate::ff01::ResponseError;

use super::Request;
use super::Response;
use super::worker::Worker;

#[derive(Debug, Clone)]
pub struct ProxyClient
{
    sender:   Sender<Request>, 
    receiver: Receiver<Result<Response, ResponseError>>, 
    
    pending_transcation: Option<Request>
}

#[derive(Debug)]
pub enum TransactionError
{
    Pending,
    ChannelFull,
    ChannelClosed,
    TagMismatch,
    ResponseError(ResponseError)
}

impl From<TrySendError<Request>> for TransactionError
{
    fn from(err: TrySendError<Request>) -> Self
    {
        match err
        {
            TrySendError::Full(_)   => TransactionError::ChannelFull,
            TrySendError::Closed(_) => TransactionError::ChannelClosed,
        }
    }
}

impl From<ResponseError> for TransactionError
{
    fn from(err: ResponseError) -> Self
    {
        TransactionError::ResponseError(err)
    }
}

impl ProxyClient
{
    pub fn new(config: ClientConfig) -> Result<Self, ClientError>
    {
        let client = Client::new(config)?;
        
        let (client_sender, worker_receiver) = unbounded::<Request>();
        let (worker_sender, client_receiver) = unbounded::<Result<Response, ResponseError>>();
        
        let worker = Worker::new(client, worker_receiver, worker_sender);
        
        let worker_handle = thread::spawn(move || 
        {
            match worker.run()
            {
                Ok(_) => eprintln!("[WORK] shutdown."),
                Err(e) => eprintln!("[WORK] aborted: {:?}", e),
            }
        });
        
        // discard handle to get independent thread
        // TODO: maybe handle termination gracefully?
        _ = worker_handle;
        
        Ok(Self { sender: client_sender, receiver: client_receiver, pending_transcation: None })
    }
    
    pub fn get_next_entry(&mut self) -> Result<Entry, TransactionError>
    {
        match self.update_transaction(Request::GetNextEntry)? 
        {
            Response::GetNextEntry(entry) => Ok(entry),
            _ => Err(TransactionError::TagMismatch)
        }
    }
    
    pub fn get_scrap_quantity(&mut self, entry: &Entry) -> Result<f64, TransactionError>
    {
        match self.update_transaction(Request::GetScrapQuantity(entry.doc_entry))?
        {
            Response::GetScrapQuantity(v) => Ok(v),
            _ => Err(TransactionError::TagMismatch),
        }
    }
    
    /// TODO: figure out everything that is needed
    pub fn finalize(&mut self) -> Result<Option<()>, TransactionError>
    {
        todo!();
    }
    
    fn update_transaction(&mut self, request: Request) -> Result<Response, TransactionError>
    {
        match &self.pending_transcation
        {
            Some(pending_request) =>
            {
                if discriminant(pending_request) != discriminant(&request)
                {
                    return Err(TransactionError::TagMismatch);
                }
                
                match self.receiver.try_recv()
                {
                    Ok(result) => return Ok(result?),
                    Err(error) =>
                    {
                        println!("[MAIN] Failed to receive: {:?}", error);
                        
                        match error
                        {
                            TryRecvError::Empty  => return Err(TransactionError::Pending),
                            TryRecvError::Closed => return Err(TransactionError::ChannelClosed),
                        }
                    },
                }
            },
            None => 
            {
                println!("[MAIN] Sending request");
                self.sender.try_send(request.clone())?;
                self.pending_transcation = Some(request.clone());
                
                println!("[MAIN] Sent request");
                return Err(TransactionError::Pending);
            },
        }
    }
}