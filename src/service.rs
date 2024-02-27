use std::sync::Arc;
use chrono::Utc;
use rand::Rng;

use crate::{
    model::{
        domain::Order, error::GeneralError, view::OrderView
    }, 
    repository::Repository
};
pub struct OrderService {
    repository: Arc<dyn Repository<Order>>,
}

impl OrderService {
    pub fn new(repository: Arc<dyn Repository<Order>>) -> Self {
        Self { repository }
    }

    pub fn add_order(&self, table_number: i32, menu_item: String, quantity: u8) -> Result<(), GeneralError> {
        if table_number <= 0 {
            return Err(GeneralError::ValidationFailed("Invalid table number".to_string()));
        }
        if menu_item.trim().is_empty() {
            return Err(GeneralError::ValidationFailed("Menu item cannot be empty".to_string()));
        }
        if quantity == 0 {
            return Err(GeneralError::ValidationFailed("Quantity must be greater than 0".to_string()));
        }
        let created_at = Utc::now();
        let cooking_time = rand::thread_rng().gen_range(5..=15);
        let finished_at = created_at + chrono::Duration::minutes(cooking_time as i64);
        let order = Order::new(table_number, menu_item, quantity, created_at, finished_at);
        self.repository.add(order)
    }

    pub fn remove_order(&self, order_id: usize) -> Result<(), GeneralError> {
        self.repository.remove(order_id)
    }

    pub fn get_remaining_orders_by_table_number(&self, table_number: i32) -> Result<Vec<OrderView>, GeneralError> {
        let orders = self.repository.list()?;
        let order_by_table_number = orders.into_iter()
            .filter(|order| order.table_number == table_number)
            .map(|order| OrderView::from_order(&order))
            .collect();
        Ok(order_by_table_number)
    }

    pub fn get_order(&self, order_id: usize) -> Result<OrderView, GeneralError> {
        let order = self.repository.get(order_id)?;
        Ok(OrderView::from_order(&order))
    }
}