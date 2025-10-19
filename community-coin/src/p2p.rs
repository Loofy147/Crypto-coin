//! P2P networking logic for the Community Coin blockchain.

use libp2p::{
    gossipsub::{self, Gossipsub, GossipsubEvent, IdentTopic as Topic, MessageAuthenticity},
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, SwarmBuilder},
    NetworkBehaviour, PeerId, Swarm, Transport,
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
#[behaviour(event_process = true)]
pub struct P2pBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for P2pBehaviour {
    fn inject_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: peer_id,
            message_id: id,
            message,
        } = event
        {
            println!(
                "Got message: {} with id: {} from peer: {:?}",
                String::from_utf8_lossy(&message.data),
                id,
                peer_id
            );
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for P2pBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }
}

pub struct NetworkService {
    pub swarm: Swarm<P2pBehaviour>,
    pub topic: Topic,
}

impl NetworkService {
    pub async fn new() -> Self {
        let id_keys = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Local peer id: {:?}", peer_id);

        let transport = libp2p::development_transport(id_keys.clone()).await.unwrap();

        let message_id_fn = |message: &gossipsub::GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .message_id_fn(message_id_fn)
            .build()
            .unwrap();

        let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(id_keys), gossipsub_config).unwrap();

        let topic = Topic::new("community-coin");
        gossipsub.subscribe(&topic).unwrap();

        let mdns = Mdns::new(Default::default()).await.unwrap();
        let behaviour = P2pBehaviour { gossipsub, mdns };
        let swarm = SwarmBuilder::new(transport, behaviour, peer_id).executor(Box::new(|fut| {
            tokio::spawn(fut);
        })).build();

        NetworkService { swarm, topic }
    }
}
