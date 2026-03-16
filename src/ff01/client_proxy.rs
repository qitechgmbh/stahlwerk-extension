use std::thread;
use std::mem::discriminant;

use smol::channel::{TryRecvError, Receiver, Sender, unbounded};

use beas_bsl::{Client, ClientConfig};

use crate::ff01::Request;

use super::{
    Entry, 
    InternalRequest, 
    Response, 
    Worker, 
    ProxyTransactionError, 
    ResponseError,
    ClientTransactionError
};

#[derive(Debug, Clone)]
pub struct ProxyClient
{
    sender:   Sender<InternalRequest>, 
    receiver: Receiver<Result<Response, ResponseError>>, 
    state:    State,
}

pub struct RequestMaker<'a>
{
    state: &'a mut State,
}

#[derive(Debug, Clone)]
pub enum State 
{
    Idle,
    Sending(InternalRequest),
    Pending,
}

impl ProxyClient
{
    pub fn new(config: ClientConfig) -> Result<Self, ClientTransactionError> {
        let client = Client::new(config)?;
        
        let (client_sender, worker_receiver) = unbounded::<InternalRequest>();
        let (worker_sender, client_receiver) = unbounded::<Result<Response, ResponseError>>();
        
        let worker = Worker::new(client, worker_receiver, worker_sender);
        
        let worker_handle = thread::spawn(move || 
        {
            match worker.run()
            {
                Ok(_)  => eprintln!("[WORKER] shutdown."),
                Err(e) => eprintln!("[WORKER] aborted: {:?}", e),
            }
        });
        
        // discard handle to get independent thread
        _ = worker_handle;
        
        Ok(Self { sender: client_sender, receiver: client_receiver, state: State::Idle })
    }
    
    pub fn can_queue_request(&mut self) -> bool{
        return matches!(self.state, State::Idle);
    }

    pub fn queue_request(&mut self, request: Request) -> Result<(), ()> {
        if !self.can_queue_request() { return Err(()); }
        self.state = State::Sending(request.to_internal());
        Ok(())
    }

    pub fn poll_response(&mut self) -> Result<Response, ProxyTransactionError>
    {
        use State::*;

        match &self.state {
            Idle => Err(ProxyTransactionError::NoPendingRequest),
            Sending(request) => {
                self.sender.try_send(request.clone())?;
                self.state = Pending;
                Err(ProxyTransactionError::Pending)
            },
            Pending => {
                match self.receiver.try_recv() {
                    Ok(result) => {
                        self.state = Idle;
                        Ok(result?)
                    },
                    Err(error) => {
                        match error {
                            TryRecvError::Empty  => Err(ProxyTransactionError::Pending),
                            TryRecvError::Closed => Err(ProxyTransactionError::ChannelClosed),
                        }
                    },
                }
            },
        }
    }

    pub fn finalize(&mut self, 
        entry: &Entry,
        counted_quantity: u32,
    ) -> Result<(), ProxyTransactionError>
    {
        // let quantity_good = counted_quantity - entry.scrap_quantity;
// 
        // let issue_line = IssueLine {
        //     line_number2: 10,
        //     quantity:     counted_quantity as f32,
        //     item_code:    entry.item_code.to_owned(),
        //     whs_code:     entry.whs_code.to_owned(),
        // };
// 
        // let data = BackflushRequest {
        //     doc_entry:      entry.doc_entry,
        //     line_number:    entry.line_number,
        //     doc_date:       None,
        //     close_entry:    false,
        //     quantity_good:  counted_quantity - entry.scrap_quantity,
        //     issue_lines:    vec![issue_line],
        //     receipt_lines:  Vec::new(),
        // };
// 
        // let request = Request::Backflush(data);
        // match self.update_transaction(request)?
        // {
        //     Response::Backflush => Ok(()),
        //     _ => Err(TransactionError::TagMismatch),
        // }

        Ok(())
    }
}