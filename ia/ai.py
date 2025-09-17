from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import random
import re
from typing import List, Set
import uvicorn

app = FastAPI(title="Enhanced MVP AI Service for Patent Platform")

# CORS: autorise les requêtes du backend Rust
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:8080", "http://127.0.0.1:8080", "*"],
    allow_methods=["POST", "GET"],
    allow_headers=["*"],
)

class AiRequest(BaseModel):
    raw_idea: str

class AiResponse(BaseModel):
    title: str
    problem: str
    solution: str
    claim: str  # ✅ MVP: une seule revendication principale
    cpc_code: str
    novelty_score: int

# Base de connaissances enrichie pour les codes CPC
cpc_semantic_map = {
    # Domaines techniques → codes CPC
    "agriculture": ["A01B", "A01C", "A01D", "A01F"],
    "medical": ["A61B", "A61C", "A61D", "A61F", "A61H", "A61J", "A61K", "A61L", "A61M", "A61N"],
    "health": ["A61B", "A61H", "A61N"],
    "transport": ["B60", "B61", "B62", "B63", "B64"],
    "vehicle": ["B60", "B62", "B64"],
    "computing": ["G06F", "G06N", "G06Q", "G06T"],
    "software": ["G06F8", "G06F11", "G06F21"],
    "algorithm": ["G06N", "G06F17"],
    "energy": ["F03", "H02J", "H02K", "H02M"],
    "power": ["H02J", "H02M"],
    "construction": ["E04B", "E04C", "E04D", "E04F", "E04G"],
    "building": ["E04B", "E04H"],
    "education": ["G09B", "G09C"],
    "learning": ["G09B"],
    "environment": ["B01D", "B01F", "B01J", "B09B", "C02F"],
    "filter": ["B01D"],
    "food": ["A23B", "A23C", "A23D", "A23F", "A23G", "A23L"],
    "electronics": ["H01L", "H01R", "H01T", "H02M"],
    "circuit": ["H03K", "H03M", "H04L"],
    "communication": ["H04L", "H04N", "H04W"],
    "network": ["H04L", "H04W"],
    "mechanical": ["F16B", "F16C", "F16D", "F16H", "F16K"],
    "tool": ["B25B", "B25C", "B25D", "B25F", "B25G", "B25H"],
    "textile": ["D03D", "D04B", "D06F", "D06H"],
    "chemistry": ["C07C", "C07D", "C07F", "C07H"],
    "biology": ["C12M", "C12N", "C12P", "C12Q"],
    "physics": ["G01R", "G01S", "G01T", "G01V"],
    "measurement": ["G01B", "G01D", "G01F", "G01K", "G01L", "G01M", "G01N", "G01P", "G01Q", "G01R", "G01S"],
    "container": ["B65D", "B65F", "B67D"],
    "kitchen": ["A47J", "A47G"],
    "lighting": ["F21S", "F21V", "H05B"],
    "security": ["G08B", "H04L9", "G06F21"],
    "toy": ["A63H", "A63B"],
}

# Mots techniques pour enrichir les descriptions
technical_verbs = [
    "optimiser", "réduire", "augmenter", "stabiliser", "automatiser", "synchroniser",
    "moduler", "réguler", "convertir", "transmettre", "détecter", "mesurer", "analyser"
]

technical_nouns = [
    "efficacité", "fiabilité", "durabilité", "précision", "sécurité", "performance",
    "coût", "consommation", "temps", "ergonomie", "maintenance", "compatibilité"
]

def extract_keywords(text: str) -> List[str]:
    """Extrait les mots-clés significatifs (noms, verbes techniques) de l'idée."""
    if not text or len(text.strip()) < 3:
        return ["solution", "technique", "innovation"]
    
    text = text.lower()
    # Supprime la ponctuation et chiffres isolés
    text = re.sub(r'[^\w\s]', ' ', text)
    text = re.sub(r'\b\d+\b', ' ', text)
    words = text.split()
    
    # Filtre les mots courts ou trop communs
    stop_words = {
        'this', 'that', 'with', 'have', 'which', 'also', 'they', 'them', 'the', 'and',
        'for', 'are', 'not', 'but', 'can', 'will', 'would', 'could', 'should', 'may',
        'ces', 'les', 'des', 'est', 'pas', 'que', 'qui', 'dans', 'pour', 'plus', 'tout'
    }
    
    keywords = [
        word for word in words
        if len(word) >= 3 and word not in stop_words
        and (word in technical_verbs or word in technical_nouns or len(word) > 4)
    ]
    
    # Supprime les doublons en conservant l'ordre
    seen = set()
    unique_keywords = []
    for word in keywords:
        if word not in seen:
            seen.add(word)
            unique_keywords.append(word)
    
    return unique_keywords[:8]  # Limite à 8 mots-clés pertinents

def generate_cpc_code(text: str) -> str:
    """Génère un code CPC basé sur une analyse sémantique améliorée."""
    keywords = extract_keywords(text)
    text_lower = text.lower()
    
    # Vérifie chaque domaine sémantique
    for domain, codes in cpc_semantic_map.items():
        if domain in text_lower or any(kw in text_lower for kw in [domain]):
            return random.choice(codes) + str(random.randint(10, 99))
    
    # Vérifie si des mots-clés appartiennent à un domaine
    for keyword in keywords:
        for domain, codes in cpc_semantic_map.items():
            if keyword in domain or domain in keyword:
                return random.choice(codes) + str(random.randint(10, 99))
    
    # Fallback par mots-clés génériques
    if any(kw in text_lower for kw in ["device", "system", "method", "apparatus"]):
        return "G06F" + str(random.randint(10, 99))
    elif any(kw in text_lower for kw in ["machine", "tool", "mechanism"]):
        return "B25B" + str(random.randint(10, 99))
    else:
        return "A47G" + str(random.randint(10, 99))  # Objet domestique par défaut

def generate_title(keywords: List[str]) -> str:
    """Génère un titre technique et descriptif."""
    if not keywords:
        return "Dispositif Technique Innovant"
    
    # Construit un titre basé sur le premier mot-clé + contexte
    main_keyword = keywords[0]
    if len(keywords) > 1:
        context = keywords[1]
        return f"Dispositif de {main_keyword} avec {context} amélioré"
    else:
        return f"Système de {main_keyword} optimisé"

def generate_problem(keywords: List[str]) -> str:
    """Génère une description du problème technique réaliste."""
    if not keywords:
        return "Les solutions existantes dans ce domaine présentent des limitations en termes d'efficacité, de coût ou de fiabilité, nécessitant une approche innovante pour une amélioration significative."
    
    main_keyword = keywords[0]
    if len(keywords) > 1:
        secondary = keywords[1]
        return f"Les dispositifs actuels de {main_keyword} souffrent de limitations liées à {secondary}, entraînant une inefficacité opérationnelle, des coûts élevés ou une fiabilité insuffisante."
    else:
        return f"L'état de la technique dans le domaine de {main_keyword} présente des lacunes en matière de performance et d'optimisation, nécessitant une solution innovante."

def generate_solution(keywords: List[str]) -> str:
    """Génère une description de la solution technique précise."""
    if not keywords:
        return "L'invention propose un système novateur intégrant des moyens techniques avancés pour optimiser les performances tout en réduisant les coûts de fabrication et d'utilisation."
    
    main_keyword = keywords[0]
    verb = random.choice(technical_verbs)
    noun = random.choice(technical_nouns)
    
    if len(keywords) > 1:
        mechanism = keywords[1]
        return f"Cette invention résout le problème en introduisant un mécanisme de {mechanism} qui permet de {verb} la {noun} de manière significative, grâce à une configuration technique innovante."
    else:
        return f"L'invention propose une approche basée sur {main_keyword} qui permet de {verb} la {noun} par l'intégration de composants techniques optimisés et d'un procédé de contrôle précis."

def generate_claim(title: str, keywords: List[str]) -> str:
    """Génère UNE revendication principale avec structure juridique correcte."""
    if not keywords:
        return "1. Dispositif technique caractérisé par des moyens configurés pour optimiser les performances opérationnelles tout en réduisant les coûts de fabrication."
    
    main_keyword = keywords[0]
    if len(keywords) > 1:
        feature = keywords[1]
        return f"1. Dispositif pour {title.lower()} comprenant au moins un module configuré pour {main_keyword}, caractérisé en ce que ledit module intègre des moyens d'optimisation de {feature} agencés de manière à améliorer la performance globale du système."
    else:
        verb = random.choice(technical_verbs)
        noun = random.choice(technical_nouns)
        return f"1. Système technique comprenant des moyens configurés pour {verb} la {noun}, caractérisé en ce que lesdits moyens sont agencés selon une architecture innovante permettant une amélioration significative par rapport à l'état de la technique."

def calculate_novelty_score(keywords: List[str]) -> int:
    """Calcule un score de nouveauté plus réaliste basé sur la spécificité de l'idée."""
    if not keywords:
        return 55  # Score bas pour idée vide
    
    # Score de base
    base_score = 65
    
    # Bonus pour nombre de mots-clés uniques
    keyword_bonus = min(len(keywords) * 4, 20)
    
    # Bonus pour présence de verbes/noms techniques
    technical_bonus = 0
    for kw in keywords:
        if kw in technical_verbs or kw in technical_nouns:
            technical_bonus += 3
    technical_bonus = min(technical_bonus, 10)
    
    # Bonus aléatoire pour variabilité
    random_bonus = random.randint(-5, 5)
    
    score = base_score + keyword_bonus + technical_bonus + random_bonus
    return max(55, min(95, score))  # Garantit un score entre 55 et 95

@app.post("/ai/structure", response_model=AiResponse)
async def structure_patent(request: AiRequest):
    """
    Endpoint principal pour le MVP.
    Transforme une idée brute en structure de brevet simplifiée et améliorée.
    """
    try:
        if not request.raw_idea or len(request.raw_idea.strip()) < 3:
            # Génération dégradée mais cohérente pour idées très courtes
            return AiResponse(
                title="Invention Technique",
                problem="Problème technique général nécessitant une solution innovante.",
                solution="Solution technique basée sur des principes d'optimisation et d'efficacité.",
                claim="1. Dispositif technique caractérisé par ses moyens d'amélioration des performances.",
                cpc_code="G06F17",
                novelty_score=55
            )

        idea = request.raw_idea.strip()
        keywords = extract_keywords(idea)

        title = generate_title(keywords)
        problem = generate_problem(keywords)
        solution = generate_solution(keywords)
        claim = generate_claim(title, keywords)
        cpc_code = generate_cpc_code(idea)
        novelty_score = calculate_novelty_score(keywords)

        return AiResponse(
            title=title,
            problem=problem,
            solution=solution,
            claim=claim,
            cpc_code=cpc_code,
            novelty_score=novelty_score
        )

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Erreur interne du service IA: {str(e)}")

@app.get("/health")
async def health_check():
    """Endpoint de santé pour le monitoring."""
    return {"status": "healthy", "service": "Enhanced MVP AI Service"}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)