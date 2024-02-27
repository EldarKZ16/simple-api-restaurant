use std::{collections::HashMap, error::Error, sync::Arc};

use crate::{model::{error::{OrderError, OrderErrorRejection}, http::AddOrderRequest}, service::OrderService};
use warp::{http::StatusCode, reject::Rejection, reply::Reply};

pub async fn add_order_handler(
    order_service: Arc<OrderService>,
    new_order: AddOrderRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    order_service.add_order(new_order.table_number, new_order.menu_item, new_order.quantity)
        .map(|_| StatusCode::OK)
        .map_err(|err| err.reject())
}

pub async fn get_order_handler(
    order_id: usize,
    order_service: Arc<OrderService>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    match order_service.get_order(order_id) {
        Ok(order_view) => Ok(Box::new(warp::reply::json(&order_view))),
        Err(err) => Err(err.reject()),
    }
}

pub async fn get_orders_by_table_number_handler(
    params: HashMap<String, i32>,
    order_service: Arc<OrderService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(table_number) = params.get("table_number") {
        match order_service.get_remaining_orders_by_table_number(*table_number) {
            Ok(orders) => Ok(warp::reply::json(&orders)),
            Err(err) => Err(err.reject()),
        }
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn remove_order_handler(
    order_id: usize,
    order_service: Arc<OrderService>
) -> Result<impl warp::Reply, warp::Rejection> {
    order_service.remove_order(order_id)
        .map(|_| StatusCode::OK)
        .map_err(|err| err.reject())
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(OrderErrorRejection { err }) = err.find() {
        let (code, message) = match err {
            OrderError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            OrderError::LockFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            OrderError::ValidationFailed(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
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