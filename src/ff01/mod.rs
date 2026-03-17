pub use beas_bsl::TransactionError as ClientTransactionError;
use beas_bsl::api::{Date, Time};
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
    GetWorkerSubmission(&'a Entry),
    Finalize(FinalizeRequest),
}

impl<'a> Request<'a>
{
    fn to_internal(self) -> InternalRequest {
        use Request::*;

        match self {
            GetNextEntry => 
                InternalRequest::GetNextEntry,
            GetWorkerSubmission(entry) => 
                InternalRequest::GetWorkerSubmission(entry.doc_entry, entry.line_number),
            Finalize(request) => {

                let quantity_good = request.quantity_counted as f64 - request.quantity_scrap;
                let quantity_good = quantity_good.max(0.0);

                let duration = request.from_time.compute_duration(request.to_time);

                let request = time_receipt::post::Request {
                    doc_entry:          request.doc_entry,
                    line_number:        10,
                    line_number2:       10,
                    line_number3:       Some(0),
                    time_type:          Some("A".to_string()),
                    resource_id:        Some("FF01".to_string()),
                    quantity_good:      Some(quantity_good),
                    personnel_id:       request.personnel_id,
                    quantity_scrap:     Some(0.0),
                    start_date:         Some(request.start_date),
                    end_date:           Some(request.end_date),
                    from_time:          Some(request.from_time),
                    to_time:            Some(request.to_time),
                    close_entry:        Some(true),
                    manual_booking:     Some(false),
                    duration:           Some(duration.as_secs_f32()),
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
    GetWorkerSubmission(i32, i32),
    Finalize(time_receipt::post::Request),
    Terminate,
}

#[derive(Debug, Clone)]
pub enum Response
{ 
    GetNextEntry(Option<Entry>),
    GetWorkerSubmission(Option<(String, f64)>),
    Finalize,
}

#[derive(Debug, Clone)]
pub struct Entry
{
    pub doc_entry:      i32,
    pub line_number:    i32,
    pub item_code:      String,
    pub whs_code:       String,
    pub weight_bounds:  TargetRange, 
}

#[derive(Debug, Clone)]
pub struct FinalizeRequest {
    pub doc_entry: i32,
    pub personnel_id: String,
    pub start_date: Date,
    pub end_date: Date,
    pub from_time: Time,
    pub to_time: Time,

    pub quantity_scrap: f64,
    pub quantity_counted:  u32,
}