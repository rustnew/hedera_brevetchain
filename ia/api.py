# api.py
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import time
from contextlib import asynccontextmanager
import logging

# === Import du moteur IA ===
from ai import get_patent_generator

# === Configuration du logging ===
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# === Gestionnaire de cycle de vie (CORRIGÉ) ===
@asynccontextmanager
async def lifespan(app: FastAPI):  # ← async ajouté ici !!
    logger.info("Pré-chargement du modèle IA...")
    _ = get_patent_generator()  # Charge le modèle
    yield
    logger.info("Arrêt de l'application")

# === Initialisation de l'API ===
app = FastAPI(
    title="Générateur de Brevets IA",
    description="Transforme une idée en un brevet complet et structuré en une seule requête",
    version="1.0.0",
    lifespan=lifespan
)

# === Middleware CORS ===
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# === Modèle d'entrée ===
class IdeaRequest(BaseModel):
    idea: str = Field(..., min_length=5, max_length=500, description="Votre idée innovante")

# === Modèle de réponse ===
class PatentResponse(BaseModel):
    full_patent: str = Field(..., description="Texte complet du brevet, formaté comme un document officiel")

# === Route principale ===
@app.post("/generate", response_model=PatentResponse)
async def generate_full_patent(request: IdeaRequest):
    """
    Génère un brevet complet à partir d'une idée.
    """
    start_time = time.time()
    idea = request.idea.strip()

    if len(idea.split()) < 2:
        raise HTTPException(status_code=400, detail="L'idée doit contenir au moins 2 mots.")

    try:
        generator = get_patent_generator()
        structured = generator.generate_patent_structured(idea)

        # === Génère un brevet bien formaté ===
        full_text = f"""
OFFICE DES BREVETS INTELLECTUELS
=================================

BREVET D'INVENTION - DOCUMENT PROVISOIRE
Généré par IA - Preuve d'antériorité possible via Hedera (à intégrer)

┌──────────────────────────────────────────────────────────────────────┐
│                     FICHE D'INVENTION                                │
└──────────────────────────────────────────────────────────────────────┘

TITRE :
{structured.get('title', 'Non disponible')}

PROBLÈME TECHNIQUE RÉSOLU :
{structured.get('problem', 'Non disponible')}

DESCRIPTION DE L'INVENTION :
{structured.get('solution', 'Non disponible')}

REVENDICATIONS :
{chr(10).join(f"{i+1}. {claim}" for i, claim in enumerate(structured.get('claims', [])[:5]))}

CLASSIFICATION CPC :
{structured.get('cpc_code', 'Non disponible')}

SCORE DE NOVETÉ : {structured.get('novelty_score', 'N/A')}/100

IDÉE ORIGINALE :
"{idea}"

TEMPS DE GÉNÉRATION : {round(time.time() - start_time, 2)} secondes
MODELE IA : {generator.model_name}
DEVICE : {generator._device}
"""
        full_text = "\n".join(line.strip() for line in full_text.splitlines() if line.strip() or line == "\n")
        return {"full_patent": full_text}

    except Exception as e:
        logger.error(f"Erreur lors de la génération : {e}")
        raise HTTPException(status_code=500, detail="Erreur interne lors de la génération du brevet")

# === Point d'entrée ===
if __name__ == "__main__":
    import uvicorn
    uvicorn.run("api:app", host="0.0.0.0", port=8000, reload=True)