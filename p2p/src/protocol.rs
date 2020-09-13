pub mod bitcoin;
pub use self::bitcoin::Bitcoin;

use crate::event::Event;

use std::fmt::Debug;
use std::net;

use nakamoto_common::block::time::{LocalDuration, LocalTime};

/// Identifies a peer.
pub type PeerId = net::SocketAddr;

/// A timeout.
pub type Timeout = LocalDuration;

/// Link direction of the peer connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    /// Inbound conneciton.
    Inbound,
    /// Outbound connection.
    Outbound,
}

/// A message that can be sent to a peer.
pub trait Message: Send + Sync + 'static {
    /// The message payload.
    type Payload: Clone + Debug;

    /// Construct a message from a payload and magic.
    fn from_parts(payload: Self::Payload, magic: u32) -> Self;
    /// Retrieve the message payload.
    fn payload(&self) -> &Self::Payload;
    /// Retrieve the message magic.
    fn magic(&self) -> u32;
    /// Display the message.
    fn display(&self) -> &'static str;
}

/// Timeout source descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutSource {
    /// Header sync.
    Synch(PeerId),
    /// Peer connect.
    Connect(PeerId),
    /// Peer handshake.
    Handshake(PeerId),
    /// Peer ping.
    Ping(PeerId),
    /// A general timeout.
    Global,
}

/// A protocol input event, parametrized over the network message type.
/// These are input events generated outside of the protocol.
#[derive(Debug, Clone)]
pub enum Input<M, C> {
    /// New connection with a peer.
    Connected {
        /// Remote peer id.
        addr: PeerId,
        /// Local peer id.
        local_addr: PeerId,
        /// Link direction.
        link: Link,
    },
    /// Disconnected from peer.
    Disconnected(PeerId),
    /// Received a message from a remote peer.
    Received(PeerId, M),
    /// Sent a message to a remote peer, of the given size.
    Sent(PeerId, usize),
    /// An external command has been received.
    Command(C),
    /// A timeout has been reached.
    Timeout(TimeoutSource),
}

/// Output of a state transition (step) of the `Protocol` state machine.
#[derive(Debug)]
pub enum Out<M: Message> {
    /// Send a message to a peer.
    Message(PeerId, M),
    /// Connect to a peer.
    Connect(PeerId, Timeout),
    /// Disconnect from a peer.
    Disconnect(PeerId),
    /// Set a timeout associated with a peer.
    SetTimeout(TimeoutSource, Timeout),
    /// An event has occured.
    Event(Event<M::Payload>),
    /// Shutdown protocol.
    Shutdown,
}

impl<M: Message> From<Event<M::Payload>> for Out<M> {
    fn from(event: Event<M::Payload>) -> Self {
        Out::Event(event)
    }
}

/// A finite-state machine that can advance one step at a time, given an input event.
/// Parametrized over the message type.
pub trait Protocol<M: Message> {
    /// Duration of inactivity before timing out a peer.
    const IDLE_TIMEOUT: LocalDuration;

    /// A command to query or control the protocol.
    type Command;
    /// The output of a state machine transition.
    type Output: Iterator<Item = Out<M>>;

    /// Initialize the protocol. Called once before any event is sent to the state machine.
    fn initialize(&mut self, time: LocalTime) -> Self::Output;

    /// Process the next event and advance the state-machine by one step.
    /// Returns messages destined for peers.
    fn step(&mut self, event: Input<M, Self::Command>, local_time: LocalTime) -> Self::Output;
}
