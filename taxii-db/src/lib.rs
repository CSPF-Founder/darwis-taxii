//! Database layer for TAXII.
//!
//! # Architecture
//!
//! This crate provides database access for both TAXII 1.x and TAXII 2.x:
//!
//! - **Models**: Database entities with static CRUD methods using sqlx macros
//! - **Repository**: Trait-based database access layer for mockability
//! - **DatabaseManager**: Connection pool lifecycle management
//! - **TaxiiPool**: Type-safe pool wrapper for database operations

pub mod error;
pub mod manager;
pub mod migrations;
pub mod models;
pub mod pool;
pub mod repository;

// Core types
pub use error::{DatabaseError, DatabaseResult};
pub use manager::DatabaseManager;
pub use pool::{PoolOptions, TaxiiPool};

// Auth models
pub use models::account::{Account, TAXII1_PERMISSIONS, TAXII2_PERMISSIONS, validate_permissions};

// TAXII 1.x models
pub use models::taxii1::{
    ContentBindingFilter, ContentBlock, ContentBlockFilter, DataCollection, InboxMessage,
    NewInboxMessage, ResultSet, Service, Subscription, UpdateDataCollection,
};

// TAXII 2.x models
pub use models::taxii2::{
    ApiRoot, Collection, FilteredResult, Job, JobDetail, NewJob, NewSTIXObject, PaginatedResult,
    PaginationCursor, STIXObject, Taxii2QueryParams, VersionInfo, VersionsResult, get_next_param,
    parse_next_param,
};

// Repository traits and implementations
pub use repository::{
    DbTaxii1Repository, DbTaxii2Repository, Taxii1Repository, Taxii2Repository, get_object_version,
};
