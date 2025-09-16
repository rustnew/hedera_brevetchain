// Gestion de la navigation par onglets
document.addEventListener('DOMContentLoaded', function() {
    // Navigation par onglets
    const navLinks = document.querySelectorAll('.nav-link');
    const tabContents = document.querySelectorAll('.tab-content');
    
    navLinks.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            const tabId = this.getAttribute('data-tab');
            
            // Mettre à jour la navigation
            navLinks.forEach(l => l.classList.remove('active'));
            this.classList.add('active');
            
            // Afficher le contenu correspondant
            tabContents.forEach(content => content.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            
            // Charger les brevets si on ouvre le tableau de bord
            if (tabId === 'dashboard') {
                loadPatents();
            }
        });
    });
    
    // Soumission du formulaire de brevet
    const patentForm = document.getElementById('patentForm');
    const resultSection = document.getElementById('result');
    const alertBox = document.getElementById('alert');
    
    patentForm.addEventListener('submit', async function(e) {
        e.preventDefault();
        
        // Afficher l'indicateur de chargement
        showAlert('Traitement en cours...', 'info');
        
        // Récupérer les données du formulaire
        const formData = {
            user: {
                full_name: document.getElementById('fullName').value,
                email: document.getElementById('email').value,
                wallet_address: document.getElementById('walletAddress').value
            },
            patent: {
                raw_idea: document.getElementById('rawIdea').value
            }
        };
        
        try {
            // Envoyer la requête au backend
            const response = await fetch('http://localhost:8080/api/submit', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(formData)
            });
            
            if (!response.ok) {
                throw new Error(`Erreur HTTP: ${response.status}`);
            }
            
            const result = await response.json();
            
            // Afficher les résultats générés par l'IA
            document.getElementById('patentTitle').value = result.structured_patent.title;
            document.getElementById('patentClaims').value = result.structured_patent.claims.join('\n\n');
            document.getElementById('patentSummary').value = result.structured_patent.summary;
            
            // Afficher la section des résultats
            resultSection.classList.remove('hidden');
            
            // Stocker l'ID du brevet pour l'enregistrement blockchain
            resultSection.setAttribute('data-patent-id', result.patent_id);
            
            showAlert('Brevet généré avec succès par l\'IA!', 'success');
            
        } catch (error) {
            console.error('Erreur:', error);
            showAlert('Erreur lors de la génération du brevet: ' + error.message, 'error');
        }
    });
    
    // Enregistrement sur la blockchain
    const submitBlockchainBtn = document.getElementById('submitBlockchain');
    
    submitBlockchainBtn.addEventListener('click', async function() {
        const patentId = resultSection.getAttribute('data-patent-id');
        
        if (!patentId) {
            showAlert('Aucun brevet à enregistrer', 'error');
            return;
        }
        
        showAlert('Enregistrement sur la blockchain en cours...', 'info');
        submitBlockchainBtn.disabled = true;
        
        try {
            // Envoyer la requête au backend pour l'enregistrement blockchain
            const response = await fetch('http://localhost:8080/api/register-blockchain', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ patent_id: patentId })
            });
            
            if (!response.ok) {
                throw new Error(`Erreur HTTP: ${response.status}`);
            }
            
            const result = await response.json();
            
            // Message de succès
            showAlert('Brevet enregistré avec succès sur la blockchain Hedera! Transaction hash: ' + result.transaction_hash, 'success');
            
            // Réinitialiser le formulaire
            patentForm.reset();
            resultSection.classList.add('hidden');
            
            // Recharger la liste des brevets
            loadPatents();
            
        } catch (error) {
            console.error('Erreur blockchain:', error);
            showAlert('Erreur lors de l\'enregistrement blockchain: ' + error.message, 'error');
        } finally {
            submitBlockchainBtn.disabled = false;
        }
    });
    
    // Fonction pour afficher les alertes
    function showAlert(message, type) {
        alertBox.textContent = message;
        alertBox.className = 'alert';
        alertBox.classList.add(type);
        alertBox.classList.remove('hidden');
        
        // Cacher automatiquement après 5 secondes
        setTimeout(() => {
            alertBox.classList.add('hidden');
        }, 5000);
    }
    
    // Fonction pour charger les brevets depuis l'API
    async function loadPatents() {
        const patentsList = document.getElementById('patentsList');
        
        // Afficher un indicateur de chargement
        patentsList.innerHTML = '<p class="empty-state">Chargement des brevets...</p>';
        
        try {
            // Récupérer les brevets depuis l'API
            const response = await fetch('http://localhost:8080/api/patents');
            
            if (!response.ok) {
                throw new Error(`Erreur HTTP: ${response.status}`);
            }
            
            const patents = await response.json();
            
            // Vérifier s'il y a des brevets
            if (patents.length === 0) {
                patentsList.innerHTML = '<p class="empty-state">Aucun brevet déposé pour le moment.</p>';
                return;
            }
            
            // Générer le HTML pour chaque brevet
            patentsList.innerHTML = patents.map(patent => `
                <div class="patent-item">
                    <h3>${patent.title}</h3>
                    <div class="patent-meta">
                        <span>Déposé le: ${new Date(patent.created_at).toLocaleDateString('fr-FR')}</span>
                        <span class="patent-status status-${patent.status}">${getStatusText(patent.status)}</span>
                    </div>
                    <p>${patent.summary}</p>
                    ${patent.transaction_hash ? `<div class="patent-meta">
                        <span>Hash de transaction: ${patent.transaction_hash}</span>
                    </div>` : ''}
                </div>
            `).join('');
            
        } catch (error) {
            console.error('Erreur lors du chargement des brevets:', error);
            patentsList.innerHTML = '<p class="empty-state">Erreur lors du chargement des brevets.</p>';
        }
    }
    
    // Fonction utilitaire pour obtenir le texte du statut
    function getStatusText(status) {
        const statusMap = {
            'draft': 'Brouillon',
            'submitted': 'Soumis',
            'onblockchain': 'Sur la blockchain',
            'rejected': 'Rejeté'
        };
        return statusMap[status] || status;
    }
    
    // Charger les brevets au chargement initial si on est sur le tableau de bord
    if (document.getElementById('dashboard').classList.contains('active')) {
        loadPatents();
    }
    
    // Permettre aux boutons de navigation de changer d'onglet
    document.querySelectorAll('[data-tab]').forEach(element => {
        if (element.tagName === 'BUTTON') {
            element.addEventListener('click', function() {
                const tabId = this.getAttribute('data-tab');
                document.querySelector(`.nav-link[data-tab="${tabId}"]`).click();
            });
        }
    });
});