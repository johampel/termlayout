/// Provides the conversion from a `Layout` to a `RcLayout`.
///
/// It adds a [`From`] implementation for a specific a `Layout` implementation into an `RcLayout`.
#[macro_export]
macro_rules! rc_layout {
    ($source: ident) => {
        impl From<$source> for $crate::RcLayout {
            fn from(value: $source) -> Self {
                ::std::rc::Rc::new(value)
            }
        }
    };
}

/// Provides the conversion from a `FormattedLayout` to a `BoxedFormattedLayout`.
///
/// It adds a [`From`] implementation for a specific a `FormattedLayout` implementation into an `BoxedFormattedLayout`.
#[macro_export]
macro_rules! box_formatted_layout {
    ($source: ident) => {
        impl<'a> From<$source<'a>> for $crate::BoxedFormattedLayout<'a> {
            fn from(value: $source<'a>) -> Self {
                Box::new(value)
            }
        }
    };
}
