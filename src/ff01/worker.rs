use std::{thread, time::Duration};

use beas_bsl::Client;
use smol::channel::{TryRecvError, TrySendError};

use crate::ff01::{ 
    Response, WorkerReceiver, WorkerSender, requests::{ 
        get_next_entry, 
        get_scrap_quantity, 
        post_time_receipt 
    } 
};

use super::{InternalRequest, ResponseError};

#[derive(Debug)]
pub enum WorkerError
{
    Closed,
    Timeout,
}

pub struct Worker
{
    client:   Client,
    sender:   WorkerSender, 
    receiver: WorkerReceiver, 
}

impl Worker
{
    /// max number of attempts before yielding a Timeout
    /// Error
    const MAX_SEND_ATTEMPTS:   u32 = 50; //TODO: pass via params in new
    const SEND_TIMEOUT_MILLIS: u64 = 50;
}

impl Worker
{
    pub fn new(client: Client, receiver: WorkerReceiver, sender: WorkerSender) -> Self { 
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
            
            use InternalRequest::*;
            match request
            {
                GetNextEntry => {
                    match get_next_entry(&self.client) {
                        Ok(v)  => self.send(Ok(Response::GetNextEntry(v)))?,
                        Err(e) => self.send(Err(e))?,
                    };
                },
                GetScrapQuantity(doc_entry, line_number) => {
                    match get_scrap_quantity(&self.client, doc_entry, line_number) {
                        Ok(v)  => self.send(Ok(Response::GetScrapQuantity(v)))?,
                        Err(e) => self.send(Err(e))?,
                    };
                },
                Finalize(data) => {
                    match post_time_receipt(&self.client, data) {
                        Ok(_)  => self.send(Ok(Response::Finalize))?,
                        Err(e) => self.send(Err(e))?,
                    };
                },
                Terminate => return Ok(()),
            }   
        }
    }
    
    /// attempt to send a message to the owning proxy_client. If number of attempts
    /// exceeds limit terminate return a timeout error which would terminate
    /// the worker, since no connection to the owning Object can be established.
    fn send(&self, mut payload: Result<Response, ResponseError>) -> Result<(), WorkerError>
    {
        for attempt in 0..=Self::MAX_SEND_ATTEMPTS {
            match self.sender.try_send(payload) {
                Ok(_) => return Ok(()),
                Err(TrySendError::Closed(_)) => return Err(WorkerError::Closed),
                Err(TrySendError::Full(data)) => 
                {
                    if attempt == Self::MAX_SEND_ATTEMPTS {
                        return Err(WorkerError::Timeout);
                    }

                    payload = data;
                    thread::sleep(Duration::from_millis(Self::SEND_TIMEOUT_MILLIS));
                }
            }
        }

        // This point should never be reached
        Err(WorkerError::Timeout)
    }
}