# GLOBALNEWS — Lightweight Natural Language Processing (NLP) Research

**Document ID:** `GN-DOC-P0-004`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Inspection Date:** 2026-11-09  
**Core Target:** High-precision intelligence extraction without GPUs or heavyweight LLMs, operating within $<350\text{ MB}$ RAM on Apple Silicon M1.

---

## 1. The Zero-Heavyweight-LLM Strategy

### 1.1 The Operational Problem with LLMs for Continuous Ingestion
Modern commercial and open-source Large Language Models (e.g. LLaMA-3-8B, Mistral-7B, GPT-4o) present fundamental operational disqualifiers for local-first, background continuous news monitoring:
1. **Memory Saturation:** An 8-billion parameter model quantized to 4 bits (`Q4_K_M`) occupies $\approx 4.8\text{ GB}$ of RAM. On an Apple Silicon M1 with 8GB shared unified memory, running this model triggers severe macOS memory swapping, causing UI frame drops and system lag.
2. **Energy & Battery Drain:** Continuous streaming inference of 2,000 articles per day across neural cores would drain laptop battery reserves in $<3\text{ hours}$.
3. **Hallucination Risk:** Generative language models fabricate facts, distort casualty numbers, and introduce temporal confusion when summarizing breaking news.
4. **Latency:** Even with optimized llama.cpp on Metal, token generation speeds of 30 tokens/sec mean processing 100 articles takes 5–8 minutes, lagging behind breaking news feeds.

### 1.2 The Deterministic & Statistical NLP Alternative
GLOBALNEWS deploys a pipeline of specialized, deterministic, and statistical NLP models that process text in **milliseconds per article** with zero hallucination and a cumulative memory footprint under **$120\text{ MB}$**:

```
+-----------------------------------------------------------------------------------+
|                            LIGHTWEIGHT NLP PIPELINE                               |
+-----------------------------------------------------------------------------------+
                                          |
  [Raw Ingested Article]                  |   Latency: < 0.2 ms
  -----------------------> [ 1. Language ID: fastText lid.176.ftz ]
                                          |   Memory: 3.5 MB
                                          v
                           [ 2. Text Normalization: Trafilatura ]
                                          |   Latency: 5-15 ms
                                          |   Extracts clean title, body, date
                                          v
                           [ 3. Named Entity Recognition: spaCy ]
                                          |   Latency: 8-12 ms
                                          |   Extracts GPE, LOC, ORG, EVENT
                                          v
                           [ 4. Offline Toponym Resolution ]
                                          |   Latency: < 1.0 ms
                                          |   SQLite GeoNames + Population Prior
                                          v
                           [ 5. IPTC Topic Classification ]
                                          |   Latency: < 2.0 ms
                                          |   TF-IDF + Linear SGD Classifier
                                          v
                           [ 6. Multi-Factor Impact Differentiation ]
                                          |   Latency: < 0.5 ms
                                          |   Linguistic Polarity vs Severity Matrix
                                          v
                           [ 7. Extractive Summarization: TextRank ]
                                          |   Latency: 3-5 ms
                                          |   Graph-based sentence centrality
                                          v
  [Structured Event Vector: Coordinates, Entities, Impact, Summary, Citations]
```

---

## 2. Pipeline Stage Breakdown

### 2.1 Language Identification (FastText)
* **Model:** `lid.176.ftz` (Compressed 16-bit quantized FastText model).
* **Disk Footprint:** $917\text{ KB}$.
* **RAM Usage:** $\approx 3.5\text{ MB}$.
* **Throughput:** Over $80,000\text{ sentences/sec}$ on Apple M1.
* **Accuracy:** $98.4\%$ macro-accuracy across 176 languages.
* **Routing Strategy:** Articles classified as English route to `en_core_web_sm`; articles classified in other major languages (Spanish, French, German, Chinese, Arabic, Russian) route to `xx_ent_wiki_sm` and native language stopwords.

---

### 2.2 Named Entity Recognition (spaCy Small Models)
* **English Model:** `en_core_web_sm` (v3.8.x). Size: $12\text{ MB}$.
* **Multilingual Model:** `xx_ent_wiki_sm` (v3.8.x). Size: $13\text{ MB}$.
* **Execution Time:** $10\text{ ms}$ per 500-word article on CPU.
* **Target Entities:**
  * `GPE`: Countries, cities, administrative regions (primary geoparsing input).
  * `LOC`: Non-GPE geographical features (straits, mountain ranges, oceans).
  * `ORG`: National governments, multinational alliances (NATO, UN), central banks.
  * `EVENT`: Named historical or current conflicts, storms, summits.

---

### 2.3 Toponym Resolution & Offline Geoparsing

The greatest failure mode of automated geospatial news is **toponym ambiguity**:
* *Paris:* Paris, France ($2.1\text{M}$ pop) vs. Paris, Texas ($25\text{k}$ pop).
* *Georgia:* The sovereign Caucasus nation vs. the US State.
* *Cambridge:* UK ($125\text{k}$ pop) vs. Massachusetts, USA ($118\text{k}$ pop).

#### Two-Stage Geoparsing Algorithm (Zero External API Calls)
```
[ Extracted GPE / LOC Text: e.g. "Paris" ]
                     |
                     v
+-------------------------------------------------------------+
| Stage 1: Contextual Administrative Scoping                  |
| Scan document for co-occurring Country or State entities    |
| Example: If "France" or "Macron" appears in same article,   |
| bias country filter to 'FR'.                                |
+-------------------------------------------------------------+
                     |
                     v
+-------------------------------------------------------------+
| Stage 2: SQLite GeoNames Indexed Query                      |
| SELECT geonameid, name, latitude, longitude, country_code,  |
|        population, feature_class                            |
| FROM geonames                                               |
| WHERE name = ? COLLATE NOCASE                               |
| ORDER BY                                                    |
|   (CASE WHEN country_code = ? THEN 100 ELSE 0 END) +        |
|   LOG(population + 1) * 2.5 DESC                            |
| LIMIT 1;                                                    |
+-------------------------------------------------------------+
                     |
                     v
[ Resolved: Paris, Île-de-France (48.8566° N, 2.3522° E) ]
```

* **Data Asset:** Bundled SQLite table containing GeoNames `cities15000` ($25,000\text{ global cities}$ with population $> 15,000$, all countries, and primary administrative divisions).
* **Storage Footprint:** $14\text{ MB}$ SQLite file with primary B-Tree index on `name` and R-Tree index on spatial bounds.
* **Query Latency:** $< 0.4\text{ ms}$ per resolution.

---

## 3. Topic & Event Taxonomy Classification

GLOBALNEWS implements the **IPTC Media Topics standard** (International Press Telecommunications Council), an open, 1,200-category hierarchical taxonomy recognized globally across news publishers.

### Top-Level IPTC Domain Mapping
1. `01000000`: Arts, culture, entertainment
2. `02000000`: Crime, law, and justice
3. `03000000`: Disaster, accident, and emergency
4. `04000000`: Economy, business, and finance
5. `05000000`: Education
6. `06000000`: Environmental issues
7. `07000000`: Health and medicine
8. `08000000`: Human interest
9. `09000000`: Labour and employment
10. `10000000`: Armed conflicts and politics
11. `11000000`: Science and technology
12. `14000000`: Social issues
13. `15000000`: Sports
14. `16000000`: Weather and climate

### Classification Engine Architecture
* **Vectorization:** TF-IDF with sublinear term-frequency scaling ($\text{TF} = 1 + \log(\text{tf})$), n-gram range $(1, 2)$, max features $15,000$.
* **Classifier:** Linear Support Vector Machine (`SGDClassifier(loss='modified_huber')`) calibrated to output well-behaved posterior probabilities $P(\text{Topic}_j \mid \vec{d})$.
* **Model Size:** $< 8\text{ MB}$ serialized model file.
* **Inference Latency:** $< 1.5\text{ ms}$ per article. Zero neural network runtime required.

---

## 4. Multi-Factor Sentiment, Polarity & Impact Differentiation

Traditional sentiment analysis (e.g. VADER, TextBlob) is fundamentally unsuited for news intelligence. If an article reports:
> *"Rescue workers successfully rescued 14 survivors from the collapsed earthquake rubble."*

A naive sentiment tool rates this as strongly **Positive** ($+0.85$) due to "successfully", "rescued", and "survivors". However, in reality:
* The event is a catastrophic disaster (Negative Impact, High Severity).
* The specific development is a positive humanitarian breakthrough (Positive Event Direction).

GLOBALNEWS strictly decomposes news developments into **six independent orthogonal dimensions**:

```
+-------------------------------------------------------------------------------------------+
|                          SIX-DIMENSIONAL EVENT EVALUATION                                 |
+--------------------------+-----------------------+----------------------------------------+
| Dimension                | Scale / Range         | Operational Definition                 |
+--------------------------+-----------------------+----------------------------------------+
| 1. Linguistic Sentiment  | $[-1.0 \dots +1.0]$   | Emotional valence of words in headline |
| 2. Event Direction       | $[-1.0 \dots +1.0]$   | Progress (+) vs Degeneration (-)       |
| 3. Real-World Impact     | $[0.0 \dots 10.0]$    | Casualties, economic & physical shock  |
| 4. Event Severity        | $[0.0 \dots 10.0]$    | Irreversibility, scale, and urgency    |
| 5. Evidence Confidence   | $[0.0 \dots 1.0]$     | Corroboration score across sources     |
| 6. Media Coverage Volume | Integer count         | Raw count of ingested articles         |
+--------------------------+-----------------------+----------------------------------------+
```

### 4.1 Real-World Impact Scoring Formulation
$$\text{ImpactScore} = \min\left(10.0, \, w_{\text{cat}} \cdot \text{BaseWeight} + \sum_{k} \beta_k \cdot \text{IndicatorTrigger}_k\right)$$

Where:
* $\text{IndicatorTrigger}_k$ represents extracted quantified variables:
  * Casualty numbers: $\log_{10}(\text{deaths} + 1) \times 2.0$
  * Magnitude for earthquakes: $\max(0, (\text{Mag} - 4.5) \times 2.5)$
  * Financial impact: $\log_{10}(\text{CurrencyDisruption} + 1)$
  * Population displaced: $\log_{10}(\text{Displaced} + 1) \times 1.2$

---

## 5. Extractive Summarization (TextRank / LexRank)

To guarantee zero hallucinations and maintain 100% auditable source attribution, GLOBALNEWS rejects generative abstractive summarization. It uses an optimized graph-based **TextRank** extractive summarizer.

### 5.1 Algorithmic Formulation
1. Segment cleaned article text into sentences $\{S_1, S_2, \dots, S_m\}$.
2. Compute pairwise sentence similarity matrix $\mathbf{W} \in \mathbb{R}^{m \times m}$ based on BM25 or token overlap:
   $$W_{ij} = \frac{|\{w \in S_i\} \cap \{w \in S_j\}|}{\log(|S_i|) + \log(|S_j|)}$$
3. Construct directed sentence graph and compute PageRank score $PR(S_i)$ iteratively:
   $$PR(S_i) = (1 - d) + d \sum_{j \in In(S_i)} \frac{W_{ji}}{\sum_{k} W_{jk}} PR(S_j)$$
   where damping factor $d = 0.85$.
4. Select the top 2 or 3 highest-ranking sentences in their original chronological reading order.
5. Store exact character offsets linking sentences back to original publisher text.

* **Performance:** Takes $< 4\text{ ms}$ on standard articles.
* **Output:** A concise, factual, 3-sentence summary of the event that directly mirrors verified source text.

