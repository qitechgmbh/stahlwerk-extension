use std::time::Duration;

use beas_bsl::ClientConfig;
use stahlwerk_extension::{ProxyClient, TransactionError};

pub fn main()
{
    use TransactionError::Pending;

    let config = ClientConfig::from_file("config.json").unwrap();
    
    let mut proxy = ProxyClient::new(config).unwrap();
    
    let entry = loop {
        let entry = match proxy.get_next_entry()
        {
            Ok(v) => v,
            Err(e) => match e
            {
                Pending => continue,
                _ => panic!("Oh no bro: {:?}", e),
            },
        };

        break entry;
    };

    println!("Entry: {:?}", entry);

    let quantity_scrap = loop {
        let scrap_quantity = match proxy.get_quantity_scrap(&entry)
        {
            Ok(v) => v,
            Err(e) => match e
            {
                Pending => continue,
                _ => panic!("Oh no bro: {:?}", e),
            },
        };

        println!("received scrap_quantity: {:?}", scrap_quantity);

        if scrap_quantity != entry.scrap_quantity
        {
            break scrap_quantity;
        }

        std::thread::sleep(Duration::from_millis(2000));
    };

    loop
    {
        match proxy.finalize(&entry, quantity_scrap as f32, 145.0)
        {
            Ok(_) => break,
            Err(e) => match e
            {
                Pending => continue,
                _ => panic!("Oh no bro: {:?}", e),
            },
        };
    }

    println!("Request completed successfully");
}