use libp2p::{
    futures::StreamExt,
    gossipsub::{self, IdentTopic, MessageAuthenticity},
    identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns, noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info};


mod web_server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    id: String,
    author: String,
    content: String,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum NetworkMessage {
    Post(Post),
    Heartbeat,
}

#[derive(NetworkBehaviour)]
struct ZetaBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    kad: kad::Behaviour<MemoryStore>,
}

#[derive(Clone)]
pub struct NetworkState {
    pub peers: Arc<RwLock<HashMap<PeerId, String>>>,
    pub posts: Arc<RwLock<Vec<Post>>>,
    pub local_peer_id: PeerId,
}

impl NetworkState {
    fn new(local_peer_id: PeerId) -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            posts: Arc::new(RwLock::new(Vec::new())),
            local_peer_id,
        }
    }

    pub async fn add_peer(&self, peer_id: PeerId, address: String) {
        self.peers.write().await.insert(peer_id, address);
    }

    pub async fn remove_peer(&self, peer_id: &PeerId) {
        self.peers.write().await.remove(peer_id);
    }

    pub async fn add_post(&self, post: Post) {
        let mut posts = self.posts.write().await;
        posts.insert(0, post); // Insert au début pour avoir les posts récents en premier
        
        // Limiter à 1000 posts en mémoire
        if posts.len() > 1000 {
            posts.truncate(1000);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialiser le logger
    tracing_subscriber::fmt::init();

    info!("🚀 Démarrage de Zeta2 - Réseau social décentralisé");

    // Lire les arguments de ligne de commande
    let args: Vec<String> = std::env::args().collect();
    let is_relay = args.contains(&"--relay".to_string());
    let relay_addr: Option<String> = args.iter()
        .position(|x| x == "--relay-addr")
        .and_then(|i| args.get(i + 1))
        .cloned();

    // Générer une paire de clés
    let local_key = Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    info!("🔑 Peer ID local: {}", local_peer_id);

    // Créer le swarm avec le nouveau builder pattern
    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // Configuration Gossipsub
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(1))
                .validation_mode(gossipsub::ValidationMode::Permissive)
                .build()
                .expect("Configuration Gossipsub valide");

            let mut gossipsub = gossipsub::Behaviour::new(
                MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .expect("Impossible de créer le comportement Gossipsub");

            // S'abonner au topic
            let topic = IdentTopic::new("zeta2-social");
            gossipsub.subscribe(&topic).unwrap();

            // Configuration Identify
            let identify = identify::Behaviour::new(identify::Config::new(
                "/zeta2/1.0.0".to_string(),
                key.public(),
            ));

            // Configuration Kademlia
            let kad = kad::Behaviour::new(local_peer_id, MemoryStore::new(local_peer_id));

            // Configuration mDNS
            let mdns = mdns::Behaviour::new(mdns::Config::default(), local_peer_id)
                .expect("Impossible de créer mDNS");

            // Ping
            let ping = ping::Behaviour::new(ping::Config::new());

            Ok(ZetaBehaviour {
                gossipsub,
                mdns,
                ping,
                identify,
                kad,
            })
        })?
        .build();

    // S'abonner au topic (déjà fait dans le builder mais on garde la référence)
    let topic = IdentTopic::new("zeta2-social");
    info!("📡 Abonné au topic: {}", topic);

    // Écouter sur toutes les interfaces
    if is_relay {
        // Mode serveur relay - écouter sur un port public
        swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?)?;
        info!("🖥️  Mode RELAY activé - Écoute sur 0.0.0.0:4001");
    } else {
        // Mode client - écouter sur un port aléatoire
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        info!("💻 Mode CLIENT activé");

        // Se connecter au relay si fourni
        if let Some(addr) = relay_addr {
            match addr.parse::<Multiaddr>() {
                Ok(relay_multiaddr) => {
                    info!("🔗 Connexion au relay: {}", relay_multiaddr);
                    swarm.dial(relay_multiaddr)?;
                }
                Err(e) => {
                    error!("❌ Adresse relay invalide: {}", e);
                }
            }
        }
    }

    // État du réseau
    let network_state = NetworkState::new(local_peer_id);

    // Channel pour communiquer entre le serveur web et le swarm
    let (post_tx, mut post_rx) = mpsc::unbounded_channel::<Post>();

    // Démarrer le serveur web
    let web_state = network_state.clone();
    tokio::spawn(async move {
        if let Err(e) = web_server::start_server(web_state, post_tx).await {
            error!("❌ Erreur du serveur web: {}", e);
        }
    });

    // Boucle événements
    loop {
        tokio::select! {
            // Recevoir les posts du serveur web
            Some(post) = post_rx.recv() => {
                let msg = NetworkMessage::Post(post);
                if let Ok(json) = serde_json::to_vec(&msg) {
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json) {
                        error!("❌ Erreur lors de la publication: {}", e);
                    } else {
                        info!("📤 Post publié sur le réseau");
                    }
                }
            }
            // Événements du swarm
            event = swarm.select_next_some() => match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("🎧 Écoute sur: {}/p2p/{}", address, local_peer_id);
            }
            SwarmEvent::Behaviour(ZetaBehaviourEvent::Gossipsub(
                gossipsub::Event::Message {
                    propagation_source: _peer_id,
                    message,
                    ..
                },
            )) => {
                if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                    match msg {
                        NetworkMessage::Post(post) => {
                            info!("📨 Nouveau post de {}: {}", post.author, post.content);
                            network_state.add_post(post).await;
                        }
                        NetworkMessage::Heartbeat => {
                            // Heartbeat pour maintenir la connexion
                        }
                    }
                }
            }
            SwarmEvent::Behaviour(ZetaBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    info!("🔍 Peer découvert via mDNS: {} à {}", peer_id, multiaddr);
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    network_state.add_peer(peer_id, multiaddr.to_string()).await;
                }
            }
            SwarmEvent::Behaviour(ZetaBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, multiaddr) in list {
                    info!("⏰ Peer expiré: {} à {}", peer_id, multiaddr);
                    network_state.remove_peer(&peer_id).await;
                }
            }
            SwarmEvent::Behaviour(ZetaBehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info,
                ..
            })) => {
                info!("🆔 Peer identifié: {}", peer_id);
                for addr in info.listen_addrs {
                    network_state.add_peer(peer_id, addr.to_string()).await;
                }
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("✅ Connexion établie avec: {}", peer_id);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("❌ Connexion fermée avec: {}", peer_id);
                network_state.remove_peer(&peer_id).await;
            }
            _ => {}
            }
        }
    }
}
