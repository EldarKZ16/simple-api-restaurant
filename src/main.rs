use simple_api_restaurant::{AddOrderRequest, InMemoryOrderRepository, OrderError, OrderErrorRejection, OrderService};
use std::{collections::HashMap, error::Error, sync::Arc};
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter};

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
        .and_then(add_order_handler);
    let get_order_by_id = orders_path
        .and(warp::get())
        .and(warp::path::param::<usize>())
        .and(warp::path::end())
        .and(order_service_filter.clone())
        .and_then(get_order_handler);
    let get_orders_by_table_number = orders_path
        .and(warp::get())
        .and(warp::query::<HashMap<String, i32>>())
        .and(warp::path::end())
        .and(order_service_filter.clone())
        .and_then(get_orders_by_table_number_handler);
    let remove_order = orders_path
        .and(warp::delete())
        .and(warp::path::param::<usize>())
        .and(warp::path::end())
        .and(order_service_filter)
        .and_then(remove_order_handler);
    let orders_route = add_order
        .or(get_order_by_id)
        .or(get_orders_by_table_number)
        .or(remove_order);
    //

    let routes = healthcheck_route
        .or(orders_route)
        .recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn add_order_handler(
    order_service: Arc<OrderService>,
    new_order: AddOrderRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    order_service.add_order(new_order.table_number, new_order.menu_item, new_order.quantity)
        .map(|_| StatusCode::CREATED)
        .map_err(|e| e.reject())
}

async fn get_order_handler(
    order_id: usize,
    order_service: Arc<OrderService>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    match order_service.get_order(order_id) {
        Ok(order_view) => Ok(Box::new(warp::reply::json(&order_view))),
        Err(_) => Ok(Box::new(StatusCode::NOT_FOUND)),
    }
}

async fn get_orders_by_table_number_handler(
    params: HashMap<String, i32>,
    order_service: Arc<OrderService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(table_number) = params.get("table_number") {
        match order_service.get_remaining_orders_by_table_number(*table_number) {
            Ok(orders) => Ok(warp::reply::json(&orders)),
            Err(_) => Err(warp::reject::not_found()),
        }
    } else {
        Err(warp::reject::not_found())
    }
}

async fn remove_order_handler(
    order_id: usize,
    order_service: Arc<OrderService>
) -> Result<impl warp::Reply, warp::Rejection> {
    order_service.remove_order(order_id)
        .map(|_| StatusCode::OK)
        .map_err(|e| e.reject())
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(OrderErrorRejection { err }) = err.find() {
        let (code, message) = match err {
            OrderError::NotFound => (StatusCode::NOT_FOUND, "Order not found"),
            OrderError::LockFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            OrderError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            OrderError::ValidationFailed(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            OrderError::ParseFailed(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        };

        let json = warp::reply::json(&HashMap::from([("message", message)]));
        Ok(warp::reply::with_status(json, code))
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        let message = match e.source() {
            Some(cause) => cause.to_string(),
            None => "BAD_REQUEST".to_string(),
        };
        let json = warp::reply::json(&HashMap::from([("message", &message)]));
        Ok(warp::reply::with_status(json, StatusCode::BAD_REQUEST))
    } else {
        let json = warp::reply::json(&HashMap::from([("message", "unhandled exception")]));
        Ok(warp::reply::with_status(json, StatusCode::INTERNAL_SERVER_ERROR))
    }
}