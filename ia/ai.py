# ai_mock_service.py
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import random
import re
from typing import List
import uvicorn

app = FastAPI(title="Mock AI Service for Patent Platform")

# Configuration CORS pour autoriser les requêtes depuis votre backend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:8080", "http://127.0.0.1:8080"],
    allow_methods=["POST"],
    allow_headers=["*"],
)

class AiRequest(BaseModel):
    raw_idea: str

class AiResponse(BaseModel):
    title: str
    problem: str
    solution: str
    claims: List[str]
    cpc_code: str
    novelty_score: int

# Mots-clés pour la génération de codes CPC
cpc_categories = {
    "agriculture": "A01",
    "medical": "A61",
    "transport": "B60",
    "computing": "G06",
    "energy": "F03",
    "construction": "E04",
    "education": "G09",
    "environment": "B01D",
    "food": "A23",
    "electronics": "H01"
}

def generate_cpc_code(text):
    text_lower = text.lower()
    for category, code in cpc_categories.items():
        if category in text_lower:
            # Ajouter un suffixe numérique aléatoire pour compléter le code CPC
            return f"{code}{random.randint(1, 99):02d}"
    # Code par défaut si aucune catégorie n'est trouvée
    return f"G06{random.randint(10, 99):02d}"

def extract_keywords(text):
    """Extrait des mots-clés pertinents de l'idée"""
    words = re.findall(r'\b[a-zA-Z]{4,}\b', text)
    return [word.lower() for word in words if word.lower() not in ['that', 'this', 'with', 'have', 'which']]

@app.post("/ai/structure", response_model=AiResponse)
async def structure_patent(request: AiRequest):
    try:
        keywords = extract_keywords(request.raw_idea)
        
        # Générer un titre basé sur les mots-clés
        title = " ".join(keywords[:4]).title() if keywords else "Innovative Solution"
        
        # Générer une description du problème
        problem = f"The problem addressed is related to {', '.join(keywords[:3])}." if keywords else "A common problem in the field."
        
        # Générer une solution
        solution = f"This invention provides a solution for {title} by implementing a novel approach that improves efficiency and reduces costs."
        
        # Générer des revendications
        claims = [
            f"A system for {title} comprising components A, B and C.",
            f"The system of claim 1, wherein component A is configured to optimize performance.",
            f"A method for implementing {title} using the system of claim 1."
        ]
        
        # Générer un code CPC
        cpc_code = generate_cpc_code(request.raw_idea)
        
        # Générer un score de nouveauté (entre 50 et 95)
        novelty_score = random.randint(50, 95)
        
        return AiResponse(
            title=title,
            problem=problem,
            solution=solution,
            claims=claims,
            cpc_code=cpc_code,
            novelty_score=novelty_score
        )
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error processing request: {str(e)}")

@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "Mock AI Service"}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)