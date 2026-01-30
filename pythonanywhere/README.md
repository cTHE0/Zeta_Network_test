# Zeta Network - Fichiers pour PythonAnywhere

Ce dossier contient les fichiers nécessaires pour héberger l'interface web de Zeta Network sur PythonAnywhere.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ┌──────────────────┐                                           │
│  │  zetanet.org     │  PythonAnywhere (Flask)                   │
│  │  (votre site)    │  Sert: HTML + CSS + JS                    │
│  └────────┬─────────┘                                           │
│           │ L'utilisateur charge la page                        │
│           ▼                                                     │
│  ┌──────────────────┐                                           │
│  │  Navigateur      │  Le JavaScript se connecte directement    │
│  │  de l'utilisateur│  au relay via WebSocket                   │
│  └────────┬─────────┘                                           │
│           │ WebSocket (ws://65.75.201.11:3030/ws)               │
│           ▼                                                     │
│  ┌──────────────────┐     ┌──────────────────┐                  │
│  │  Relay P2P       │◀───▶│  Autres nœuds    │                  │
│  │  ServerCheap     │     │  P2P natifs      │                  │
│  │  65.75.201.11    │     └──────────────────┘                  │
│  └──────────────────┘                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Installation sur PythonAnywhere

### Méthode 1: Fichier HTML unique (plus simple)

1. Allez sur [PythonAnywhere](https://www.pythonanywhere.com)
2. Créez un compte gratuit
3. Allez dans **Web** > **Add a new web app**
4. Choisissez **Manual configuration** > **Python 3.10**
5. Dans la section **Static files**, ajoutez:
   - URL: `/`
   - Directory: `/home/votre_username/`
6. Uploadez `index.html` dans `/home/votre_username/`
7. Renommez-le en `index.html`

### Méthode 2: Application Flask (plus flexible)

1. Créez une web app Flask
2. Remplacez `flask_app.py` par le contenu fourni
3. Créez un dossier `templates/`
4. Copiez `index.html` dans `templates/`
5. Modifiez l'URL du relay si nécessaire
6. Rechargez l'application

## Configuration

Dans `index.html`, modifiez ces valeurs selon votre configuration:

```javascript
window.RELAY_CONFIG = {
    // Adresse WebSocket de votre relay P2P
    websocket: "ws://65.75.201.11:3030/ws",
    // API REST du relay
    api: "http://65.75.201.11:3030/api"
};
```

## Prérequis

Le **relay P2P** (sur ServerCheap ou autre VPS) doit être:
- En cours d'exécution (`./target/release/zeta2 --relay`)
- Accessible sur le port 3030 (firewall ouvert)
- Configuré avec CORS activé (déjà inclus dans le code)

## Test

1. Ouvrez votre site (ex: `zetanet.pythonanywhere.com`)
2. Vous devriez voir "Connexion au relay..."
3. Une fois connecté, vous pouvez poster des messages
4. Les messages sont relayés à tous les autres clients via P2P

## Limitations

- **PythonAnywhere gratuit** ne supporte pas les WebSockets sortants depuis le serveur
- Le client JavaScript se connecte **directement** au relay (pas via PythonAnywhere)
- Le relay doit avoir un certificat SSL si vous voulez utiliser `wss://` (HTTPS)

## HTTPS/WSS

Pour une connexion sécurisée:
1. Configurez un certificat SSL sur votre relay (Let's Encrypt)
2. Modifiez l'URL: `wss://votre-relay.com:3030/ws`
3. PythonAnywhere utilise HTTPS par défaut, donc le WebSocket doit aussi être sécurisé
