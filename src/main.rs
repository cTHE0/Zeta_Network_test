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
use std::fs;
use std::path::Path;
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
        posts.insert(0, post);
        
        if posts.len() > 1000 {
            posts.truncate(1000);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialiser le logger avec format personnalisé
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .init();

    info!("🚀 Démarrage de Zeta2 - Réseau social décentralisé");

    // Lire les arguments
    let args: Vec<String> = std::env::args().collect();
    let is_relay = args.contains(&"--relay".to_string());
    let disable_mdns = args.contains(&"--no-mdns".to_string());
    let relay_addr: Option<String> = args.iter()
        .position(|x| x == "--relay-addr")
        .and_then(|i| args.get(i + 1))
        .cloned();

    info!("⚙️  Mode: {}", if is_relay { "RELAY (Serveur)" } else { "CLIENT" });
    if disable_mdns {
        info!("⚠️  mDNS désactivé");
    }

    // Charger ou générer les clés (persistance pour garder le même Peer ID)
    let key_file = "identity.key";
    let local_key = if Path::new(key_file).exists() {
        info!("🔐 Chargement des clés existantes...");
        let key_bytes = fs::read(key_file)?;
        Keypair::from_protobuf_encoding(&key_bytes)?
    } else {
        info!("🔑 Génération de nouvelles clés...");
        let key = Keypair::generate_ed25519();
        let key_bytes = key.to_protobuf_encoding()?;
        fs::write(key_file, key_bytes)?;
        info!("💾 Clés sauvegardées dans {}", key_file);
        key
    };
    
    let local_peer_id = PeerId::from(local_key.public());
    info!("🔑 Peer ID: {}", local_peer_id);

    info!("📝 Initialisation du swarm...");

    // Créer le swarm
    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            info!("📝 Configuration Gossipsub...");
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(1))
                .validation_mode(gossipsub::ValidationMode::Permissive)
                .build()
                .expect("Configuration Gossipsub valide");

            let mut gossipsub = gossipsub::Behaviour::new(
                MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .expect("Impossible de créer Gossipsub");

            let topic = IdentTopic::new("zeta2-social");
            gossipsub.subscribe(&topic).unwrap();

            info!("📝 Configuration Identify...");
            let identify = identify::Behaviour::new(identify::Config::new(
                "/zeta2/1.0.0".to_string(),
                key.public(),
            ));

            info!("📝 Configuration Kademlia...");
            let kad = kad::Behaviour::new(local_peer_id, MemoryStore::new(local_peer_id));

            info!("📝 Configuration mDNS...");
            let mdns = mdns::Behaviour::new(mdns::Config::default(), local_peer_id)
                .expect("Impossible de créer mDNS");

            info!("📝 Configuration Ping...");
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

    info!("✅ Swarm créé avec succès");
    let topic = IdentTopic::new("zeta2-social");
    info!("📡 Abonné au topic: {}", topic);

    // Configurer les listeners
    info!("📝 Configuration des listeners...");
    
    if is_relay {
        info!("🖥️  Mode RELAY - Écoute sur 0.0.0.0:4001");
        match swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?) {
            Ok(_) => info!("✅ Listener TCP configuré"),
            Err(e) => error!("❌ Erreur lors de l'ajout du listener: {}", e),
        }
    } else {
        info!("💻 Mode CLIENT - Port aléatoire");
        match swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?) {
            Ok(_) => info!("✅ Listener TCP configuré"),
            Err(e) => error!("❌ Erreur lors de l'ajout du listener: {}", e),
        }

        if let Some(addr) = relay_addr {
            match addr.parse::<Multiaddr>() {
                Ok(relay_multiaddr) => {
                    info!("🔗 Connexion au relay: {}", relay_multiaddr);
                    match swarm.dial(relay_multiaddr.clone()) {
                        Ok(_) => info!("✅ Dial initié vers relay"),
                        Err(e) => error!("❌ Erreur dial relay: {}", e),
                    }
                }
                Err(e) => {
                    error!("❌ Adresse relay invalide: {}", e);
                }
            }
        }
    }

    // État du réseau
    let network_state = NetworkState::new(local_peer_id);

    // Channel pour les posts
    let (post_tx, mut post_rx) = mpsc::unbounded_channel::<Post>();

    // Démarrer le serveur web
    let web_state = network_state.clone();
    tokio::spawn(async move {
        if let Err(e) = web_server::start_server(web_state, post_tx).await {
            error!("❌ Erreur serveur web: {}", e);
        }
    });

    info!("🎉 Zeta2 démarré! Interface web sur http://localhost:3030");
    info!("⏳ En attente des événements réseau...");

    // Boucle événements
    loop {
        tokio::select! {
            Some(post) = post_rx.recv() => {
                let msg = NetworkMessage::Post(post);
                if let Ok(json) = serde_json::to_vec(&msg) {
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json) {
                        error!("❌ Erreur publication: {}", e);
                    } else {
                        info!("📤 Post publié");
                    }
                }
            }
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
                                info!("📨 Nouveau post: {}", post.author);
                                network_state.add_post(post).await;
                            }
                            NetworkMessage::Heartbeat => {}
                        }
                    }
                }
                SwarmEvent::Behaviour(ZetaBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        info!("🔍 Peer découvert: {}", peer_id);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        network_state.add_peer(peer_id, multiaddr.to_string()).await;
                    }
                }
                SwarmEvent::Behaviour(ZetaBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _) in list {
                        info!("⏰ Peer expiré: {}", peer_id);
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
                    info!("✅ Connexion: {}", peer_id);
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    info!("❌ Déconnexion: {}", peer_id);
                    network_state.remove_peer(&peer_id).await;
                }
                SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                    info!("📥 Connexion entrante: {} -> {}", send_back_addr, local_addr);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    error!("❌ Erreur connexion sortante vers {:?}: {}", peer_id, error);
                }
                SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                    error!("❌ Erreur connexion entrante de {} vers {}: {}", send_back_addr, local_addr, error);
                }
                SwarmEvent::Dialing { peer_id, .. } => {
                    info!("📞 Tentative de connexion à: {:?}", peer_id);
                }
                _ => {}
            }
        }
    }
}
