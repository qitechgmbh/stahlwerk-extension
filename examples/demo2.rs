use beas_bsl::{Client, ClientConfig, api::{Date, time_receipt}};


pub fn main()
{
    let config = ClientConfig::from_file("config.json").expect("Where config?");
    
    let client = Client::new(config).expect("Why no client?");

    let request = time_receipt::post::Request {
        doc_entry:          54366,
        line_number:        10,
        line_number2:       10,
        line_number3:       Some(0),
        time_type:          Some("A".to_string()),
        resource_id:        Some("FF01".to_string()),
        quantity_good:      Some(12.0),
        personnel_id:       "04711".to_string(),
        quantity_scrap:     Some(0.0),
        start_date:         Some(Date { year: 2026, month: 03, day: 12 }),
        end_date:           Some(Date { year: 2026, month: 03, day: 12 }),
        from_time:          Some("12:00".to_string()),
        to_time:            Some("14:30".to_string()),
        close_entry:        Some(true),
        manual_booking:     Some(false),
        duration:           Some(60),
        calculate_duration: Some(false),
        remarks:            Some("Hello World".to_string()),
        ..Default::default()
    };

    let response = 
        client
        .single_request()
        .production()
        .time_receipt()
        .post(request);

    println!("response: {:?}", response);

    // loop {}
}