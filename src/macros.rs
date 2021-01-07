#[macro_export]
macro_rules! wrap_err {
    ($from:ty, $to:ty) => {
        impl From<$from> for $to {
            fn from(e: $from) -> Self {
                Self(e)
            }
        }
    };
}

#[macro_export]
macro_rules! inc_sql {
    ($e:expr) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sql/", $e, ".sql"))
    };
}

#[macro_export]
macro_rules! generic_handler_err {
    ($ty:ty, $err: expr) => {
        impl From<tokio_postgres::Error> for $ty {
            fn from(e: tokio_postgres::Error) -> Self {
                $err(e.into())
            }
        }
        impl From<tokio_pg_mapper::Error> for $ty {
            fn from(e: tokio_pg_mapper::Error) -> Self {
                $err(e.into())
            }
        }
        impl From<deadpool_postgres::PoolError> for $ty {
            fn from(e: deadpool_postgres::PoolError) -> Self {
                $err(e.into())
            }
        }
        impl From<anyhow::Error> for $ty {
            fn from(e: anyhow::Error) -> Self {
                $err(e.into())
            }
        }

        impl From<url::ParseError> for $ty {
            fn from(e: url::ParseError) -> Self {
                $err(e.into())
            }
        }
    };
}
#[macro_export]
macro_rules! validation_handler_err {
    ($ty:ty, $err: expr) => {
        impl From<validator::ValidationErrors> for $ty {
            fn from(e: validator::ValidationErrors) -> Self {
                $err(e.into())
            }
        }
    };
}

#[macro_export]
macro_rules! hide_internal {
    ($t: tt, $self: ident) => {
        match $self {
            $t::Internal(_) => "Internal Error".to_string(),
            _ => $self.to_string(),
        };
    };
}
