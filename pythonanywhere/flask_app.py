# ============================================
# Zeta Network - Serveur Flask pour PythonAnywhere
# ============================================
# 
# Ce fichier sert l'interface web et les fichiers WASM compilés.
# Le code Rust est exécuté localement dans le navigateur via WebAssembly.
#
# Instructions pour PythonAnywhere:
# 1. Créez un compte sur pythonanywhere.com
# 2. Allez dans Web > Add a new web app > Flask
# 3. Remplacez le contenu de flask_app.py par ce fichier
# 4. Uploadez index.html à la racine (même dossier que flask_app.py)
# 5. Créez un dossier "pkg" et uploadez les fichiers WASM dedans:
#    - pkg/zeta_wasm.js
#    - pkg/zeta_wasm_bg.wasm
#    - pkg/zeta_wasm.d.ts
#    - pkg/zeta_wasm_bg.wasm.d.ts
#    - pkg/package.json
# 6. Rechargez l'application

from flask import Flask, send_from_directory, make_response
import os

app = Flask(__name__, static_folder='.')

# Configuration des relais
KNOWN_RELAYS = [
    {
        "name": "ServerCheap Primary",
        "ws": "ws://65.75.201.11:3030/ws",
        "api": "http://65.75.201.11:3030/api",
        "location": "USA"
    }
]

@app.route('/')
def index():
    """Page principale - Interface Zeta Network avec WASM"""
    return send_from_directory('.', 'index.html')

@app.route('/pkg/<path:filename>')
def serve_wasm(filename):
    """Sert les fichiers WASM avec les bons headers MIME"""
    response = make_response(send_from_directory('pkg', filename))
    
    # Headers CORS pour permettre le chargement WASM
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Methods'] = 'GET, OPTIONS'
    response.headers['Access-Control-Allow-Headers'] = 'Content-Type'
    
    # Type MIME correct pour WASM
    if filename.endswith('.wasm'):
        response.headers['Content-Type'] = 'application/wasm'
    elif filename.endswith('.js'):
        response.headers['Content-Type'] = 'application/javascript'
    
    return response

@app.route('/health')
def health():
    """Endpoint de santé"""
    return {'status': 'ok', 'relays': KNOWN_RELAYS}

@app.route('/api/relays')
def get_relays():
    """Retourne la liste des relais connus"""
    return {'relays': KNOWN_RELAYS}

# Pour développement local
if __name__ == '__main__':
    app.run(debug=True, port=5000)
