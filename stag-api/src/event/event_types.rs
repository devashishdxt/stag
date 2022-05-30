use primitive_types::U256;
use serde::{Deserialize, Serialize};

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
        /// Amount of tokens minted
        amount: U256,
        /// Denom of tokens minted
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
    /// Close channel on IBC enabled chain
    CloseChannelInitOnSoloMachine {
        /// Chain ID of IBC enabled chain
        chain_id: String,
        /// Channel ID of IBC enabled chain on solo machine
        channel_id: ChannelId,
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
