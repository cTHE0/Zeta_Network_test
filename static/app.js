// État de l'application
let currentUser = localStorage.getItem('zeta2_username') || '';
let lastPostCount = 0;

// Initialisation
document.addEventListener('DOMContentLoaded', () => {
    const authorInput = document.getElementById('author');
    if (currentUser) {
        authorInput.value = currentUser;
    }

    // Gestionnaire de formulaire
    const postForm = document.getElementById('post-form');
    postForm.addEventListener('submit', handleSubmit);

    // Compteur de caractères
    const contentTextarea = document.getElementById('content');
    const charCount = document.getElementById('char-count');
    contentTextarea.addEventListener('input', () => {
        const count = contentTextarea.value.length;
        charCount.textContent = count;
        charCount.style.color = count > 250 ? '#e0245e' : '#8899a6';
    });

    // Actualiser les données
    fetchNetworkInfo();
    setInterval(fetchNetworkInfo, 2000); // Rafraîchir toutes les 2 secondes
});

// Récupérer les informations du réseau
async function fetchNetworkInfo() {
    try {
        const response = await fetch('/api/network');
        if (!response.ok) {
            throw new Error('Erreur réseau');
        }
        
        const data = await response.json();
        updateUI(data);
    } catch (error) {
        console.error('Erreur lors de la récupération des données:', error);
    }
}

// Mettre à jour l'interface
function updateUI(data) {
    // Mettre à jour le Peer ID local
    const localPeerIdEl = document.getElementById('local-peer-id');
    localPeerIdEl.textContent = data.local_peer_id;

    // Mettre à jour le nombre de pairs
    const peerCountEl = document.getElementById('peer-count');
    peerCountEl.textContent = data.peers.length;

    // Mettre à jour la liste des pairs
    updatePeersList(data.peers);

    // Mettre à jour le nombre de posts
    const postCountEl = document.getElementById('post-count');
    postCountEl.textContent = data.posts.length;

    // Mettre à jour le fil d'actualité
    updatePostsFeed(data.posts);
}

// Mettre à jour la liste des pairs
function updatePeersList(peers) {
    const peersList = document.getElementById('peers-list');
    
    if (peers.length === 0) {
        peersList.innerHTML = '<p class="empty-state">Aucun pair connecté</p>';
        return;
    }

    const peersHTML = peers.map(peer => `
        <div class="peer-item">
            <div class="peer-id">
                <span class="status-indicator status-connected"></span>
                ${truncateId(peer.id)}
            </div>
            <div class="peer-address">${peer.address}</div>
        </div>
    `).join('');

    peersList.innerHTML = peersHTML;
}

// Mettre à jour le fil d'actualité
function updatePostsFeed(posts) {
    const postsFeed = document.getElementById('posts-feed');
    
    if (posts.length === 0) {
        postsFeed.innerHTML = '<p class="empty-state">Aucun post pour le moment</p>';
        return;
    }

    // Notification si nouveau post
    if (posts.length > lastPostCount) {
        playNotificationSound();
    }
    lastPostCount = posts.length;

    const postsHTML = posts.map(post => `
        <div class="post-item">
            <div class="post-header">
                <span class="post-author">👤 ${escapeHtml(post.author)}</span>
                <span class="post-time">${formatTimestamp(post.timestamp)}</span>
            </div>
            <div class="post-content">${escapeHtml(post.content)}</div>
            <div class="post-id">ID: ${post.id}</div>
        </div>
    `).join('');

    postsFeed.innerHTML = postsHTML;
}

// Gestionnaire de soumission du formulaire
async function handleSubmit(event) {
    event.preventDefault();
    
    const authorInput = document.getElementById('author');
    const contentInput = document.getElementById('content');
    
    const author = authorInput.value.trim();
    const content = contentInput.value.trim();
    
    if (!author || !content) {
        alert('Veuillez remplir tous les champs');
        return;
    }

    // Sauvegarder le nom d'utilisateur
    currentUser = author;
    localStorage.setItem('zeta2_username', author);

    try {
        const response = await fetch('/api/post', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ author, content }),
        });

        if (!response.ok) {
            throw new Error('Erreur lors de la publication');
        }

        // Réinitialiser le formulaire
        contentInput.value = '';
        document.getElementById('char-count').textContent = '0';

        // Rafraîchir immédiatement
        fetchNetworkInfo();
        
        // Notification de succès
        showNotification('✅ Post publié avec succès !');
    } catch (error) {
        console.error('Erreur:', error);
        showNotification('❌ Erreur lors de la publication', true);
    }
}

// Utilitaires
function truncateId(id) {
    if (id.length <= 20) return id;
    return id.substring(0, 10) + '...' + id.substring(id.length - 10);
}

function formatTimestamp(timestamp) {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = Math.floor((now - date) / 1000);

    if (diff < 60) return 'À l\'instant';
    if (diff < 3600) return `Il y a ${Math.floor(diff / 60)} min`;
    if (diff < 86400) return `Il y a ${Math.floor(diff / 3600)} h`;
    
    return date.toLocaleDateString('fr-FR', {
        day: 'numeric',
        month: 'short',
        hour: '2-digit',
        minute: '2-digit'
    });
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showNotification(message, isError = false) {
    // Simple notification en console pour le prototype
    console.log(message);
    
    // On pourrait améliorer avec un toast/notification visuelle
}

function playNotificationSound() {
    // Pour le prototype, on skip le son
    // On pourrait ajouter un son de notification ici
}
