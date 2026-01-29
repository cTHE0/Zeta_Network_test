# Configuration exemple pour Zeta2

## Exemple de déploiement avec 2 PC + 1 serveur

### Serveur (IP publique : 203.0.113.42)
```bash
# Sur le serveur
cd zeta2
cargo run --release -- --relay

# La console affichera quelque chose comme :
# 🎧 Écoute sur: /ip4/203.0.113.42/tcp/4001/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
```

### PC 1 (derrière NAT)
```bash
cd zeta2
cargo run --release -- --relay-addr "/ip4/203.0.113.42/tcp/4001/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X"

# Ouvrir le navigateur
# http://localhost:3030
```

### PC 2 (derrière NAT)
```bash
cd zeta2
cargo run --release -- --relay-addr "/ip4/203.0.113.42/tcp/4001/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X"

# Ouvrir le navigateur
# http://localhost:3030
```

## Configuration réseau local (même LAN)

Si tous les appareils sont sur le même réseau local :

```bash
# Sur chaque machine
cd zeta2
cargo run --release

# La découverte automatique via mDNS fera le reste !
```

## Variables d'environnement utiles

```bash
# Logs détaillés
export RUST_LOG=debug
cargo run --release

# Logs très détaillés (pour déboguer)
export RUST_LOG=trace
cargo run --release

# Logs normaux
export RUST_LOG=info
cargo run --release
```

## Firewall / Sécurité

### Sur le serveur relay

Ouvrez le port TCP 4001 :

```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 4001/tcp

# firewalld (CentOS/RHEL)
sudo firewall-cmd --permanent --add-port=4001/tcp
sudo firewall-cmd --reload

# iptables
sudo iptables -A INPUT -p tcp --dport 4001 -j ACCEPT
```

### Sur les clients

Aucune configuration firewall nécessaire (connexions sortantes uniquement).

## Performance et optimisation

### Limiter l'utilisation mémoire

Le système garde max 1000 posts en mémoire. Pour modifier :

Éditez `src/main.rs`, ligne ~60 :
```rust
if posts.len() > 1000 {  // Changez cette valeur
    posts.truncate(1000);
}
```

### Ajuster la fréquence de rafraîchissement

L'interface web se rafraîchit toutes les 2 secondes.

Éditez `static/app.js`, ligne ~28 :
```javascript
setInterval(fetchNetworkInfo, 2000); // Changez la valeur en ms
```

## Troubleshooting

### "Address already in use" sur le port 3030

Un autre processus utilise le port. Changez le port dans `src/web_server.rs` :

```rust
warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
                                                 ^^^^ Changez ce numéro
```

### Les pairs ne se connectent pas

1. Vérifiez que le serveur relay est accessible (ping, telnet)
2. Vérifiez que vous avez copié l'adresse complète avec le Peer ID
3. Activez les logs debug : `RUST_LOG=debug cargo run`
4. Vérifiez le firewall

### Compilation lente

La première compilation prend du temps (dépendances). Utilisez :

```bash
# Compilation parallèle plus rapide
cargo build --release -j $(nproc)
```

## Commandes utiles

```bash
# Compiler sans lancer
cargo build --release

# Nettoyer et recompiler
cargo clean && cargo build --release

# Vérifier le code sans compiler
cargo check

# Lancer les tests (si ajoutés)
cargo test

# Voir la taille du binaire
ls -lh target/release/zeta2
```
