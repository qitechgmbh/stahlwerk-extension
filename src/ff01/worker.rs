use std::{thread, time::Duration};

use beas_bsl::{Client, ClientError, api::{FilterOperator, Ordering, QueryOptions, WorkorderBom}};
use smol::channel::{Receiver, Sender, TryRecvError, TrySendError };

use crate::ff01::{Entry, Response};

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
    pub fn new(client: Client, receiver: Receiver<Request>, sender: Sender<Result<Response, ResponseError>>) -> Self
    { 
        Self { client, receiver, sender } 
    }
    
    pub fn run(self) -> Result<(), WorkerError>
    {
        loop 
        {
            match self.receiver.try_recv()
            {
                Ok(request)  => 
                {
                    println!("[WORK] handling next request: {:?}", &request);

                    match request
                    {
                        Request::GetNextEntry => 
                        {
                            match self.get_next_entry() 
                            {
                                Ok(entry) => self.send(Ok(Response::GetNextEntry(entry)))?,
                                Err(error) => self.send(Err(error))?,
                            };
                        },
                        Request::GetCurrentOrderPos => 
                        {
                            
                        },
                        Request::Backflush => 
                        {
                            
                        },
                        Request::Terminate => return Ok(()),
                    }
                },
                Err(TryRecvError::Empty) => 
                {
                    // No request is waiting. 
                    // Sleep briefly to avoid high CPU usage.
                    thread::sleep(Duration::from_millis(50));
                },
                Err(TryRecvError::Closed) => return Err(WorkerError::Closed),
            }   
        }
    }
    
    fn send(&self, payload: Result<Response, ResponseError>) -> Result<(), WorkerError>
    {
        const MAX_ATTEMPTS: u32 = 50;
        
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
                    if attempts >= MAX_ATTEMPTS
                    {
                        return Err(WorkerError::Timeout);
                    }
                    
                    payload = data;
                    attempts += 1;
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
            }
        }
    }
    
    fn get_next_entry(&self) -> Result<Option<Entry>, ResponseError>
    {
        let workorder = match self.get_workorder()?
        {
            Some(workorder) => workorder,
            None => return Ok(None),
        };
        
        println!("[WORK] got workorder:");
        
        let boms_raw = self.get_workorders_boms(workorder.doc_entry)?;
        
        let boms = Self::filter_boms(boms_raw);
        
        println!("[WORK] got boms: {}", boms.len());

        let qc_order = self.get_qc_order(workorder.doc_entry)?;

        println!("[WORK] got qc_order: {:?}", qc_order);

        

        Ok(None)
    }
    
    fn filter_boms(items: Vec<WorkorderBom>) -> Vec<WorkorderBom>
    {
        let mut found = Vec::new();

        for item in &items
        {
            if let Some(code) = &item.item_code
            {
                if code.contains("ZURO-") { found.push(item.clone()); }
            }
        }
        
        return found;
    }
    
    fn get_workorder(&self) -> Result<Option<beas_bsl::api::Workorder>, ClientError>
    {
        let options = 
            QueryOptions::new()
            .top(1)
            .skip(0)
            .order_by("StartDate", Ordering::Descending)
            .filter("ApsStatus", FilterOperator::Equal, "true")
            .filter("Closed", FilterOperator::Equal, "0");
        
        let workorders = 
            self.client
            .request()
            .production()
            .workorder()
            .get(options);
            
        match workorders
        {
            Ok(workorders) => Ok(workorders.first().cloned()),
            Err(e) => Err(e),
        }
    }
    
    fn get_workorders_boms(&self, doc_entry: i32) -> Result<Vec<beas_bsl::api::WorkorderBom>, ClientError>
    {
        let options = 
            QueryOptions::new()
            .filter("DocEntry", FilterOperator::Equal, doc_entry.to_string())
            ;
        
        let workorders = 
            self.client
            .request()
            .production()
            .workorder_bom()
            .get(options);
            
        match workorders
        {
            Ok(workorders) => Ok(workorders),
            Err(e) => Err(e),
        }
    }
    
    // 50683
    
    fn get_qc_order(&self, doc_entry: i32) -> Result<Option<beas_bsl::api::QCOrder>, ClientError>
    {
        let options = 
            QueryOptions::new()
            .top(5)
            .skip(0)
            .order_by("DocEntry", Ordering::Descending)
            .filter("DocEntry", FilterOperator::Equal, doc_entry.to_string())
            ;
        
        let qcorders = 
            self.client
            .request()
            .quality_control()
            .qcorder()
            .get(options);
            
        match qcorders
        {
            Ok(qcorders) => Ok(qcorders.first().cloned()),
            Err(e) => Err(e),
        }
    }
}