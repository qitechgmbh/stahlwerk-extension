use std::{thread, time::Duration};

use beas_bsl::Client;
use smol::channel::{Receiver, Sender, TryRecvError, TrySendError };

use crate::ff01::{ Response, requests::{ get_next_entry, get_quantity_scrap } };

use super::{Request, ResponseError};

#[derive(Debug)]
pub enum WorkerError
{
    Closed,
    Timeout,
}

pub struct Worker
{
    client:   Client,
    sender:   Sender<Result<Response, ResponseError>>, 
    receiver: Receiver<Request>, 
}

impl Worker
{
    /// max number of attempts before yielding a Timeout
    /// Error
    const MAX_SEND_ATTEMPTS:   u32 = 50; //TODO: pass via params in new
    const SEND_TIMEOUT_MILLIS: u64 = 50;
    
    pub fn new(
        client: Client, 
        receiver: Receiver<Request>, 
        sender: Sender<Result<Response, ResponseError>>
    ) -> Self
    { 
        Self { client, receiver, sender } 
    }
    
    pub fn run(self) -> Result<(), WorkerError>
    {
        loop 
        {
            let request = match self.receiver.try_recv()
            {
                Ok(v) => v,
                Err(TryRecvError::Empty) =>
                {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                },
                Err(TryRecvError::Closed) => return Err(WorkerError::Closed),
            };
            
            match request
            {
                Request::GetNextEntry =>
                {
                    match get_next_entry(&self.client)
                    {
                        Ok(v)  => self.send(Ok(Response::GetNextEntry(v)))?,
                        Err(e) => self.send(Err(e))?,
                    };
                },
                Request::GetScrapQuantity(doc_entry) => 
                {
                    match get_quantity_scrap(&self.client, doc_entry)
                    {
                        Ok(v)  => self.send(Ok(Response::GetScrapQuantity(v)))?,
                        Err(e) => self.send(Err(ResponseError::Client(e)))?,
                    };
                },
                Request::Backflush => 
                {
                    
                },
                Request::Terminate => return Ok(()),
            }   
        }
    }
    
    /// attempt to send a message to the owning proxy_client. If number of attempts
    /// exceeds limit terminate return a timeout error which would terminate
    /// the worker, since no connection to the owning Object can be established.
    fn send(&self, payload: Result<Response, ResponseError>) -> Result<(), WorkerError>
    {
        let mut attempts: u32 = 0;
        let mut payload = payload;
        
        loop
        {
            match self.sender.try_send(payload)
            {
                Ok(_) => return Ok(()),
                Err(TrySendError::Closed(_)) => return Err(WorkerError::Closed),
                Err(TrySendError::Full(data)) => 
                {
                    if attempts >= Self::MAX_SEND_ATTEMPTS
                    {
                        return Err(WorkerError::Timeout);
                    }
                    
                    payload = data;
                    attempts += 1;
                    thread::sleep(Duration::from_millis(Self::SEND_TIMEOUT_MILLIS));
                    continue;
                }
            }
        }
    }
}