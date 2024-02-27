use std::sync::Arc;

use crate::{
    model::{
        error::OrderError, 
        view::OrderView
    }, 
    repository::OrderRepository
};
pub struct OrderService {
    repository: Arc<dyn OrderRepository>,
}

impl OrderService {
    pub fn new(repository: Arc<dyn OrderRepository>) -> Self {
        Self { repository }
    }

    pub fn add_order(&self, table_number: i32, menu_item: String, quantity: u8) -> Result<(), OrderError> {
        if table_number <= 0 {
            return Err(OrderError::ValidationFailed("Invalid table number".to_string()));
        }
        if menu_item.trim().is_empty() {
            return Err(OrderError::ValidationFailed("Menu item cannot be empty".to_string()));
        }
        if quantity == 0 {
            return Err(OrderError::ValidationFailed("Quantity must be greater than 0".to_string()));
        }
        self.repository.add(table_number, menu_item, quantity)
    }

    pub fn remove_order(&self, order_id: usize) -> Result<(), OrderError> {
        self.repository.remove(order_id)
    }

    pub fn get_remaining_orders_by_table_number(&self, table_number: i32) -> Result<Vec<OrderView>, OrderError> {
        let orders = self.repository.get_remaining_by_table_number(table_number)?;
        Ok(orders.into_iter().map(|order| OrderView::from_order(&order)).collect())
    }

    pub fn get_order(&self, order_id: usize) -> Result<OrderView, OrderError> {
        let order = self.repository.get(order_id)?;
        Ok(OrderView::from_order(&order))
    }
}