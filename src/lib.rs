use std::collections::HashMap;
use std::error;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Order {
    id: usize,
    table_number: i32,
    item: String,
    quantity: u8,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>
}

impl Order {
    pub fn new(id: usize, table_number: i32, item: String, quantity: u8, created_at: DateTime<Utc>, finished_at: DateTime<Utc>) -> Self {
        Self {
            id,
            table_number,
            item,
            quantity,
            created_at,
            finished_at,
        }
    }
}

#[derive(Debug)]
pub enum OrderError {
    NotFound,
    LockFailed(String),
    DatabaseError(String),
}

impl Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match *self {
           OrderError::NotFound => write!(f, "Not found"),
           OrderError::LockFailed(ref msg) => write!(f, "Lock failed, reason: {}", msg),
           OrderError::DatabaseError(ref msg) => write!(f, "Database error, reason: {}", msg),
       }
    }
}

impl error::Error for OrderError {}

pub trait OrderRepository {
    fn add(&self, table_number: i32, item: String, quantity: u8) -> Result<(), OrderError>;
    fn remove(&self, table_number: i32, order_id: usize) -> Result<(), OrderError>;
    fn get_by_table_number(&self, table_number: i32) -> Result<Vec<Order>, OrderError>;
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
    fn add(&self, table_number: i32, item: String, quantity: u8) -> Result<(), OrderError> {
        let id = self.generate_order_id()?;
        let created_at = Utc::now();
        let cooking_time = rand::thread_rng().gen_range(5..=15);
        let finished_at = created_at + chrono::Duration::minutes(cooking_time as i64);
        let order = Order::new(id, table_number, item, quantity, created_at, finished_at);

        let mut orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        orders.entry(table_number).or_insert_with(Vec::new).push(order);
        Ok(())
    }

    fn remove(&self, table_number: i32, order_id: usize) -> Result<(), OrderError> {
        let mut orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        if let Some(orders_for_table) = orders.get_mut(&table_number) {
            orders_for_table.retain(|order| order.id != order_id);
        }
        Ok(())
    }

    fn get_by_table_number(&self, table_number: i32) -> Result<Vec<Order>, OrderError> {
        let orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        match orders.get(&table_number) {
            Some(orders_for_table) => Ok(orders_for_table.clone()),
            None => Ok(Vec::new()),
        }
    }

    fn get(&self, order_id: usize) -> Result<Order, OrderError> {
        let orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        for (_, orders_for_table) in orders.iter() {
            if let Some(order) = orders_for_table.iter().find(|&order| order.id == order_id) {
                return Ok(order.clone());
            }
        }
        Err(OrderError::NotFound)
    }
}