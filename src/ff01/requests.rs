use beas_bsl::{
    Client, 
    TransactionError, api::{
        BackflushRequest, FilterBuilder, QCOrderMeasurement, QueryOptions, WorkorderBom, WorkorderPosition, WorkorderRouting
    }
};

use crate::ff01::{Bounds, Entry, ResponseError};

pub fn get_next_entry(client: &Client) -> Result<Entry, ResponseError>
{
    // Get Workorder Routing
    let wo_routing = match get_workorder_routing(client)?
    {
        Some(workorder) => workorder,
        None => return data_error("No ready workorders"),
    };

    let doc_entry = wo_routing.doc_entry;
    let line_number = wo_routing.line_number;

    // Get Workorder Bom
    let Some(wo_bom) = get_workorder_bom(client, doc_entry, line_number)? else { 
        return data_error(format!("No matching WorkorderRoutings for {}", doc_entry)); 
    };

    let Some(item_code) = wo_bom.item_code else { 
        return data_error(format!("ItemCode is null"));
    };

    let Some(whs_code) = wo_bom.whs_code else { 
        return data_error(format!("ItemCode is null"));
    };

    // Get Workorder Pos
    let wo_pos = match get_workorder_pos(client, doc_entry, line_number)?
    {
        Some(value) => value,
        None => todo!(),
    };
    
    let Some(scrap_quantity) = wo_pos.quantity_scrap else { 
        return data_error(format!("ItemCode is null"));
    };

    // Get QCOrder Measurement
    let qcorder_measurement = match get_qcorder_measurement(client, doc_entry, line_number)?
    {
        Some(value) => value,
        None => todo!(),
    };
    
    let min     = unpack_nullable(qcorder_measurement.minimal, "min")?;
    let max     = unpack_nullable(qcorder_measurement.maximum, "max")?;
    let desired = unpack_nullable(qcorder_measurement.desired_value, "desired")?;

    let weight_bounds = Bounds { min, max, desired };

    // return result
    let entry = Entry { 
        doc_entry, 
        line_number,
        scrap_quantity, 
        item_code,
        whs_code,
        weight_bounds,
    };

    Ok(entry)
}

fn get_workorder_routing(
    client: &Client
) -> Result<Option<WorkorderRouting>, TransactionError>
{
    let filter = 
        FilterBuilder::new()
            .equals("CurrentRunning", true).and()
            .equals("ResourceId", "FF01").and()
            .equals("Closed", false).and()
            // filter only for the first step
            .equals("LineNumber2", 10)
            .build();
    
    let options = QueryOptions::new().filter(filter);
    
    let result = 
        client
        .single_request()
        .production()
        .workorder_routing()
        .get(options);
        
    match result
    {
        Ok(items) => Ok(items.first().cloned()),
        Err(e) => Err(e),
    }
}

fn get_workorder_bom(client: &Client, doc_entry: i32, line_number: i32) -> Result<Option<WorkorderBom>, TransactionError>
{
    let filter = 
        FilterBuilder::new()
            .equals("DocEntry", doc_entry)
            .and()
            .equals("LineNumber", line_number)
            .and()
            .equals("LineNumber2", 10)
            .build();
    
    let options = QueryOptions::new().filter(filter);
    
    let result = 
        client
        .single_request()
        .production()
        .workorder_bom()
        .get(options);
        
    match result
    {
        Ok(items) => Ok(items.first().cloned()),
        Err(e) => Err(e),
    }
}

fn get_workorder_pos(client: &Client, doc_entry: i32, line_number: i32) -> Result<Option<WorkorderPosition>, TransactionError>
{
    let filter = 
        FilterBuilder::new()
        .equals("DocEntry", doc_entry).and()
        .equals("LineNumber", line_number)
        .build();
    
    let options = QueryOptions::new().filter(filter);
    
    let result = 
        client
        .single_request()
        .production()
        .workorder_pos()
        .get(options);
        
    match result
    {
        Ok(items) => Ok(items.first().cloned()),
        Err(e) => Err(e),
    }
}

pub fn get_scrap_quantity(
    client: &Client, 
    doc_entry: i32, 
    line_number: i32
) -> Result<f64, ResponseError>
{
    let wo_pos = match get_workorder_pos(client, doc_entry, line_number)?
    {
        Some(value) => value,
        None => todo!(),
    };
    
    let Some(scrap_quantity) = wo_pos.quantity_scrap else { 
        return data_error(format!("ItemCode is null"));
    };

    Ok(scrap_quantity)
}

fn get_qcorder_measurement(client: &Client, doc_entry: i32, line_number: i32) -> Result<Option<QCOrderMeasurement>, TransactionError>
{
    let filter = 
        FilterBuilder::new()
        .equals("WoDocEntry", doc_entry).and()
        .equals("WoLineNumber", line_number).and()
        .equals("LineNumber2", 10).and()
        .equals("QCDescription", "QiTech_Gewicht")
        .build();
    
    let options = QueryOptions::new().filter(filter);
    
    let result = 
        client
        .single_request()
        .quality_control()
        .qcorder_measurement()
        .get(options);
            
    match result
    {
        Ok(workorders) => Ok(workorders.first().cloned()),
        Err(e) => Err(e),
    }
}

pub fn post_backflush(client: &Client, data: BackflushRequest) -> Result<(), ResponseError>
{
    let result = 
        client
        .single_request()
        .production()
        .backflush()
        .post(data);
            
    match result
    {
        Ok(()) => Ok(()),
        Err(e) => Err(ResponseError::ClientTransaction(e)),
    }
}

fn unpack_nullable<T>(value: Option<T>, name: &'static str) -> Result<T, ResponseError>
{
    match value
    {
        Some(item) => Ok(item),
        None       => return data_error(format!("Received null for {}", name)),
    }
}

fn data_error<T, M>(msg: M) -> Result<T, ResponseError>
where
    M: Into<String>,
{
    Err(ResponseError::InvalidData(msg.into()))
}