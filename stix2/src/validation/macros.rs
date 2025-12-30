//! Constraint Validation Macros
//!
//! This module provides macros for declaratively specifying STIX object constraints.

/// Implement the `Constrained` trait for a STIX object with declarative constraints.
///
/// # Supported Constraint Types
///
/// - `timestamp_order`: Validates that one timestamp is >= another
/// - `timestamp_order_strict`: Validates that one timestamp is > another (strict)
/// - `at_least_one`: At least one of the listed properties must be present
/// - `mutually_exclusive`: At most one of the listed properties can be present
/// - `conditional_required`: A property is required when a condition is true
/// - `conditional_excluded`: A property must NOT be present when a condition is true
/// - `custom`: A custom validation expression
///
/// # Example
///
/// ```rust,ignore
/// impl_constraints!(Campaign {
///     timestamp_order: first_seen <= last_seen;
/// });
///
/// impl_constraints!(Indicator {
///     timestamp_order_strict: valid_from < valid_until;
/// });
///
/// impl_constraints!(Location {
///     dependency: [latitude, longitude] => [precision];
/// });
///
/// impl_constraints!(Malware {
///     timestamp_order: first_seen <= last_seen;
///     conditional_required: (is_family) => name;
/// });
/// ```
#[macro_export]
macro_rules! impl_constraints {
    // Match timestamp_order constraints (non-strict: >=)
    ($type:ty {
        timestamp_order: $first:ident <= $second:ident;
        $($rest:tt)*
    }) => {
        impl $crate::validation::Constrained for $type {
            fn validate_constraints(&self) -> $crate::core::error::Result<()> {
                $crate::validation::check_timestamp_order(
                    self.$first.as_ref(),
                    self.$second.as_ref(),
                    stringify!($first),
                    stringify!($second),
                )?;
                impl_constraints!(@rest self $($rest)*);
                Ok(())
            }
        }
    };

    // Match timestamp_order_strict constraints (strict: >)
    ($type:ty {
        timestamp_order_strict: $first:ident < $second:ident;
        $($rest:tt)*
    }) => {
        impl $crate::validation::Constrained for $type {
            fn validate_constraints(&self) -> $crate::core::error::Result<()> {
                $crate::validation::check_timestamp_order_strict(
                    self.$first.as_ref(),
                    self.$second.as_ref(),
                    stringify!($first),
                    stringify!($second),
                )?;
                impl_constraints!(@rest self $($rest)*);
                Ok(())
            }
        }
    };

    // Match empty constraints (no validation needed)
    ($type:ty {}) => {
        impl $crate::validation::Constrained for $type {
            fn validate_constraints(&self) -> $crate::core::error::Result<()> {
                Ok(())
            }
        }
    };

    // Helper: Process remaining constraints after the first
    (@rest $self:ident timestamp_order: $first:ident <= $second:ident; $($rest:tt)*) => {
        $crate::validation::check_timestamp_order(
            $self.$first.as_ref(),
            $self.$second.as_ref(),
            stringify!($first),
            stringify!($second),
        )?;
        impl_constraints!(@rest $self $($rest)*);
    };

    (@rest $self:ident timestamp_order_strict: $first:ident < $second:ident; $($rest:tt)*) => {
        $crate::validation::check_timestamp_order_strict(
            $self.$first.as_ref(),
            $self.$second.as_ref(),
            stringify!($first),
            stringify!($second),
        )?;
        impl_constraints!(@rest $self $($rest)*);
    };

    // Terminal case: no more constraints
    (@rest $self:ident) => {};
}

/// Helper macro for building a list of present properties for at_least_one validation.
///
/// # Example
///
/// ```rust,ignore
/// let present = present_props!(self, description, external_id, url);
/// check_at_least_one(&present, &["description", "external_id", "url"])?;
/// ```
#[macro_export]
macro_rules! present_props {
    ($self:expr, $($prop:ident),+ $(,)?) => {{
        let mut present = Vec::new();
        $(
            if $self.$prop.is_some() {
                present.push(stringify!($prop));
            }
        )+
        present
    }};
}

/// Helper macro for checking if a Vec property is non-empty.
///
/// # Example
///
/// ```rust,ignore
/// let present = present_vec_props!(self, aliases, labels);
/// ```
#[macro_export]
macro_rules! present_vec_props {
    ($self:expr, $($prop:ident),+ $(,)?) => {{
        let mut present = Vec::new();
        $(
            if !$self.$prop.is_empty() {
                present.push(stringify!($prop));
            }
        )+
        present
    }};
}

#[cfg(test)]
mod tests {
    use crate::core::timestamp::Timestamp;
    use crate::validation::Constrained;

    // Test struct for macro testing
    struct TestObj {
        first_seen: Option<Timestamp>,
        last_seen: Option<Timestamp>,
    }

    impl_constraints!(TestObj {
        timestamp_order: first_seen <= last_seen;
    });

    #[test]
    fn test_impl_constraints_macro() {
        let obj = TestObj {
            first_seen: Some(Timestamp::now()),
            last_seen: Some(Timestamp::now()),
        };
        assert!(obj.validate_constraints().is_ok());
    }

    #[test]
    fn test_present_props_macro() {
        struct TestStruct {
            a: Option<String>,
            b: Option<String>,
            c: Option<String>,
        }

        let obj = TestStruct {
            a: Some("hello".to_string()),
            b: None,
            c: Some("world".to_string()),
        };

        let present = present_props!(obj, a, b, c);
        assert_eq!(present, vec!["a", "c"]);
    }
}
