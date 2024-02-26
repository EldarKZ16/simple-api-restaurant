use std::collections::HashMap;
use std::error;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use warp::reject::Reject;

#[derive(Debug, Clone)]
pub struct Order {
    id: usize,
    table_number: i32,
    menu_item: String,
    quantity: u8,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>
}

impl Order {
    fn new(id: usize, table_number: i32, menu_item: String, quantity: u8, created_at: DateTime<Utc>, finished_at: DateTime<Utc>) -> Self {
        Self {
            id,
            table_number,
            menu_item,
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
    ValidationFailed(String),
    ParseFailed(String)
}

impl Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match *self {
           OrderError::NotFound => write!(f, "Not found"),
           OrderError::LockFailed(ref msg) => write!(f, "Lock failed, reason: {}", msg),
           OrderError::DatabaseError(ref msg) => write!(f, "Database error, reason: {}", msg),
           OrderError::ValidationFailed(ref msg) => write!(f, "Validation failed, reason: {}", msg),
           OrderError::ParseFailed(ref msg) => write!(f, "Parsing failed, reason: {}", msg),
       }
    }
}

impl error::Error for OrderError {}

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

    // fixme
    fn remove(&self, order_id: usize) -> Result<(), OrderError> {
        let mut orders = self.orders.lock()
            .map_err(|e| OrderError::LockFailed(e.to_string()))?;
        if let Some(orders_for_table) = orders.get_mut(&1) {
            orders_for_table.retain(|order| order.id != order_id);
        }
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

#[derive(Debug, Serialize)]
pub struct OrderView {
    id: usize,
    table_number: i32,
    menu_item: String,
    quantity: u8,
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

#[derive(Serialize, Deserialize)]
pub struct AddOrderRequest {
    pub table_number: i32,
    pub menu_item: String,
    pub quantity: u8
}

#[derive(Debug)]
pub struct OrderErrorRejection {
    pub err: OrderError,
}

impl Reject for OrderErrorRejection {}

impl OrderError {
    pub fn reject(self) -> warp::Rejection {
        warp::reject::custom(OrderErrorRejection { err: self })
    }
}