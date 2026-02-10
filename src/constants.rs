use std::collections::HashMap;
use std::sync::OnceLock;

static MAP: OnceLock<HashMap<&'static str, f64>> = OnceLock::new();

pub fn article_weights() -> &'static HashMap<&'static str, f64>
{
    MAP.get_or_init(|| 
    {
        HashMap::from([
            ("ZURO-20160", 9.85),
            ("ZURO-20161", 11.00),
            ("ZURO-20162", 12.25),
            ("ZURO-20163", 14.75),
            ("ZURO-20164", 12.45),
            ("ZURO-20165", 12.76),
            ("ZURO-20166", 10.82),
            ("ZURO-20167", 12.20),
            ("ZURO-20168", 12.70),
            ("ZURO-20169", 13.50),
            ("ZURO-20170", 9.70),
            ("ZURO-20171", 10.80),
            ("ZURO-20183", 11.80),
            ("ZURO-20188", 12.20),
            ("ZURO-20190", 12.25),
            ("ZURO-20200", 9.85),
            ("ZURO-20209", 11.85),
            ("ZURO-20210", 12.18),
        ])
    })
}