pub use beas_bsl::TransactionError as ClientTransactionError;
use beas_bsl::api::Date;
use beas_bsl::api::{time_receipt};

use crate::TargetRange;

mod worker;
use worker::Worker;

mod requests;

mod client_proxy;
pub use client_proxy::ProxyClient;

mod error;
pub use error::ProxyTransactionError;
pub use error::ResponseError;

type WorkerSender   = smol::channel::Sender<Result<Response, ResponseError>>;
type WorkerReceiver = smol::channel::Receiver<InternalRequest>;

type ProxySender    = smol::channel::Sender<InternalRequest>;
type ProxyReceiver  = smol::channel::Receiver<Result<Response, ResponseError>>;

type TrySendError<T> = smol::channel::TrySendError<T>;
type TryRecvError    = smol::channel::TryRecvError;

#[derive(Debug, Clone)]
pub enum Request<'a>
{
    GetNextEntry,
    GetScrapQuantity(&'a Entry),
    Finalize(&'a Entry, u32),
}

impl<'a> Request<'a>
{
    fn to_internal(self) -> InternalRequest {
        use Request::*;

        match self {
            GetNextEntry => 
                InternalRequest::GetNextEntry,
            GetScrapQuantity(entry) => 
                InternalRequest::GetScrapQuantity(entry.doc_entry, entry.line_number),
            Finalize(entry, quantity_counted) => {

                let quantity_good = quantity_counted as f64 - entry.scrap_quantity;
                let quantity_good = quantity_good.max(0.0);

                let request = time_receipt::post::Request {
                    doc_entry:          entry.doc_entry,
                    line_number:        10,
                    line_number2:       10,
                    line_number3:       Some(0),
                    time_type:          Some("A".to_string()),
                    resource_id:        Some("FF01".to_string()),
                    quantity_good:      Some(quantity_good),
                    personnel_id:       "04711".to_string(),
                    quantity_scrap:     Some(0.0),
                    start_date:         Some(Date { year: 2026, month: 03, day: 12 }),
                    end_date:           Some(Date { year: 2026, month: 03, day: 12 }),
                    from_time:          Some("12:00".to_string()),
                    to_time:            Some("15:00".to_string()),
                    close_entry:        Some(true),
                    manual_booking:     Some(false),
                    duration:           Some(60),
                    calculate_duration: Some(false),
                    remarks:            Some("QiTech-Control".to_string()),
                    ..Default::default()
                };

                InternalRequest::Finalize(request)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InternalRequest
{
    GetNextEntry,
    GetScrapQuantity(i32, i32),
    Finalize(time_receipt::post::Request),
    Terminate,
}

#[derive(Debug, Clone)]
pub enum Response
{ 
    GetNextEntry(Option<Entry>),
    GetScrapQuantity(Option<f64>),
    Finalize,
}

#[derive(Debug, Clone)]
pub struct Entry
{
    pub doc_entry:      i32,
    pub line_number:    i32,
    pub scrap_quantity: f64,
    pub item_code:      String,
    pub whs_code:       String,
    pub weight_bounds:  TargetRange, 
    pub personnel_id:   String,
}