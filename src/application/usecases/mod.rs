use crate::infrastructure::repositories::Repositories;
use std::sync::Arc;

pub mod interface;

pub struct UseCases {}

impl UseCases {
    pub fn new(_repositories: Arc<Repositories>) -> Self {
        Self {}
    }
}
