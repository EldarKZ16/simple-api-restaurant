use serde::Serialize;
use chrono::Utc;
use crate::model::domain::Order;

#[derive(Serialize)]
pub struct OrderView {
    id: usize,
    pub table_number: i32,
    pub menu_item: String,
    pub quantity: u8,
    time_to_cook: i64
}

impl OrderView {
    pub fn from_order(order: &Order) -> Self {
        let now = Utc::now();
        let time_to_cook = (order.finished_at - now).num_minutes().max(0);
        Self {
            id: order.id,
            table_number: order.table_number,
            menu_item: order.menu_item.clone(),
            quantity: order.quantity,
            time_to_cook,
        }
    }
}

