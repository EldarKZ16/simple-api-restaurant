use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use rand::Rng;

use crate::model::{
    domain::Order, 
    error::OrderError
};
pub trait OrderRepository: Send + Sync {
    fn add(&self, table_number: i32, menu_item: String, quantity: u8) -> Result<(), OrderError>;
    fn remove(&self, order_id: usize) -> Result<(), OrderError>;
    fn get_remaining_by_table_number(&self, table_number: i32) -> Result<Vec<Order>, OrderError>;
    fn get(&self, order_id: usize) -> Result<Order, OrderError>;
}

pub struct InMemoryOrderRepository {
    orders: Arc<Mutex<HashMap<i32, Vec<Order>>>>,
    order_id_counter: Arc<Mutex<usize>>,
}

impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
            order_id_counter: Arc::new(Mutex::new(0)),
        }
    }

    fn generate_order_id(&self) -> Result<usize, OrderError> {
        let mut id = self.order_id_counter.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        *id += 1;
        Ok(*id)
    }
}

impl OrderRepository for InMemoryOrderRepository {
    fn add(&self, table_number: i32, menu_item: String, quantity: u8) -> Result<(), OrderError> {
        let id = self.generate_order_id()?;
        let created_at = Utc::now();
        let cooking_time = rand::thread_rng().gen_range(5..=15);
        let finished_at = created_at + chrono::Duration::minutes(cooking_time as i64);
        let order = Order::new(id, table_number, menu_item, quantity, created_at, finished_at);

        let mut orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        orders.entry(table_number).or_insert_with(Vec::new).push(order);
        Ok(())
    }

    fn remove(&self, order_id: usize) -> Result<(), OrderError> {
        let mut orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        orders.values_mut().for_each(|order| {
            order.retain(|order| order.id != order_id);
        });
        Ok(())
    }

    fn get_remaining_by_table_number(&self, table_number: i32) -> Result<Vec<Order>, OrderError> {
        let orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        Ok(orders.get(&table_number)
            .cloned()
            .unwrap_or_else(|| Vec::new())
            .into_iter()
            .filter(|order| order.finished_at > Utc::now())
            .collect())
    }

    fn get(&self, order_id: usize) -> Result<Order, OrderError> {
        let orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        let order = orders.iter()
            .map(|(_, order)| order)
            .flatten()
            .find(|&order| order.id == order_id);
        if let Some(order) = order {
            return Ok(order.clone());
        }
        Err(OrderError::NotFound)
    }
}