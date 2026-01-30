# Zeta2 - Réseau Social Décentralisé 🌐

Un réseau social décentralisé P2P construit avec **Rust** et **libp2p**.

## 🚀 Démarrage Rapide

### Rejoindre comme utilisateur
Ouvrez simplement : **https://tt665.pythonanywhere.com**

### Devenir un relais (une seule commande !)
```bash
curl -sSL https://raw.githubusercontent.com/cTHE0/Zeta_Network_test/main/run_relay.sh | bash
```

Cette commande :
- ✅ Installe Rust automatiquement
- ✅ Installe les dépendances
- ✅ Compile le relay
- ✅ Crée un tunnel WSS sécurisé
- ✅ Affiche votre URL à partager

## 🎯 Caractéristiques

- **Complètement décentralisé** : Aucun serveur central obligatoire
- **Communication P2P** : Utilise libp2p avec GossipSub
- **Tunnel WSS automatique** : Via Cloudflare Tunnel (gratuit)
- **Interface Web** : Accessible depuis n'importe quel navigateur
- **Multi-relais** : Supporte plusieurs relais pour la résilience

## 🏗️ Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Navigateur    │     │   Navigateur    │     │   Navigateur    │
│   (Utilisateur) │     │   (Utilisateur) │     │   (Relais CLI)  │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │ WSS                   │ WSS                   │ TCP
         │                       │                       │
         └───────────┬───────────┴───────────┬───────────┘
                     │                       │
              ┌──────▼──────┐         ┌──────▼──────┐
              │   Relais 1  │◄───────►│   Relais 2  │
              │  (Rust P2P) │  libp2p │  (Rust P2P) │
              └─────────────┘         └─────────────┘
```

## 📋 Utilisation Détaillée

### Lancer un relais (si déjà cloné)
```bash
./run_relay.sh
```

### Lancer manuellement (avancé)
```bash
# Compiler
cargo build --release

# Lancer le relay
./target/release/zeta2 --port 3030

# Dans un autre terminal, lancer le tunnel
cloudflared tunnel --url http://localhost:3030
```


Le serveur public permet aux clients derrière NAT de se connecter.

```bash
cargo run --release -- --relay
```

Le serveur écoutera sur le port **4001** (TCP).

**Note importante** : Notez l'adresse complète affichée, par exemple :
```
🎧 Écoute sur: /ip4/1.2.3.4/tcp/4001/p2p/12D3KooW...
```

### Mode Client (PC derrière NAT)

Sur chaque PC derrière NAT, lancez avec l'adresse du serveur :

```bash
cargo run --release -- --relay-addr "/ip4/ADRESSE_IP_SERVEUR/tcp/4001/p2p/PEER_ID_SERVEUR"
```

Exemple :
```bash
cargo run --release -- --relay-addr "/ip4/192.168.1.100/tcp/4001/p2p/12D3KooWABC123..."
```

### Mode Local (Réseau local)

Si tous les nœuds sont sur le même réseau local :

```bash
cargo run --release
```

La découverte automatique via mDNS se fera automatiquement.

## 🌐 Interface Web

Une fois le nœud démarré, accédez à l'interface web :

```
http://localhost:3030
```

### Fonctionnalités de l'interface

- **État du réseau** : Voir votre Peer ID et le nombre de pairs connectés
- **Liste des pairs** : Voir tous les pairs connectés en temps réel
- **Publier un post** : Envoyer un message à tous les pairs (max 280 caractères)
- **Fil d'actualité** : Voir tous les posts reçus du réseau

## 🔧 Configuration

### Variables d'environnement

```bash
# Niveau de log (trace, debug, info, warn, error)
export RUST_LOG=info

# Lancer avec des logs détaillés
RUST_LOG=debug cargo run
```

### Ports utilisés

- **3030** : Interface web (localhost uniquement)
- **4001** : Port P2P pour le mode relay
- **Port aléatoire** : Port P2P pour le mode client (assigné automatiquement)

## 📖 Scénario d'utilisation typique

### Configuration avec 2 PC derrière NAT + 1 serveur

1. **Sur le serveur (IP publique)** :
   ```bash
   cargo run --release -- --relay
   ```
   Notez l'adresse multiaddr affichée.

2. **Sur le PC 1 (derrière NAT)** :
   ```bash
   cargo run --release -- --relay-addr "/ip4/IP_SERVEUR/tcp/4001/p2p/PEER_ID_SERVEUR"
   ```

3. **Sur le PC 2 (derrière NAT)** :
   ```bash
   cargo run --release -- --relay-addr "/ip4/IP_SERVEUR/tcp/4001/p2p/PEER_ID_SERVEUR"
   ```

4. **Accédez à l'interface web** sur chaque machine :
   ```
   http://localhost:3030
   ```

Les trois nœuds peuvent maintenant communiquer !

## 🛠️ Développement

### Structure du projet

```
zeta2/
├── Cargo.toml          # Dépendances Rust
├── src/
│   ├── main.rs         # Nœud P2P principal
│   └── web_server.rs   # Serveur web API
└── static/
    ├── index.html      # Interface utilisateur
    ├── style.css       # Styles
    └── app.js          # Logique frontend
```

### Technologies utilisées

- **libp2p** : Framework P2P modulaire
- **Tokio** : Runtime asynchrone
- **Warp** : Framework web
- **Serde** : Sérialisation JSON
- **GossipSub** : Protocole de messagerie pub/sub

## 🐛 Dépannage

### Les pairs ne se découvrent pas

1. Vérifiez que le relay est bien démarré et accessible
2. Vérifiez l'adresse du relay (IP + port + Peer ID)
3. Vérifiez les logs avec `RUST_LOG=debug`

### L'interface web ne se charge pas

1. Vérifiez que le port 3030 est libre
2. Vérifiez que le dossier `static/` existe
3. Vérifiez les logs du serveur web

### Les messages ne sont pas diffusés

1. Vérifiez qu'au moins un pair est connecté
2. Vérifiez les logs GossipSub
3. Attendez quelques secondes (propagation réseau)

## 📝 Améliorations futures

- [ ] Persistance des posts (base de données locale)
- [ ] Chiffrement des messages
- [ ] Système de réputation
- [ ] Support des médias (images, vidéos)
- [ ] Profils utilisateurs
- [ ] Hashtags et mentions
- [ ] Recherche de contenu
- [ ] WebRTC pour le relay plus efficace

## 📄 Licence

MIT License - Projet éducatif

## 🤝 Contribution

Ce projet est un prototype éducatif. N'hésitez pas à l'améliorer !

---

**Créé avec ❤️ et Rust 🦀**
