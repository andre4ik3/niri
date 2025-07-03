use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use knuffel::ast::{Literal, TypeName};
use knuffel::decode::{Context, Kind};
use knuffel::errors::DecodeError;
use knuffel::span::Spanned;
use knuffel::{Decode, DecodeScalar};
use miette::miette;
use zbus::names::{InterfaceName, MemberName, WellKnownName};
use zbus::zvariant::{ObjectPath, Value};
use zbus::Address;

/// Placeholder name returned when an error is encountered
const ERROR_NAME: &str = "error.error";

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum DBusBus {
    #[default]
    Session,
    System,
    Other(String),
}

impl DBusBus {
    pub fn address(&self) -> zbus::Result<Address> {
        match self {
            Self::Session => Address::session(),
            Self::System => Address::system(),
            Self::Other(other) => Address::from_str(other),
        }
    }
}

impl Display for DBusBus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Session => f.write_str("session"),
            Self::System => f.write_str("system"),
            Self::Other(other) => other.fmt(f),
        }
    }
}

impl FromStr for DBusBus {
    type Err = miette::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session" => Ok(Self::Session),
            "system" => Ok(Self::System),
            other => {
                // Check if it's a valid address but do not store it as the address
                if let Err(err) = Address::from_str(other) {
                    Err(miette!(err))
                } else {
                    Ok(Self::Other(other.to_string()))
                }
            }
        }
    }
}

/// Defines a Knuffel scalar type that wraps a zbus_names/zvariant type
macro_rules! dbus_scalar_type {
    ($niri_type:ident, $zbus_type:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $niri_type($zbus_type);

        impl<S> DecodeScalar<S> for $niri_type
        where
            S: knuffel::traits::ErrorSpan,
        {
            fn type_check(type_name: &Option<Spanned<TypeName, S>>, ctx: &mut Context<S>) {
                String::type_check(type_name, ctx)
            }

            fn raw_decode(
                value: &Spanned<Literal, S>,
                ctx: &mut Context<S>,
            ) -> Result<Self, DecodeError<S>> {
                match &**value {
                    Literal::String(ref s) => match <$zbus_type>::try_from(s.to_string()) {
                        Ok(val) => return Ok($niri_type(val)),
                        Err(err) => ctx.emit_error(DecodeError::conversion(value, err)),
                    },
                    _ => ctx.emit_error(DecodeError::scalar_kind(Kind::String, value)),
                }

                // By this point one of the errors above was emitted, return error name
                Ok($niri_type(<$zbus_type>::try_from(ERROR_NAME).unwrap()))
            }
        }

        impl From<$niri_type> for $zbus_type {
            fn from(value: $niri_type) -> Self {
                value.0
            }
        }

        impl Display for $niri_type {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

dbus_scalar_type!(DBusServiceName, WellKnownName<'static>);
dbus_scalar_type!(DBusInterfaceName, InterfaceName<'static>);
dbus_scalar_type!(DBusMethodName, MemberName<'static>);
dbus_scalar_type!(DBusObjectPath, ObjectPath<'static>);

// pub struct DBusArguments(Value);

#[derive(Decode, Debug, Clone, PartialEq)]
pub struct DBusAction {
    #[knuffel(property, str, default)]
    pub bus: DBusBus,

    #[knuffel(child, unwrap(argument))]
    pub service: DBusServiceName,

    #[knuffel(child, unwrap(argument))]
    pub object: DBusObjectPath,

    #[knuffel(child, unwrap(argument))]
    pub interface: DBusInterfaceName,

    #[knuffel(child, unwrap(argument))]
    pub method: DBusMethodName,
    // TODO...
    // #[knuffel(arguments)]
    // pub arguments: knuffel::ast::Value<knuffel::span::Span>,
    // pub arguments: Vec<String>,
}
