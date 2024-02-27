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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::view::OrderView,
        repository::InMemoryRepository
    };
    use std::sync::Arc;

    fn create_order_service() -> OrderService {
        let in_memory_repo = Arc::new(InMemoryRepository::new());
        OrderService::new(in_memory_repo)
    }

    #[test]
    fn add_and_get_order_success() {
        let order_service = create_order_service();
        let result = order_service.add_order(1, "Ramen".to_string(), 2);
        assert!(result.is_ok());

        let order_view: OrderView = order_service.get_order(1).unwrap();
        assert_eq!(order_view.menu_item, "Ramen");
        assert_eq!(order_view.quantity, 2);
    }

    #[test]
    fn add_orders_and_get_order_success() {
        let order_service = create_order_service();
        order_service.add_order(3, "Ramen".to_string(), 2).unwrap();
        order_service.add_order(2, "Udon".to_string(), 3).unwrap();
        order_service.add_order(1, "Soba".to_string(), 4).unwrap();

        let order_view: OrderView = order_service.get_order(3).unwrap();
        assert_eq!(order_view.table_number, 1);
        assert_eq!(order_view.menu_item, "Soba");
        assert_eq!(order_view.quantity, 4);
    }

    #[test]
    fn add_order_validation_failure() {
        let order_service = create_order_service();
        assert!(order_service.add_order(0, "Sushi".to_string(), 2).is_err());
        assert!(order_service.add_order(1, "".to_string(), 2).is_err());
        assert!(order_service.add_order(1, "Sushi".to_string(), 0).is_err());
    }

    #[test]
    fn remove_order_success() {
        let order_service = create_order_service();
        let add_result = order_service.add_order(1, "Ramen".to_string(), 1);
        assert!(add_result.is_ok());

        let remove_result = order_service.remove_order(1);
        assert!(remove_result.is_ok());

        assert!(order_service.get_order(1).is_err())
    }

    #[test]
    fn get_remaining_orders_by_table_number() {
        let order_service = create_order_service();
        order_service.add_order(1, "Sushi".to_string(), 2).unwrap();
        order_service.add_order(1, "Ramen".to_string(), 1).unwrap();
        order_service.add_order(1, "Udon".to_string(), 4).unwrap();
        order_service.add_order(2, "Soba".to_string(), 3).unwrap();

        let orders = order_service.get_remaining_orders_by_table_number(1).unwrap();
        assert_eq!(orders.len(), 3);
    }

    #[test]
    fn get_remaining_orders_by_table_number_empty() {
        let order_service = create_order_service();
        order_service.add_order(1, "Sushi".to_string(), 2).unwrap();

        let orders = order_service.get_remaining_orders_by_table_number(2).unwrap();
        assert_eq!(orders.len(), 0);
    }
}