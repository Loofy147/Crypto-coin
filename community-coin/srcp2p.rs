//! P2P networking logic for the Community Coin blockchain.

use libp2p::{
    core::upgrade,
    futures::StreamExt,
    gossipsub::{self, Gossipsub, GossipsubEvent, IdentTopic as Topic, MessageAuthenticity},
    identity,
    mdns::{self, tokio::Behaviour as Mdns},
    noise,
    swarm::{NetworkBehaviour, Swarm, SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    NewTransaction(String),
    NewBlock(String),
}

#[derive(NetworkBehaviour)]
pub struct P2pBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
}

pub struct NetworkService {
    pub swarm: Swarm<P2pBehaviour>,
    pub topic: Topic,
}

impl NetworkService {
    pub async fn new() -> Self {
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Local peer id: {:?}", peer_id);

        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&id_keys).unwrap())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();

        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .message_id_fn(message_id_fn)
            .build()
            .unwrap();

        let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(id_keys), gossipsub_config).unwrap();

        let topic = Topic::new("community-coin");
        gossipsub.subscribe(&topic).unwrap();

        let mdns = Mdns::new(mdns::Config::default()).unwrap();
        let behaviour = P2pBehaviour { gossipsub, mdns };
        let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

        NetworkService { swarm, topic }
    }
}
