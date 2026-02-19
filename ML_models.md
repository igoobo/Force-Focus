# **Context-Aware Adaptive Anomaly Detection System Specification (Simplified)**

## **1. System Overview**

*   **Objective**: To detect "contextual drift" in real-time by learning user work patterns (Context + Activity) and intervene flexibly.
*   **Core Strategy**:
    1.  **Global Knowledge**: Classify 'Universal Work/Rest' using a lightweight **Keyword Dictionary (HashMap)**.
    2.  **Local Override**: Prioritize user feedback to correct model judgments immediately.
    3.  **Safety Nets**: Do not intervene when uncertain; forget old data over time.

---

## **2. Data Pipeline (Client-Side)**

### **Phase 1: Preprocessing**

Raw data from `local.db` is processed into a model input vector ($V_t$).

#### **1. Active Window Focus**
*   **Input**: `active_window` (App Name, Title).
*   **Logic**: Only consider the currently active window. Background windows are ignored to reduce complexity and noise.
*   **Output**: `Visual_Weight` is effectively 1.0 for the active window and 0.0 for others.

#### **2. Simple Tokenization**
*   **Logic**:
    1.  Combine `App Name` and `Window Title`.
    2.  **Split** by non-alphanumeric characters (space, dot, dash, etc.).
    3.  Convert to **Lowercase**.
    4.  **Filter**: Remove single characters and common stop words (if any).
*   **Output**: List of Tokens (e.g., `["visual", "studio", "code", "project", "main", "rs"]`).

---

## **3. Feature Engineering (3 Axes $\to$ 6 Dimensions)**

The inputs are conceptually divided into 3 axes, which map to **6 actual dimensions** in the input vector for the SVM.

$$V_{input} = [\underbrace{X_{context}}_{Axis 1}, \underbrace{X_{log\_input}, X_{silence}, X_{burstiness}, X_{mouse}}_{Axis 2}, \underbrace{X_{interaction}}_{Axis 3}]$$

### **Axis 1: Context Score ($X_{context}$) [1 Dim]**
*   **Definition**: A value between -1.0 (Rest) and 1.0 (Work) indicating the productivity of the current screen.
*   **Calculation**: Average of token scores matched in the dictionary.
    $$X_{context} = \frac{\sum S(t_i)}{\text{Count}(t_i)}$$
*   **Scoring Logic ($S(t_i)$)**:
    1.  **Local Cache (Override)**: If token matches user feedback $\to$ Return **1.0 (Work)**.
    2.  **Global Dictionary**: Mapped values (e.g., `vscode`=1.0, `youtube`=-1.0).
    3.  **Fuzzy Match**: Use **Levenshtein Distance** to allow slight variations (e.g., version numbers).
    4.  **Unknown**: If not found, return 0.0 (Neutral).

### **Axis 2: Activity Metrics ($X_{activity}$) [4 Dims]**
Quantifies the "Quality of Action".
1.  **$X_{log\_input}$ (Input Density)**:
    *   $\log(\text{input\_events difference} + 1)$
2.  **$X_{silence}$ (Silence Duration)**:
    *   Seconds duration where input is 0.
3.  **$X_{burstiness}$ (Input Burstiness)**:
    *   Standard Deviation of input counts over the last 1 minute.
4.  **$X_{mouse}$ (Mouse Active)**:
    *   Boolean flag (1.0 or 0.0) indicating mouse movement in the last 5s.

### **Axis 3: Interaction Gate ($X_{interaction}$) [1 Dim]**
Conditional amplification of context score when input is low.
*   **Formula**:
    $$Gate = \text{Sigmoid}\left(\frac{1}{\text{Delta\_Input} + 0.1}\right)$$
    $$X_{interaction} = Gate \times X_{context}$$
*   **Effect**:
    *   High Input $\to$ Gate $\approx 0 \to X_{interaction} \approx 0$ (Context less important).
    *   Zero Input $\to$ Gate $\approx 1 \to X_{interaction} \approx X_{context}$ (Context determines Anomaly).

---

## **4. Modeling & Learning (Backend)**

### **Core Model: Weighted One-Class SVM**
*   **Algorithm**: One-Class SVM with **RBF Kernel**.
*   **Hyperparameters**:
    *   $\nu$ (Nu): 0.05 ~ 0.1.
    *   $\gamma$ (Gamma): `scale`.
*   **Input Vector**: The 6-dimensional vector defined above.
    *   StandardScaler is applied.

### **Training Strategy (Simplified)**
*   **Sample Weights ($W_i$)**:
    *   Based solely on **Context Score**.
    *   $W_{total} = \text{Sigmoid}(X_{context} \times 5) \times 2.0$
    *   (Highly productive contexts get higher weight in defining "Normal").
*   **Data Window**: Train on all available recent data (e.g., last 1000 sessions or 30 days).

---

## **5. Inference & Control (Client-Side)**

### **Real-time Inference**
*   **Output**: Decision Function Value ($Score$).
    *   $Score > 0$: **Inlier** (Normal).
    *   $-0.5 < Score \le 0$: **Weak Outlier**.
    *   $Score \le -0.5$: **Strong Outlier**.

### **State Machine (FSM) - Gauge Based**
*   **Concept**: Accumulate "Drift Seconds" instead of a hard timer.
1.  **FOCUS**: Normal state. Gauge decrements.
2.  **DRIFT**: Gauge accumulates.
    *   **Strong Outlier**: +1.0 sec/sec.
    *   **Weak Outlier**: +0.5 sec/sec.
    *   **Active Thinking (Safety)**: If Input=0 but Mouse=Moving $\to$ +0.25 sec/sec (Slow accumulation).
    *   **Recovery**: If Inlier $\to$ -2.0 sec/sec (Fast recovery).
3.  **Thresholds**:
    *   **Notify**: Gauge $\ge$ 30s.
    *   **Block (Overlay)**: Gauge $\ge$ 60s.

---

## **6. Feedback Loop**

### **Short-term: Local Cache**
*   **Trigger**: User clicks "I'm Working".
*   **Action**: Store current active tokens in `LocalCache` with TTL (24h).
*   **Effect**: Future $X_{context}$ for these tokens becomes 1.0.

### **Long-term: Global Dictionary Update**
*   (Manual/Server-side) Occasional updates to the static HashMap based on common user patterns.
