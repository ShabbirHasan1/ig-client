use crate::error::AppError;
use crate::model::requests::{
    ClosePositionRequest, CreateOrderRequest, CreateWorkingOrderRequest, UpdatePositionRequest,
};
use crate::model::responses::{
    ClosePositionResponse, CreateOrderResponse, CreateWorkingOrderResponse,
    OrderConfirmationResponse, UpdatePositionResponse,
};

use async_trait::async_trait;

#[async_trait]
/// Service for creating, updating, and managing trading orders with the IG Markets API
///
/// This trait defines the interface for interacting with the IG Markets order endpoints,
/// allowing clients to create new orders, get order confirmations, update existing positions,
/// and close positions.
pub trait OrderService: Send + Sync {
    /// Creates a new order
    async fn create_order(
        &self,

        order: &CreateOrderRequest,
    ) -> Result<CreateOrderResponse, AppError>;

    /// Gets the confirmation of an order
    async fn get_order_confirmation(
        &self,

        deal_reference: &str,
    ) -> Result<OrderConfirmationResponse, AppError>;

    /// Updates an existing position
    async fn update_position(
        &self,

        deal_id: &str,
        update: &UpdatePositionRequest,
    ) -> Result<UpdatePositionResponse, AppError>;

    /// Closes an existing position
    async fn close_position(
        &self,

        close_request: &ClosePositionRequest,
    ) -> Result<ClosePositionResponse, AppError>;

    /// Creates a new working order
    async fn create_working_order(
        &self,

        order: &CreateWorkingOrderRequest,
    ) -> Result<CreateWorkingOrderResponse, AppError>;
}
