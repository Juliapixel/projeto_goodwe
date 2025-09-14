#![allow(unused)]

#[macro_export]
macro_rules! trace {
    ($format:literal $(,$arg:expr)*) => {
        #[cfg(feature = "defmt")]
        {
            defmt::trace!($format $(,$arg)*);
        }
        #[cfg(not(feature = "defmt"))]
        {
            let _ = ($format $(,$arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($format:literal $(,$arg:expr)*) => {
        #[cfg(feature = "defmt")]
        {
            defmt::debug!($format $(,$arg)*);
        }
        #[cfg(not(feature = "defmt"))]
        {
            let _ = ($format $(,$arg)*);
        }
    };
}

#[macro_export]
macro_rules! info {
    ($format:literal $(,$arg:expr)*) => {
        #[cfg(feature = "defmt")]
        {
            defmt::info!($format $(,$arg)*);
        }
        #[cfg(not(feature = "defmt"))]
        {
            let _ = ($format $(,$arg)*);
        }
    };
}

#[macro_export]
macro_rules! _warn {
    ($format:literal $(,$arg:expr)*) => {
        #[cfg(feature = "defmt")]
        {
            defmt::warn!($format $(,$arg)*);
        }
        #[cfg(not(feature = "defmt"))]
        {
            let _ = ($format $(,$arg)*);
        }
    };
}

#[macro_export]
macro_rules! error {
    ($format:literal $(,$arg:expr)*) => {
        #[cfg(feature = "defmt")]
        {
            defmt::error!($format $(,$arg)*);
        }
        #[cfg(not(feature = "defmt"))]
        {
            let _ = ($format $(,$arg)*);
        }
    };
}

pub use _warn as warn;
pub use debug;
pub use error;
pub use info;
pub use trace;
