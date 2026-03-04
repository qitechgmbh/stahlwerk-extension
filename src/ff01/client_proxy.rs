use std::thread;
use std::mem::discriminant;

use smol::channel::{TryRecvError, TrySendError, Receiver, Sender, unbounded};

use beas_bsl::{Client, ClientConfig, api::BackflushRequest, api::backflush::IssueLine};

use super::{
    Entry, 
    Request, 
    Response, 
    Worker, 
    TransactionError, 
    ResponseError,
    ClientTransactionError
};

#[derive(Debug, Clone)]
pub struct ProxyClient
{
    sender:   Sender<Request>, 
    receiver: Receiver<Result<Response, ResponseError>>, 
    
    pending_transaction: Option<Request>
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
        TransactionError::Response(err)
    }
}

impl ProxyClient
{
    pub fn new(config: ClientConfig) -> Result<Self, ClientTransactionError>
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
        
        Ok(Self { sender: client_sender, receiver: client_receiver, pending_transaction: None })
    }
    
    pub fn get_next_entry(&mut self) -> Result<Entry, TransactionError>
    {
        match self.update_transaction(Request::GetNextEntry)? 
        {
            Response::GetNextEntry(entry) => Ok(entry),
            _ => Err(TransactionError::TagMismatch)
        }
    }
    
    pub fn get_quantity_scrap(&mut self, entry: &Entry) -> Result<f64, TransactionError>
    {
        let request = Request::GetScrapQuantity(entry.doc_entry, entry.line_number);
        match self.update_transaction(request)?
        {
            Response::GetScrapQuantity(v) => Ok(v),
            _ => Err(TransactionError::TagMismatch),
        }
    }
    
    /// TODO: figure out everything that is needed
    pub fn finalize(&mut self, 
        entry: &Entry,
        scrap_quantity:   f32,
        counted_quantity: f32,
    ) -> Result<(), TransactionError>
    {
        let issue_line = IssueLine {
            line_number2: 10,
            quantity:     counted_quantity,
            item_code:    entry.item_code.to_owned(),
            whs_code:     entry.whs_code.to_owned(),
        };

        let data = BackflushRequest {
            doc_entry:      entry.doc_entry,
            line_number:    entry.line_number,
            doc_date:       None,
            close_entry:    false,
            quantity_good:  counted_quantity - scrap_quantity,
            issue_lines:    vec![issue_line],
            receipt_lines:  Vec::new(),
        };

        let request = Request::Backflush(data);
        match self.update_transaction(request)?
        {
            Response::Backflush => Ok(()),
            _ => Err(TransactionError::TagMismatch),
        }
    }
    
    fn update_transaction(&mut self, request: Request) -> Result<Response, TransactionError>
    {
        match &self.pending_transaction
        {
            Some(pending_request) =>
            {
                if discriminant(pending_request) != discriminant(&request)
                {
                    return Err(TransactionError::TagMismatch);
                }
                
                match self.receiver.try_recv()
                {
                    Ok(result) => 
                    {
                        self.pending_transaction = None;
                        return Ok(result?)
                    },
                    Err(error) =>
                    {
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
                self.sender.try_send(request.clone())?;
                self.pending_transaction = Some(request.clone());
                
                return Err(TransactionError::Pending);
            },
        }
    }
}