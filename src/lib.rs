use std::collections::HashMap;
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

pub trait OrderRepository {
    fn add(&self, table_number: i32, item: String, quantity: u8);
    fn remove(&self, table_number: i32, order_id: usize);
    fn show_by_table_number(&self, table_number: i32) -> Vec<Order> ;
    fn show(&self, order_id: usize) -> Option<Order>;
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

    fn generate_order_id(&self) -> usize {
        let mut id = self.order_id_counter.lock().unwrap();
        *id += 1;
        *id
    }
}

impl OrderRepository for InMemoryOrderRepository {
    fn add(&self, table_number: i32, item: String, quantity: u8) {
        let id = self.generate_order_id();
        let created_at = Utc::now();
        let cooking_time = rand::thread_rng().gen_range(5..=15);
        let finished_at = created_at + chrono::Duration::minutes(cooking_time as i64);
        let order = Order::new(id, table_number, item, quantity, created_at, finished_at);

        let mut orders = self.orders.lock().unwrap();
        orders.entry(table_number).or_insert_with(Vec::new).push(order);
    }

    fn remove(&self, table_number: i32, order_id: usize) {
        let mut orders = self.orders.lock().unwrap();
        if let Some(orders_for_table) = orders.get_mut(&table_number) {
            orders_for_table.retain(|order| order.id != order_id);
        }
    }

    fn show_by_table_number(&self, table_number: i32) -> Vec<Order>  {
        let orders = self.orders.lock().unwrap();
        match orders.get(&table_number) {
            Some(orders_for_table) => orders_for_table.clone(),
            None => Vec::new(),
        }
    }

    fn show(&self, order_id: usize) -> Option<Order> {
        let orders = self.orders.lock().unwrap();
        for (_, orders_for_table) in orders.iter() {
            if let Some(order) = orders_for_table.iter().find(|&order| order.id == order_id) {
                return Some(order.clone());
            }
        }
        None
    }
}