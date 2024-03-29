use primitive_types::U256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::types::{
    chain_state::{ChannelDetails, ConnectionDetails},
    ics::core::ics24_host::identifier::{
        ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId,
    },
    public_key::PublicKey,
};

/// Events emitted by IBC service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum Event {
    // ----- IBC events ----- //
    /// Minted tokens on IBC enabled chain
    TokensMinted {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Optional request ID (for tracking purposes)
        request_id: Option<String>,
        /// Address of account on IBC enabled chain
        to_address: String,
        /// Amount of tokens minted
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
        /// Denom of tokens minted
        denom: Identifier,
        /// Hash of transaction on IBC enabled chain (in hex)
        transaction_hash: String,
    },
    /// Burnt tokens on IBC enabled chain
    TokensBurnt {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Optional request ID (for tracking purposes)
        request_id: Option<String>,
        /// Address of account on IBC enabled chain
        from_address: String,
        /// Amount of tokens minted
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
        /// Denom of tokens minted
        denom: Identifier,
        /// Hash of transaction on IBC enabled chain (in hex)
        transaction_hash: String,
    },
    /// Updated signer's public key on IBC enabled change for future messages from solo machine
    SignerUpdated {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Old signer's public key
        old_public_key: PublicKey,
        /// New signer's public key
        new_public_key: PublicKey,
    },
    /// Tokens sent from ICA (Interchain Account)
    TokensSentFromIca {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Optional request ID (for tracking purposes)
        request_id: Option<String>,
        /// Address of account on IBC enabled chain
        to_address: String,
        /// Amount of tokens sent
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
        /// Denom of tokens sent
        denom: Identifier,
        /// Hash of transaction on IBC enabled chain (in hex)
        transaction_hash: String,
    },
    /// Tokens delegated from ICA (Interchain Account) to validator address
    TokensDelegatedFromIca {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Optional request ID (for tracking purposes)
        request_id: Option<String>,
        /// Address of validator
        validator_address: String,
        /// Amount of tokens delegated
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
        /// Denom of tokens delegated
        denom: Identifier,
        /// Hash of transaction on IBC enabled chain (in hex)
        transaction_hash: String,
    },
    /// Tokens undelegated to ICA (Interchain Account) from validator address
    TokensUndelegatedToIca {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Optional request ID (for tracking purposes)
        request_id: Option<String>,
        /// Address of validator
        validator_address: String,
        /// Amount of tokens undelegated
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
        /// Denom of tokens undelegated
        denom: Identifier,
        /// Hash of transaction on IBC enabled chain (in hex)
        transaction_hash: String,
    },

    // ----- IBC connection handshake events ----- //
    /// Created solo machine client on IBC enabled chain
    CreatedSoloMachineClient {
        /// Client ID of solo machine client on IBC enabled chain
        client_id: ClientId,
    },
    /// Created tendermint client on solo machine
    CreatedTendermintClient {
        /// Client ID of IBC enabled chain on solo machine
        client_id: ClientId,
    },
    /// Initialized connection on IBC enabled chain
    InitializedConnectionOnTendermint {
        /// Connection ID of solo machine client on IBC enabled chain
        connection_id: ConnectionId,
    },
    /// Initialized connection on solo machine
    InitializedConnectionOnSoloMachine {
        /// Connection ID of IBC enabled chain on solo machine
        connection_id: ConnectionId,
    },
    /// Confirmed connection on IBC enabled chain
    ConfirmedConnectionOnTendermint {
        /// Connection ID of solo machine client on IBC enabled chain
        connection_id: ConnectionId,
    },
    /// Confirmed connection on solo machine
    ConfirmedConnectionOnSoloMachine {
        /// Connection ID of IBC enabled chain on solo machine
        connection_id: ConnectionId,
    },
    /// Initialized channel on IBC enabled chain
    InitializedChannelOnTendermint {
        /// Channel ID of solo machine client on IBC enabled chain
        channel_id: ChannelId,
        /// Port ID of channel
        port_id: PortId,
    },
    /// Initialized channel on solo machine
    InitializedChannelOnSoloMachine {
        /// Channel ID of IBC enabled chain on solo machine
        channel_id: ChannelId,
        /// Port ID of channel
        port_id: PortId,
    },
    /// Confirmed channel on IBC enabled chain
    ConfirmedChannelOnTendermint {
        /// Channel ID of solo machine client on IBC enabled chain
        channel_id: ChannelId,
        /// Port ID of channel
        port_id: PortId,
    },
    /// Confirmed channel on solo machine
    ConfirmedChannelOnSoloMachine {
        /// Channel ID of IBC enabled chain on solo machine
        channel_id: ChannelId,
        /// Port ID of channel
        port_id: PortId,
    },
    /// Connection successfully established
    ConnectionEstablished {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Connection details
        connection_details: ConnectionDetails,
    },
    /// Channel successfully created
    ChannelCreated {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Channel details
        channel_details: ChannelDetails,
    },
    /// Close channel on IBC enabled chain
    CloseChannelOnTendermint {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Port ID of channel
        port_id: PortId,
    },
    /// Close channel on IBC enabled chain
    CloseChannelOnSoloMachine {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Port ID of channel
        port_id: PortId,
    },
    /// Channel successfully closed
    ChannelClosed {
        /// Chain ID of IBC enabled chain
        chain_id: ChainId,
        /// Channel details
        channel_details: ChannelDetails,
    },

    // ----- Chain events ----- //
    /// Added new chain metadata to solo machine
    ChainAdded {
        /// Chain ID
        chain_id: ChainId,
    },

    // ----- Other events ----- //
    /// Warning
    Warning {
        /// Warning message
        message: String,
    },

    // ----- Test event ----- //
    /// Test event
    #[cfg(test)]
    Test,
}

fn serialize_u256<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    U256::from_dec_str(&s).map_err(serde::de::Error::custom)
}
