use anyhow::{Context, Result};
use prost::Message;
use prost_types::Any;

pub trait AnyConvert: Sized {
    fn from_any(value: &Any) -> Result<Self>;

    fn to_any(&self) -> Result<Any>;
}

pub fn proto_encode<M: Message>(message: &M) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(message.encoded_len());
    message
        .encode(&mut buf)
        .context("unable to encode protobuf message")?;
    Ok(buf)
}

macro_rules! impl_any_conversion {
    ($type: ty, $type_url: ident) => {
        impl $crate::types::proto_util::AnyConvert for $type {
            fn from_any(value: &::prost_types::Any) -> ::anyhow::Result<Self> {
                ::anyhow::ensure!(
                    value.type_url == $type_url,
                    "invalid type url for `Any` type: expected `{}` and found `{}`",
                    $type_url,
                    value.type_url
                );

                <Self as ::prost::Message>::decode(value.value.as_slice()).map_err(Into::into)
            }

            fn to_any(&self) -> ::anyhow::Result<::prost_types::Any> {
                Ok(::prost_types::Any {
                    type_url: $type_url.to_owned(),
                    value: $crate::types::proto_util::proto_encode(self)?,
                })
            }
        }
    };
}
