use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Order {
    pub id: usize,
    pub table_number: i32,
    pub menu_item: String,
    pub quantity: u8,
    pub created_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>
}

impl Order {
    pub fn new(table_number: i32, menu_item: String, quantity: u8, created_at: DateTime<Utc>, finished_at: DateTime<Utc>) -> Self {
        Self {
            // id auto-generated by repository
            id: 1,
            table_number,
            menu_item,
            quantity,
            created_at,
            finished_at,
        }
    }
}