// use anyhow::Error;
use cqrs_es::persist::PersistenceError;
use cqrs_es::AggregateError;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum IndexDbAggregateError {
    OptimisticLock,
    ConnectionError(String),
    DeserializationError(String),
    UnknownError(String),
}

impl Display for IndexDbAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexDbAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            IndexDbAggregateError::UnknownError(error) => write!(f, "{}", error),
            IndexDbAggregateError::DeserializationError(error) => write!(f, "{}", error),
            IndexDbAggregateError::ConnectionError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for IndexDbAggregateError {}

// impl From<DomException> for IndexDbAggregateError {
//     fn from(err: DomException) -> Self {
//         println!("error: {:?}", err);
//         // TODO: improve error handling
//         match err {
//             _ => IndexDbAggregateError::UnknownError(err.message()),
//         }
//     }
// }

impl<T: std::error::Error> From<IndexDbAggregateError> for AggregateError<T> {
    fn from(err: IndexDbAggregateError) -> Self {
        match err {
            IndexDbAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            IndexDbAggregateError::ConnectionError(_) => {
                AggregateError::DatabaseConnectionError(Box::new(err))
            }
            IndexDbAggregateError::DeserializationError(_) => {
                AggregateError::DeserializationError(Box::new(err))
            }
            IndexDbAggregateError::UnknownError(_) => {
                AggregateError::UnexpectedError(Box::new(err))
            }
        }
    }
}

impl From<serde_json::Error> for IndexDbAggregateError {
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                IndexDbAggregateError::DeserializationError(err.to_string())
            }
            serde_json::error::Category::Io | serde_json::error::Category::Eof => {
                IndexDbAggregateError::UnknownError(err.to_string())
            }
        }
    }
}

impl From<IndexDbAggregateError> for PersistenceError {
    fn from(err: IndexDbAggregateError) -> Self {
        match err {
            IndexDbAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            IndexDbAggregateError::ConnectionError(_) => {
                PersistenceError::ConnectionError(Box::new(err))
            }
            IndexDbAggregateError::DeserializationError(_) => {
                PersistenceError::UnknownError(Box::new(err))
            }
            IndexDbAggregateError::UnknownError(_) => PersistenceError::UnknownError(Box::new(err)),
        }
    }
}
