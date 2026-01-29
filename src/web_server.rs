use crate::{NetworkState, Post};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::{Filter, Rejection, Reply};

#[derive(Serialize)]
struct PeerInfo {
    id: String,
    address: String,
}

#[derive(Serialize)]
struct NetworkInfo {
    local_peer_id: String,
    peers: Vec<PeerInfo>,
    posts: Vec<Post>,
}

#[derive(Deserialize)]
struct PostRequest {
    content: String,
    author: String,
}

// Partagé entre les handlers
type SharedState = Arc<RwLock<(NetworkState, mpsc::UnboundedSender<Post>)>>;

pub async fn start_server(
    network_state: NetworkState,
    post_tx: mpsc::UnboundedSender<Post>,
) -> Result<(), Box<dyn std::error::Error>> {
    let shared_state = Arc::new(RwLock::new((network_state.clone(), post_tx)));

    // Route pour servir les fichiers statiques
    let static_files = warp::fs::dir("./static");

    // Route pour obtenir l'état du réseau
    let state = shared_state.clone();
    let network_info = warp::path("api")
        .and(warp::path("network"))
        .and(warp::get())
        .and(with_state(state))
        .and_then(get_network_info);

    // Route pour poster un message
    let state = shared_state.clone();
    let post_message = warp::path("api")
        .and(warp::path("post"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state))
        .and_then(create_post);

    // Combiner les routes
    let routes = static_files
        .or(network_info)
        .or(post_message)
        .with(warp::cors().allow_any_origin());

    tracing::info!("🌐 Serveur web démarré sur http://localhost:3030");
    
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

fn with_state(
    state: SharedState,
) -> impl Filter<Extract = (SharedState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn get_network_info(state: SharedState) -> Result<impl Reply, Rejection> {
    let state_guard = state.read().await;
    let (network_state, _) = &*state_guard;

    let peers_map = network_state.peers.read().await;
    let peers: Vec<PeerInfo> = peers_map
        .iter()
        .map(|(id, addr)| PeerInfo {
            id: id.to_string(),
            address: addr.clone(),
        })
        .collect();

    let posts = network_state.posts.read().await.clone();

    let info = NetworkInfo {
        local_peer_id: network_state.local_peer_id.to_string(),
        peers,
        posts,
    };

    Ok(warp::reply::json(&info))
}

async fn create_post(
    post_req: PostRequest,
    state: SharedState,
) -> Result<impl Reply, Rejection> {
    use chrono::Utc;
    use uuid::Uuid;

    let state_guard = state.read().await;
    let (network_state, post_tx) = &*state_guard;

    let post = Post {
        id: Uuid::new_v4().to_string(),
        author: post_req.author,
        content: post_req.content,
        timestamp: Utc::now().timestamp(),
    };

    // Ajouter localement
    network_state.add_post(post.clone()).await;

    // Envoyer au swarm pour diffusion
    if let Err(e) = post_tx.send(post.clone()) {
        tracing::error!("❌ Erreur lors de l'envoi du post au swarm: {}", e);
    }

    tracing::info!("📝 Post créé: {} - {}", post.author, post.content);

    Ok(warp::reply::json(&post))
}
