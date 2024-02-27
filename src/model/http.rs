use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AddOrderRequest {
    pub table_number: i32,
    pub menu_item: String,
    pub quantity: u8
}