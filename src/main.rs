mod model;
mod service;
mod repository;
mod handler;

use std::{collections::HashMap, sync::Arc};
use repository::InMemoryOrderRepository;
use service::OrderService;
use warp::{http::StatusCode, Filter}; 

#[tokio::main]
async fn main() {
    let in_memory_repo = Arc::new(InMemoryOrderRepository::new());
    let order_service = Arc::new(OrderService::new(in_memory_repo));
    let order_service_filter = warp::any().map(move || Arc::clone(&order_service));

    let healthcheck_route = warp::path("healthcheck").map(|| StatusCode::OK);
    let v1_path = warp::path("v1");

    // Orders API
    let orders_path = v1_path.and(warp::path("orders"));
    let add_order = orders_path
        .and(warp::post())
        .and(warp::path::end())
        .and(order_service_filter.clone())
        .and(warp::body::json())
        .and_then(handler::add_order_handler);
    let get_order_by_id = orders_path
        .and(warp::get())
        .and(warp::path::param::<usize>())
        .and(warp::path::end())
        .and(order_service_filter.clone())
        .and_then(handler::get_order_handler);
    let get_orders_by_table_number = orders_path
        .and(warp::get())
        .and(warp::query::<HashMap<String, i32>>())
        .and(warp::path::end())
        .and(order_service_filter.clone())
        .and_then(handler::get_orders_by_table_number_handler);
    let remove_order = orders_path
        .and(warp::delete())
        .and(warp::path::param::<usize>())
        .and(warp::path::end())
        .and(order_service_filter)
        .and_then(handler::remove_order_handler);
    let orders_route = add_order
        .or(get_order_by_id)
        .or(get_orders_by_table_number)
        .or(remove_order);
    //

    let routes = healthcheck_route
        .or(orders_route)
        .recover(handler::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
