use std::time::Duration;

use beas_bsl::{ ClientConfig, ClientError };
use stahlwerk_extension::ff01::{Entry, ProxyClient, TransactionError};


pub fn main() -> Result<(), ClientError>
{
    println!("start: ");
    
    let config = ClientConfig::from_file("config.json");
    println!("config: {:?}", config);
    
    let config = config.unwrap();
    
    let proxy = ProxyClient::new(config);
    println!("proxy: {:?}", proxy);
    
    let mut proxy = proxy.unwrap();
    
    let entry = get_next_entry(&mut proxy);
    
    println!("entry: {:?}", entry);
    
    let entry = entry.unwrap();
    
    _ = entry;
    
    Ok(())
}

fn get_next_entry(proxy: &mut ProxyClient) -> Result<Entry, TransactionError>
{
    let mut attempts: u32 = 0;
    
    loop 
    {
        if let Some(entry) = proxy.get_next_entry()?
        {
            return Ok(entry);
        }
        
        if attempts > 100
        {
            panic!("Exceeded attempts");
        }
        
        attempts += 1;
        std::thread::sleep(Duration::from_millis(25));
    }
}