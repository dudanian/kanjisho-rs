/// Helper macro to implement simple parsed data types' deserialize methods.
macro_rules! deserialize_type {
    ($type:ident) => {
        paste::paste! {
            fn [<deserialize_ $type>]<V>(self, visitor: V) -> Result<V::Value>
            where
                V: Visitor<'de>,
            {
                paste::paste! {
                    visitor.[<visit_ $type>](self.parse_type()?)
                }
            }
        }
    };
}

#[doc(hidden)]
macro_rules! deserialize_unimplemented_method {
    ($type:ident($($arg:ident : $ty:ty),*)) => {
        #[inline]
        paste::paste! {
            fn [<deserialize_ $type>]<V>(self, $($arg: $ty,)* _visitor: V) -> Result<V::Value>
            where
                V: Visitor<'de>,
            {
                $(
                    let _ = $arg;
                )*
                unimplemented!();
            }
        }
    };
}

/// Helper macro when implementing the `Deserializer` to explicitly
/// `unimplement!()` methods.
macro_rules! deserialize_unimplemented {
    (unit_struct) => {
        deserialize_unimplemented_method! {unit_struct(name: &'static str)}
    };
    (newtype_struct) => {
        deserialize_unimplemented_method! {newtype_struct(name: &'static str)}
    };
    (tuple) => {
        deserialize_unimplemented_method! {tuple(len: usize)}
    };
    (tuple_struct) => {
        deserialize_unimplemented_method! {tuple_struct(name: &'static str, len: usize)}
    };
    (struct) => {
        deserialize_unimplemented_method! {struct(name: &'static str, fields: &'static [&'static str])}
    };
    (enum) => {
        deserialize_unimplemented_method! {enum(name: &'static str, variants: &'static [&'static str])}
    };
    ($type:ident) => {
        deserialize_unimplemented_method! {$type()}
    };
}
