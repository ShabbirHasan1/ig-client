use crate::application::models::account::WorkingOrders;
use crate::application::models::order::{
    ClosePositionRequest, ClosePositionResponse, CreateOrderRequest, CreateOrderResponse,
    OrderConfirmation, UpdatePositionRequest, UpdatePositionResponse,
};
use crate::application::models::working_order::{
    CreateWorkingOrderRequest, CreateWorkingOrderResponse,
};
use crate::error::AppError;
use crate::session::interface::IgSession;
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
        session: &IgSession,
        order: &CreateOrderRequest,
    ) -> Result<CreateOrderResponse, AppError>;

    /// Gets the confirmation of an order
    async fn get_order_confirmation(
        &self,
        session: &IgSession,
        deal_reference: &str,
    ) -> Result<OrderConfirmation, AppError>;

    /// Updates an existing position
    async fn update_position(
        &self,
        session: &IgSession,
        deal_id: &str,
        update: &UpdatePositionRequest,
    ) -> Result<UpdatePositionResponse, AppError>;

    /// Closes an existing position
    async fn close_position(
        &self,
        session: &IgSession,
        close_request: &ClosePositionRequest,
    ) -> Result<ClosePositionResponse, AppError>;

    /// Gets all working orders
    async fn get_working_orders(&self, session: &IgSession) -> Result<WorkingOrders, AppError>;

    /// Creates a new working order
    async fn create_working_order(
        &self,
        session: &IgSession,
        order: &CreateWorkingOrderRequest,
    ) -> Result<CreateWorkingOrderResponse, AppError>;
}
