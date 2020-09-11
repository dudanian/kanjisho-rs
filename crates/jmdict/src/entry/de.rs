use serde::de;
use serde::de::{Deserializer, IgnoredAny, Visitor};
use std::fmt;

/// helper to convert re_nokanji from its encoded `Option<()>` value
/// to a simple `bool`
pub fn option_as_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionAsBool;

    impl<'de> Visitor<'de> for OptionAsBool {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an option")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_ignored_any(IgnoredAny)?;
            Ok(true)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(false)
        }
    }

    deserializer.deserialize_option(OptionAsBool)
}
