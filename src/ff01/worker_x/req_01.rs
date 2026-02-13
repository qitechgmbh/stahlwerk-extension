use beas_bsl::{Client, ClientError, api::{FilterBuilder, Ordering, QueryOptions, Workorder}};

use crate::ff01::{Bounds, Entry, ResponseError};

fn data_error<T, M>(msg: M) -> Result<T, ResponseError>
where
    M: Into<String>,
{
    Err(ResponseError::InvalidData(msg.into()))
}

pub fn get_next_entry(client: &Client) -> Result<Entry, ResponseError>
{
    // step 1: grab the next workorder that matches our criteria
    println!("Step: 1");
    
    let workorder = match get_workorder(client)?
    {
        Some(workorder) => workorder,
        None => return data_error("No Workorders"),
    };
    
    _ = workorder;
    //let doc_entry = workorder.doc_entry;
    
    //TODO: remove later
    let doc_entry: i32 = 50492;
    
    // step 2: validate resource-id
    println!("Step: 2 ({})", doc_entry);
    
    let wo_routing = match get_workorder_routing(client, doc_entry)?
    {
        Some(workorder) => workorder,
        None => return data_error(format!("No matching WorkorderRoutings for {}", doc_entry)),
    };
    
    let resource_id = unpack_nullable(wo_routing.resource_id)?;
    
    if resource_id != "FF01"
    {
        let msg = format!("Invalid ResourceId. expected: FF01, received: {}", resource_id);
        return data_error(msg);
    }
    
    // step 3: Get current quantity_scrap from workorder-pos
    println!("Step: 3");
    
    let wo_pos = match get_workorder_pos(client, doc_entry)?
    {
        Some(value) => value,
        None => todo!(),
    };
    
    let quantity_scrap = match wo_pos.quantity_scrap 
    {
        Some(value) => value,
        None => todo!(),
    };
    
    // step 4: get bounds
    println!("Step: 4");
    
    let qcorder_measurement = unpack_nullable(get_qcorder_measurement(client, doc_entry)?)?;
    
    let min     = unpack_nullable(qcorder_measurement.minimal)?;
    let max     = unpack_nullable(qcorder_measurement.maximum)?;
    let desired = unpack_nullable(qcorder_measurement.desired_value)?;

    let weight_bounds = Bounds { min, max, desired };
    
    // step 5: return result
    Ok(Entry { doc_entry, scrap_quantity: quantity_scrap, weight_bounds })
}

pub fn get_quantity_scrap(client: &Client, doc_entry: i32) -> Result<f64, ClientError>
{
    let wo_pos = match get_workorder_pos(client, doc_entry)?
    {
        Some(value) => value,
        None => todo!(),
    };
    
    match wo_pos.quantity_scrap 
    {
        Some(value) => Ok(value),
        None => todo!(),
    }
}

fn get_workorder(client: &Client) -> Result<Option<Workorder>, ClientError>
{
    let filter = 
        FilterBuilder::new()
        .equals("ApsStatus", true).and()
        .equals("Closed", 0)
        .build();

    let options = 
        QueryOptions::new()
        .top(1)
        .skip(0)
        .order_by("DocEntry", Ordering::Descending)
        .filter(filter);
       
    match client.request().production().workorder().get(options)
    {
        Ok(v)  => Ok(v.first().cloned()),
        Err(e) => Err(e),
    }
}

pub fn get_workorder_pos(client: &Client, doc_entry: i32) -> Result<Option<beas_bsl::api::WorkorderPosition>, ClientError>
{
    let filter = 
        FilterBuilder::new()
        .equals("DocEntry", doc_entry).and()
        .equals("LineNumber", 10)
        .build();
    
    let options = QueryOptions::new().filter(filter);
    
    let result = 
        client
        .request()
        .production()
        .workorder_pos()
        .get(options);
        
    match result
    {
        Ok(items) => Ok(items.first().cloned()),
        Err(e) => Err(e),
    }
}

fn get_workorder_routing(client: &Client, doc_entry: i32) -> Result<Option<beas_bsl::api::WorkorderRouting>, ClientError>
{
    let filter = 
        FilterBuilder::new()
        .equals("DocEntry", doc_entry)
        .and()
        .equals("LineNumber", 10)
        .and()
        .equals("LineNumber2", 10)
        .build();
    
    let options = QueryOptions::new().filter(filter);
    
    let result = 
        client
        .request()
        .production()
        .workorder_routing()
        .get(options);
        
    match result
    {
        Ok(items) => Ok(items.first().cloned()),
        Err(e) => Err(e),
    }
}

fn get_qcorder_measurement(client: &Client, doc_entry: i32) -> Result<Option<beas_bsl::api::QCOrderMeasurement>, ClientError>
{
    let filter = 
        FilterBuilder::new()
        .equals("WoDocEntry", doc_entry).and()
        .equals("WoLineNumber", 10).and()
        .equals("LineNumber2", 10).and()
        .equals("QCDescription", "Zuschnitt_Gewicht")
        .build();
    
    let options = QueryOptions::new().filter(filter);
    
    let result = 
        client
        .request()
        .quality_control()
        .qcorder_measurement()
        .get(options);
            
    match result
    {
        Ok(workorders) => Ok(workorders.first().cloned()),
        Err(e) => Err(e),
    }
}

fn unpack_nullable<T>(value: Option<T>) -> Result<T, ResponseError>
{
    match value
    {
        Some(item) => Ok(item),
        None       => return data_error("Received null..."),
    }
}