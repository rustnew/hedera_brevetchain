// DOM Elements
const loader = document.getElementById('loader');
const startBtn = document.getElementById('start-btn');
const registerSection = document.getElementById('register');
const submitIdeaSection = document.getElementById('submit-idea');
const aiSummarySection = document.getElementById('ai-summary');
const certificateSection = document.getElementById('certificate');
const registerForm = document.getElementById('register-form');
const submitIdeaBtn = document.getElementById('submit-idea-btn');
const registerProofBtn = document.getElementById('register-proof-btn');
const checkStatusBtn = document.getElementById('check-status-btn');
const recordingStatus = document.getElementById('recording-status');

let currentUser = null;
let currentIdeaId = null;
let currentSummaryId = null;

// Hide loader after 2s
window.addEventListener('load', () => {
    setTimeout(() => {
        loader.style.opacity = '0';
        setTimeout(() => {
            loader.style.display = 'none';
        }, 500);
    }, 2000);
});

// Start Button
startBtn.addEventListener('click', () => {
    document.querySelector('.hero-3d').scrollIntoView({ behavior: 'smooth' });
    setTimeout(() => {
        registerSection.classList.add('active');
    }, 500);
});

// Register Form
registerForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const userData = {
        full_name: document.getElementById('full_name').value,
        email: document.getElementById('email').value,
        phone: document.getElementById('phone').value || null,
        country: document.getElementById('country').value || null,
        wallet_address: document.getElementById('wallet_address').value
    };

    try {
        const response = await fetch('/api/v1/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(userData)
        });

        const result = await response.json();
        
        if (response.ok) {
            currentUser = result.user_id;
            showToast('✅ Compte créé avec succès !');
            registerSection.classList.remove('active');
            setTimeout(() => {
                submitIdeaSection.classList.add('active');
            }, 500);
        } else {
            throw new Error(result.message || 'Erreur inconnue');
        }
    } catch (error) {
        alert('Erreur : ' + error.message);
    }
});

// Submit Idea
submitIdeaBtn.addEventListener('click', async () => {
    const rawIdea = document.getElementById('raw_idea').value;
    
    if (!rawIdea.trim()) {
        alert('Veuillez décrire votre idée.');
        return;
    }

    try {
        const response = await fetch('/api/v1/submit-idea', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user_id: currentUser,
                raw_idea: rawIdea
            })
        });

        const result = await response.json();
        
        if (response.ok) {
            currentIdeaId = result.idea_id;
            showToast('💡 Idée enregistrée ! Génération du résumé IA...');
            submitIdeaSection.classList.remove('active');
            
            // Simulate AI processing delay
            setTimeout(async () => {
                await generateSummary();
            }, 2000);
        } else {
            throw new Error(result.message);
        }
    } catch (error) {
        alert('Erreur : ' + error.message);
    }
});

// Generate Summary
async function generateSummary() {
    try {
        const response = await fetch(`/api/v1/generate-summary/${currentIdeaId}`, {
            method: 'POST'
        });

        const result = await response.json();
        
        if (response.ok) {
            currentSummaryId = result.summary_id;
            
            // Fetch summary data
            const summaryResponse = await fetch(`/api/v1/summary/${currentSummaryId}`);
            const summaryData = await summaryResponse.json();
            
            // Populate UI
            document.querySelector('#title span').textContent = summaryData.title;
            document.querySelector('#problem').textContent = summaryData.problem;
            document.querySelector('#solution').textContent = summaryData.solution;
            document.querySelector('#claim').textContent = summaryData.claim;
            document.querySelector('#cpc_code').textContent = summaryData.cpc_code;
            
            aiSummarySection.classList.add('active');
        } else {
            throw new Error(result.message);
        }
    } catch (error) {
        alert('Erreur IA : ' + error.message);
    }
}

// Register Proof on Hedera
registerProofBtn.addEventListener('click', async () => {
    try {
        const response = await fetch(`/api/v1/register-proof/${currentSummaryId}`, {
            method: 'POST'
        });

        const result = await response.json();
        
        if (response.ok) {
            showToast('🔐 Preuve enregistrée sur Hedera !');
            aiSummarySection.classList.remove('active');
            
            // Simulate blockchain delay
            setTimeout(async () => {
                await getCertificate();
            }, 3000);
        } else {
            throw new Error(result.message);
        }
    } catch (error) {
        alert('Erreur Hedera : ' + error.message);
    }
});

// Get Certificate
async function getCertificate() {
    try {
        const response = await fetch(`/api/v1/certificate/${currentSummaryId}`);
        const certData = await response.json();
        
        document.getElementById('cert-hash').textContent = certData.hash;
        document.getElementById('cert-timestamp').textContent = certData.timestamp;
        document.getElementById('cert-link').href = certData.explorer_url;
        document.getElementById('cert-link').textContent = 'Voir sur HashScan';
        
        certificateSection.classList.add('active');
    } catch (error) {
        alert('Erreur certificat : ' + error.message);
    }
}

// Check Status
checkStatusBtn.addEventListener('click', async () => {
    try {
        const response = await fetch(`/api/v1/status/${currentIdeaId}`);
        const statusData = await response.json();
        
        // Update status steps (already completed in MVP flow)
        showToast('✅ Statut actualisé !');
    } catch (error) {
        alert('Erreur statut : ' + error.message);
    }
});

// Audio Recording (Placeholder)
document.getElementById('record-btn').addEventListener('click', () => {
    recordingStatus.textContent = '🔴 Enregistrement en cours... (fonctionnalité à implémenter)';
    setTimeout(() => {
        recordingStatus.textContent = '✅ Audio prêt à être transcrit !';
    }, 3000);
});

// Toast Notifications
function showToast(message) {
    const toast = document.createElement('div');
    toast.textContent = message;
    toast.style.position = 'fixed';
    toast.style.top = '20px';
    toast.style.right = '20px';
    toast.style.background = '#10b981';
    toast.style.color = 'white';
    toast.style.padding = '1rem 2rem';
    toast.style.borderRadius = '50px';
    toast.style.boxShadow = '0 5px 15px rgba(0,0,0,0.2)';
    toast.style.zIndex = '9999';
    document.body.appendChild(toast);
    
    setTimeout(() => {
        toast.style.opacity = '0';
        setTimeout(() => {
            document.body.removeChild(toast);
        }, 500);
    }, 3000);
}

// Smooth scroll for navigation
document.querySelectorAll('nav a').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const targetId = this.getAttribute('href');
        document.querySelector(targetId).scrollIntoView({
            behavior: 'smooth'
        });
    });
});

// 🌌 Constellation d'Intelligence — Version LUMINEUSE & FLUIDE
function createConstellation() {
    const canvas = document.createElement('div');
    canvas.id = 'constellation';
    document.body.appendChild(canvas);

    const particleCount = 120;
    const particles = [];
    const connections = [];

    // Créer les particules (étoiles IA — lumineuses)
    for (let i = 0; i < particleCount; i++) {
        const particle = document.createElement('div');
        particle.className = 'particle';
        
        // Position aléatoire
        const x = Math.random() * 100;
        const y = Math.random() * 100;
        particle.style.left = `${x}%`;
        particle.style.top = `${y}%`;
        
        // Taille aléatoire
        const size = 3 + Math.random() * 7;
        particle.style.width = `${size}px`;
        particle.style.height = `${size}px`;
        
        // Couleur aléatoire dans la palette bleu vif
        const colors = ['#00aaff', '#00ccff', '#00f5ff', '#66ccff'];
        const color = colors[Math.floor(Math.random() * colors.length)];
        particle.style.background = color;
        particle.style.boxShadow = `0 0 ${8 + size}px ${color}`;
        
        // Animation fluide
        particle.style.animationDelay = `${Math.random() * 5}s`;
        particle.style.animationDuration = `${8 + Math.random() * 10}s`;
        particle.style.animationTimingFunction = 'ease-in-out';
        
        canvas.appendChild(particle);
        particles.push({
            element: particle,
            x: x,
            y: y,
            size: size,
            color: color
        });
    }

    // Créer des connexions entre particules proches (effet réseau vivant)
    function createConnections() {
        // Nettoyer les anciennes connexions
        connections.forEach(conn => conn.element.remove());
        connections.length = 0;

        // Créer de nouvelles connexions
        for (let i = 0; i < particles.length; i++) {
            for (let j = i + 1; j < particles.length; j++) {
                const p1 = particles[i];
                const p2 = particles[j];
                
                // Calculer la distance
                const dx = (p1.x - p2.x) * window.innerWidth / 100;
                const dy = (p1.y - p2.y) * window.innerHeight / 100;
                const distance = Math.sqrt(dx * dx + dy * dy);

                // Connecter si proches
                if (distance < 150) {
                    const connection = document.createElement('div');
                    connection.className = 'connection';
                    
                    // Position et taille
                    connection.style.width = `${distance}px`;
                    
                    // Angle
                    const angle = Math.atan2(dy, dx) * 180 / Math.PI;
                    connection.style.transformOrigin = 'left center';
                    connection.style.transform = `rotate(${angle}deg)`;
                    
                    // Position
                    connection.style.left = `${p1.x}%`;
                    connection.style.top = `${p1.y}%`;
                    
                    // Opacité et couleur basées sur la distance
                    const opacity = 1 - (distance / 150);
                    connection.style.opacity = opacity * 0.7;
                    connection.style.background = `linear-gradient(90deg, transparent, ${p1.color}, transparent)`;
                    
                    canvas.appendChild(connection);
                    connections.push({
                        element: connection,
                        p1: p1,
                        p2: p2
                    });
                }
            }
        }
    }

    // Mettre à jour les connexions périodiquement pour un effet vivant
    setInterval(createConnections, 2500);
    
    // Créer les connexions initiales
    setTimeout(createConnections, 500);

    // Animation douce du fond pour un effet “respiration”
    let breath = 0;
    setInterval(() => {
        breath += 0.02;
        canvas.style.background = `
            radial-gradient(circle at 30% 30%, rgba(0, 204, 255, ${0.1 + 0.05 * Math.sin(breath)}), transparent 60%),
            radial-gradient(circle at 70% 70%, rgba(0, 245, 255, ${0.05 + 0.03 * Math.sin(breath + 1)}), transparent 60%)
        `;
    }, 100);
}