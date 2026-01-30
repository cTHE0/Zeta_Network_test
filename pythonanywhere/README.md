# Zeta Network - Déploiement PythonAnywhere

Ce dossier contient les fichiers nécessaires pour héberger Zeta Network sur PythonAnywhere (zetanet.org).

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ARCHITECTURE ZETA NETWORK                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   zetanet.org (PythonAnywhere)                                  │
│   ┌─────────────────────────────────────────┐                   │
│   │  • index.html (interface)               │                   │
│   │  • pkg/zeta_wasm.js (bridge JS)         │  ◄── Serveur     │
│   │  • pkg/zeta_wasm_bg.wasm (248KB)        │      statique    │
│   │    (Code Rust compilé)                  │                   │
│   └─────────────────────────────────────────┘                   │
│                          │                                       │
│                          │ Téléchargement WASM                  │
│                          ▼                                       │
│   ┌─────────────────────────────────────────┐                   │
│   │          NAVIGATEUR CLIENT              │                   │
│   │  ┌───────────────────────────────────┐  │                   │
│   │  │  Code Rust WASM s'exécute ICI     │  │                   │
│   │  │  • Génération clés ed25519        │  │                   │
│   │  │  • Signature des messages         │  │                   │
│   │  │  • Logique P2P                    │  │                   │
│   │  └───────────────────────────────────┘  │                   │
│   └─────────────────────────────────────────┘                   │
│                          │                                       │
│                          │ WebSocket                            │
│                          ▼                                       │
│   ┌─────────────────────────────────────────┐                   │
│   │      RELAY P2P (ServerCheap)            │                   │
│   │      65.75.201.11:3030                  │                   │
│   │  • Transfert messages entre pairs       │                   │
│   │  • Ne stocke pas les données           │                   │
│   │  • Peut être remplacé par n'importe    │                   │
│   │    quel autre relay                     │                   │
│   └─────────────────────────────────────────┘                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Fichiers à uploader sur PythonAnywhere

```
pythonanywhere/
├── flask_app.py          # Application Flask
├── index.html            # Interface HTML avec WASM
└── pkg/                  # Fichiers WASM compilés
    ├── zeta_wasm.js      # Bridge JavaScript (27KB)
    ├── zeta_wasm_bg.wasm # Code Rust compilé (248KB)
    ├── zeta_wasm.d.ts    # Types TypeScript
    ├── zeta_wasm_bg.wasm.d.ts
    └── package.json
```

## Instructions de déploiement

### 1. Créer le compte PythonAnywhere

1. Allez sur https://www.pythonanywhere.com
2. Créez un compte gratuit
3. Allez dans **Web** > **Add a new web app**
4. Choisissez **Flask** et **Python 3.10**

### 2. Configuration du domaine (optionnel)

Si vous avez un domaine personnalisé (zetanet.org):
1. Allez dans **Web** > Votre application
2. Dans **Domain**, ajoutez votre domaine
3. Configurez les DNS de votre domaine pour pointer vers PythonAnywhere

### 3. Uploader les fichiers

Via l'interface PythonAnywhere (Files):

1. **Remplacez** `/home/votre_username/mysite/flask_app.py` par le contenu de `flask_app.py`
2. **Uploadez** `index.html` dans `/home/votre_username/mysite/`
3. **Créez** le dossier `/home/votre_username/mysite/pkg/`
4. **Uploadez** tous les fichiers du dossier `pkg/` :
   - `zeta_wasm.js`
   - `zeta_wasm_bg.wasm`
   - `zeta_wasm.d.ts`
   - `zeta_wasm_bg.wasm.d.ts`
   - `package.json`

### 4. Recharger l'application

1. Allez dans **Web**
2. Cliquez sur **Reload** pour votre application

### 5. Tester

Ouvrez votre site (ex: `votre_username.pythonanywhere.com` ou `zetanet.org`)

Vous devriez voir:
1. L'écran de chargement avec les étapes
2. Le WASM se télécharge (~248KB)
3. Votre identité ed25519 est générée
4. Connexion au relay P2P

## Vérification

Ouvrez la console du navigateur (F12) pour voir:
```
🚀 Zeta Network - Démarrage avec WASM...
📡 Relais connus: [{...}]
📦 Chargement du module WASM...
🔑 Génération de l'identité ed25519...
✅ Zeta Network prêt!
```

## Ajout de nouveaux relais

Pour ajouter des relais, modifiez la variable `KNOWN_RELAYS` dans:
- `index.html` (côté client)
- `flask_app.py` (pour l'API `/api/relays`)

```javascript
window.KNOWN_RELAYS = [
    {
        name: "ServerCheap Primary",
        ws: "ws://65.75.201.11:3030/ws",
        api: "http://65.75.201.11:3030/api",
        location: "USA"
    },
    {
        name: "Europe Relay",
        ws: "ws://eu.zetanet.org:3030/ws",
        api: "http://eu.zetanet.org:3030/api",
        location: "EU"
    }
];
```

## Important

- Le fichier WASM (248KB) contient le **vrai code Rust** compilé
- Les clés ed25519 sont générées **localement** dans le navigateur
- Les clés sont stockées dans **localStorage** (persistantes)
- Les relais ne peuvent **pas** lire vos messages (chiffrement E2E à venir)
- Vous pouvez ajouter autant de relais que vous voulez pour la redondance
