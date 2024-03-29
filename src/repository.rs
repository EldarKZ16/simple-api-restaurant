use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::model::{
    domain::Order, 
    error::GeneralError
};

pub trait Repository<T>: Send + Sync {
    fn add(&self, entity: T) -> Result<(), GeneralError>;
    fn remove(&self, id: usize) -> Result<(), GeneralError>;
    fn list(&self) -> Result<Vec<T>, GeneralError>;
    fn get(&self, id: usize) -> Result<T, GeneralError>;
}

pub struct InMemoryRepository<T> {
    orders: Arc<Mutex<HashMap<usize, T>>>,
    id_counter: Arc<Mutex<usize>>,
}

impl<T> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
            id_counter: Arc::new(Mutex::new(0)),
        }
    }

    fn generate_id(&self) -> Result<usize, GeneralError> {
        let mut id = self.id_counter.lock()
            .map_err(|e| GeneralError::LockFailed(e.to_string()))?;
        *id += 1;
        Ok(*id)
    }
}

impl Repository<Order> for InMemoryRepository<Order> {
    fn add(&self, entity: Order) -> Result<(), GeneralError> {
        let id = self.generate_id()?;
        let order = Order {
            id,
            ..entity
        };
        let mut orders = self.orders.lock()
            .map_err(|e| GeneralError::LockFailed(e.to_string()))?;
        orders.entry(id).or_insert_with(|| order);
        Ok(())
    }

    fn remove(&self, order_id: usize) -> Result<(), GeneralError> {
        let mut orders = self.orders.lock()
            .map_err(|e| GeneralError::LockFailed(e.to_string()))?;
        orders.remove_entry(&order_id);
        Ok(())
    }

    fn list(&self) -> Result<Vec<Order>, GeneralError> {
        let orders = self.orders.lock()
            .map_err(|e| GeneralError::LockFailed(e.to_string()))?;
        Ok(orders.clone().into_values().collect())
    }

    fn get(&self, order_id: usize) -> Result<Order, GeneralError> {
        let orders = self.orders.lock()
            .map_err(|e| GeneralError::LockFailed(e.to_string()))?;
        let order = orders.clone()
            .into_iter()
            .find(|order| order.0 == order_id)
            .map(|order| order.1);
        if let Some(order) = order {
            return Ok(order.clone());
        }
        Err(GeneralError::NotFound)
    }
}