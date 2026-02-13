use beas_bsl::{ ClientConfig, ClientError };
use stahlwerk_extension::ff01::{ProxyClient};

// Example for happy path
pub fn main() -> Result<(), ClientError>
{
    let config = ClientConfig::from_file("config.json").unwrap();
    
    let mut proxy = ProxyClient::new(config).unwrap();
    
    loop 
    {
        // user submits new workorder
        
        // we detect the next order/entry
        let entry =  proxy.get_next_entry().unwrap();
    
        // Machine starts counting/measuring
        loop 
        {
            // user submitted scrap quantity
            
            // poll for scrap quantity
            let scrap_quantity = proxy.get_scrap_quantity(&entry).unwrap();
            
            // detect change in scrap quantity
            if scrap_quantity != entry.scrap_quantity
            {
                // Machine stops counting/measuring
                break;
            }
        }
        
        // submit data via finalize
        proxy.finalize().unwrap();   
    }
}