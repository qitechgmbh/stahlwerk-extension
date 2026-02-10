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
    SendError(TrySendError<Request>),
    ChannelClosed,
    TagMismatch,
    ResponseError(ResponseError)
}

impl From<TrySendError<Request>> for TransactionError
{
    fn from(err: TrySendError<Request>) -> Self
    {
        TransactionError::SendError(err)
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
        let backing_client = Client::new(config)?;
        
        let (client_sender, worker_receiver) = unbounded::<Request>();
        let (worker_sender, client_receiver) = unbounded::<Result<Response, ResponseError>>();
        
        let worker = Worker::new(backing_client, worker_receiver, worker_sender);
        
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
    
    // steps: get next entry -> 
    
    // which informations do we need in step one??
    
    // compare scrap amount with entry and current
    
    // simply call backflush
    
    pub fn get_next_entry(&mut self) -> Result<Option<Entry>, TransactionError>
    {
        match self.update_transaction(Request::GetNextEntry)?
        {
            Some(entry) => 
            {
                match entry 
                {
                    Response::GetNextEntry(entry) => Ok(entry),
                    _ => Err(TransactionError::TagMismatch)
                }
            },
            None => Ok(None),
        }
    }
    
    pub fn check_scrap_update(&mut self) -> Result<Option<()>, TransactionError>
    {
        todo!();
    }
    
    pub fn finalize(&mut self) -> Result<Option<()>, TransactionError>
    {
        todo!();
    }
    
    fn update_transaction(&mut self, request: Request) -> Result<Option<Response>, TransactionError>
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
                    Ok(result) => return Ok(Some(result?)),
                    Err(error) =>
                    {
                        println!("[MAIN] Failed to receive: {:?}", error);
                        
                        match error
                        {
                            TryRecvError::Empty  => return Ok(None),
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
                return Ok(None);
            },
        }
    }
}