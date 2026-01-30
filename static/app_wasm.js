// app_wasm.js - Client JavaScript pour Zeta Network (version WASM)
// Ce fichier charge et interagit avec le code Rust compilé en WebAssembly

import init, { 
    init_node, 
    publish_post, 
    on_message, 
    on_peer_change, 
    on_status_change,
    get_peers,
    get_posts,
    get_node_info
} from './pkg/zeta2.js';

// État de l'application
let nodeInfo = null;
let posts = [];
let peers = [];

// Éléments DOM
const elements = {
    loadingScreen: document.getElementById('loading-screen'),
    mainContent: document.getElementById('main-content'),
    loadingStatus: document.getElementById('loading-status'),
    connectionIndicator: document.getElementById('connection-indicator'),
    connectionText: document.getElementById('connection-text'),
    localPeerId: document.getElementById('local-peer-id'),
    localName: document.getElementById('local-name'),
    nodeMode: document.getElementById('node-mode'),
    peerCount: document.getElementById('peer-count'),
    peersList: document.getElementById('peers-list'),
    postForm: document.getElementById('post-form'),
    authorInput: document.getElementById('author'),
    contentInput: document.getElementById('content'),
    charCount: document.getElementById('char-count'),
    postsFeed: document.getElementById('posts-feed'),
    postCountBadge: document.getElementById('post-count-badge'),
    submitBtn: document.getElementById('submit-btn'),
    // Steps
    stepWasm: document.getElementById('step-wasm'),
    stepKeypair: document.getElementById('step-keypair'),
    stepConnect: document.getElementById('step-connect'),
};

// Mise à jour des étapes de chargement
function updateStep(stepId, status) {
    const step = document.getElementById(stepId);
    if (step) {
        const icon = step.querySelector('.step-icon');
        if (status === 'loading') {
            icon.textContent = '⏳';
            step.classList.add('loading');
            step.classList.remove('done', 'error');
        } else if (status === 'done') {
            icon.textContent = '✅';
            step.classList.add('done');
            step.classList.remove('loading', 'error');
        } else if (status === 'error') {
            icon.textContent = '❌';
            step.classList.add('error');
            step.classList.remove('loading', 'done');
        }
    }
}

// Initialisation
async function initApp() {
    console.log('🚀 Démarrage de Zeta Network (WASM)...');
    
    try {
        // Étape 1: Charger le WASM
        elements.loadingStatus.textContent = 'Chargement du code Rust...';
        updateStep('step-wasm', 'loading');
        
        await init();
        
        updateStep('step-wasm', 'done');
        console.log('✅ WASM chargé');
        
        // Étape 2: Initialiser le nœud
        elements.loadingStatus.textContent = 'Génération de votre identité...';
        updateStep('step-keypair', 'loading');
        
        // Configurer les callbacks avant d'initialiser
        on_message((msgJson) => {
            const msg = JSON.parse(msgJson);
            handleNetworkMessage(msg);
        });
        
        on_peer_change((peersJson) => {
            peers = JSON.parse(peersJson);
            updatePeersList();
        });
        
        on_status_change((status) => {
            handleStatusChange(status);
        });
        
        // Obtenir l'adresse du relay depuis l'URL si fournie
        const urlParams = new URLSearchParams(window.location.search);
        const relayAddr = urlParams.get('relay');
        
        // Initialiser le nœud P2P
        const node = await init_node(relayAddr);
        
        updateStep('step-keypair', 'done');
        console.log('✅ Nœud initialisé:', node.peer_id);
        
        // Étape 3: Connexion au réseau
        elements.loadingStatus.textContent = 'Connexion au réseau P2P...';
        updateStep('step-connect', 'loading');
        
        // Sauvegarder les infos du nœud
        nodeInfo = {
            peerId: node.peer_id,
            name: node.name
        };
        
        // Mettre à jour l'interface
        elements.localPeerId.textContent = nodeInfo.peerId.substring(0, 20) + '...';
        elements.localPeerId.title = nodeInfo.peerId;
        elements.localName.textContent = nodeInfo.name;
        
        // Charger le nom sauvegardé si existant
        const savedName = localStorage.getItem('zeta_username');
        if (savedName) {
            elements.authorInput.value = savedName;
        }
        
        // La connexion sera confirmée via le callback on_status_change
        
    } catch (error) {
        console.error('❌ Erreur d\'initialisation:', error);
        elements.loadingStatus.textContent = `Erreur: ${error.message || error}`;
        updateStep('step-wasm', 'error');
        setConnectionStatus('error', 'Erreur d\'initialisation');
    }
}

// Gestion des messages du réseau
function handleNetworkMessage(msg) {
    console.log('📨 Message reçu:', msg);
    
    if (msg.Post) {
        addPost(msg.Post);
    } else if (msg.PeerJoined) {
        console.log(`👋 Nouveau pair: ${msg.PeerJoined.name}`);
    } else if (msg.PeerLeft) {
        console.log(`👋 Pair parti: ${msg.PeerLeft.peer_id}`);
    }
}

// Gestion des changements de statut
function handleStatusChange(status) {
    console.log('📊 Statut:', status);
    
    switch (status) {
        case 'connecting':
            setConnectionStatus('connecting', 'Connexion en cours...');
            break;
        case 'connected':
            setConnectionStatus('connected', 'Connecté au réseau P2P');
            updateStep('step-connect', 'done');
            showMainContent();
            break;
        case 'disconnected':
            setConnectionStatus('disconnected', 'Déconnecté');
            break;
        case 'error':
            setConnectionStatus('error', 'Erreur de connexion');
            updateStep('step-connect', 'error');
            break;
    }
}

// Afficher le contenu principal
function showMainContent() {
    elements.loadingScreen.style.display = 'none';
    elements.mainContent.style.display = 'grid';
}

// Mettre à jour le statut de connexion
function setConnectionStatus(status, text) {
    elements.connectionIndicator.className = `status-indicator status-${status}`;
    elements.connectionText.textContent = text;
}

// Mettre à jour la liste des pairs
function updatePeersList() {
    elements.peerCount.textContent = peers.length;
    
    if (peers.length === 0) {
        elements.peersList.innerHTML = '<p class="empty-state">Recherche de pairs...</p>';
        return;
    }
    
    elements.peersList.innerHTML = peers.map(peer => `
        <div class="peer-item ${peer.is_browser ? 'browser-peer' : 'native-peer'}">
            <span class="peer-icon">${peer.is_browser ? '🌐' : '💻'}</span>
            <div class="peer-info">
                <span class="peer-name">${peer.name || 'Anonyme'}</span>
                <span class="peer-id">${peer.peer_id.substring(0, 16)}...</span>
            </div>
            <span class="peer-type">${peer.is_browser ? 'Browser' : 'Natif'}</span>
        </div>
    `).join('');
}

// Ajouter un post à l'interface
function addPost(post) {
    // Éviter les doublons
    if (posts.some(p => p.id === post.id)) {
        return;
    }
    
    posts.unshift(post);
    updatePostsFeed();
}

// Mettre à jour le fil d'actualité
function updatePostsFeed() {
    elements.postCountBadge.textContent = posts.length;
    
    if (posts.length === 0) {
        elements.postsFeed.innerHTML = '<p class="empty-state">Aucun post pour le moment. Soyez le premier !</p>';
        return;
    }
    
    elements.postsFeed.innerHTML = posts.map(post => {
        const date = new Date(post.timestamp * 1000);
        const timeAgo = getTimeAgo(date);
        const isLocal = post.author === nodeInfo?.peerId;
        
        return `
            <div class="post ${isLocal ? 'post-local' : ''}">
                <div class="post-header">
                    <span class="post-author">${escapeHtml(post.author_name)}</span>
                    <span class="post-time" title="${date.toLocaleString()}">${timeAgo}</span>
                </div>
                <div class="post-content">${escapeHtml(post.content)}</div>
                <div class="post-footer">
                    <span class="post-peer-id" title="${post.author}">
                        ${isLocal ? '📍 Vous' : `🔗 ${post.author.substring(0, 12)}...`}
                    </span>
                </div>
            </div>
        `;
    }).join('');
}

// Gestion du formulaire
elements.postForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const author = elements.authorInput.value.trim();
    const content = elements.contentInput.value.trim();
    
    if (!author || !content) {
        return;
    }
    
    // Sauvegarder le nom
    localStorage.setItem('zeta_username', author);
    
    // Désactiver le bouton
    elements.submitBtn.disabled = true;
    elements.submitBtn.textContent = '📤 Publication...';
    
    try {
        // Publier via WASM
        const postJson = publish_post(content, author);
        const post = JSON.parse(postJson);
        
        // Ajouter localement
        addPost(post);
        
        // Vider le champ
        elements.contentInput.value = '';
        elements.charCount.textContent = '0';
        
        console.log('✅ Post publié:', post);
        
    } catch (error) {
        console.error('❌ Erreur publication:', error);
        alert('Erreur lors de la publication: ' + error);
    } finally {
        elements.submitBtn.disabled = false;
        elements.submitBtn.textContent = '📤 Publier sur le réseau P2P';
    }
});

// Compteur de caractères
elements.contentInput.addEventListener('input', () => {
    elements.charCount.textContent = elements.contentInput.value.length;
});

// Utilitaires
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function getTimeAgo(date) {
    const seconds = Math.floor((new Date() - date) / 1000);
    
    if (seconds < 60) return 'À l\'instant';
    if (seconds < 3600) return `Il y a ${Math.floor(seconds / 60)} min`;
    if (seconds < 86400) return `Il y a ${Math.floor(seconds / 3600)} h`;
    return date.toLocaleDateString('fr-FR');
}

// Démarrer l'application
initApp();
