# ============================================
# Zeta Network - Serveur Flask pour PythonAnywhere
# ============================================
# 
# Ce fichier sert l'interface web depuis PythonAnywhere.
# Le client JavaScript se connecte au relay P2P via WebSocket.
#
# Instructions pour PythonAnywhere:
# 1. Créez un compte sur pythonanywhere.com
# 2. Allez dans Web > Add a new web app > Flask
# 3. Remplacez le contenu de flask_app.py par ce fichier
# 4. Uploadez index.html dans le dossier templates/
# 5. Rechargez l'application

from flask import Flask, render_template, send_from_directory
import os

app = Flask(__name__)

# Configuration
RELAY_HOST = "65.75.201.11"
RELAY_PORT = "3030"

@app.route('/')
def index():
    """Page principale - Interface Zeta Network"""
    return render_template('index.html', 
                          relay_ws=f"ws://{RELAY_HOST}:{RELAY_PORT}/ws",
                          relay_api=f"http://{RELAY_HOST}:{RELAY_PORT}/api")

@app.route('/health')
def health():
    """Endpoint de santé"""
    return {'status': 'ok', 'relay': f"{RELAY_HOST}:{RELAY_PORT}"}

# Pour développement local
if __name__ == '__main__':
    app.run(debug=True, port=5000)
