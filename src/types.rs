use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WeightedItem 
{
    pub code:     String,
    pub name:     String,
    pub weight:   f32, // weight is in kilo
    pub quantity: u32,
}