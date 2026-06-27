import * as vscode from 'vscode';

// ---------------------------------------------------------------------------
// Data contract
// ---------------------------------------------------------------------------

export interface AresEvidence {
    source: string;
    detail: string;
    line?: number;
    column?: number;
}

export interface AresDecision {
    date: string;
    summary: string;
    author: string;
}

export interface AresDashboard {
    repository: any;
    graph: any;
    integrity: any;
    coverage: any;
    intelligence: any;
    performance: any;
    health: any;
    activity: any[];
    version: any;
}


export interface AresResponse {
    answer: string;
    confidence: number;
    evidence: AresEvidence[];
    related_decisions: AresDecision[];
    query_type?: string;
    file_path?: string;
    dashboard?: AresDashboard;
    recent_queries?: any[];
    [key: string]: any;
}

export interface AresError {
    message: string;
    detail?: string;
}

// ---------------------------------------------------------------------------
// Webview message types (extension → webview)
// ---------------------------------------------------------------------------

type PanelMessage =
    | { type: 'loading' }
    | { type: 'update'; data: AresResponse }
    | { type: 'error'; error: AresError };

// ---------------------------------------------------------------------------
// AresQueryPanel — singleton webview panel
// ---------------------------------------------------------------------------

export class AresQueryPanel {
    public static readonly viewType = 'aresMemoryPanel';
    private static instance: AresQueryPanel | undefined;

    private readonly panel: vscode.WebviewPanel;
    private readonly disposables: vscode.Disposable[] = [];

    // -----------------------------------------------------------------------
    // Construction & lifecycle
    // -----------------------------------------------------------------------

    private constructor(panel: vscode.WebviewPanel) {
        this.panel = panel;
        this.panel.onDidDispose(() => this.dispose(), null, this.disposables);
    }

    private dispose(): void {
        AresQueryPanel.instance = undefined;
        this.panel.dispose();
        while (this.disposables.length) {
            const d = this.disposables.pop();
            if (d) {
                d.dispose();
            }
        }
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /** Show loading spinner immediately, then post data or error later. */
    public static showLoading(context: vscode.ExtensionContext): AresQueryPanel {
        const inst = AresQueryPanel.ensurePanel(context);
        inst.postMessage({ type: 'loading' });
        return inst;
    }

    /** Show a successful response. */
    public static show(context: vscode.ExtensionContext, data: AresResponse): AresQueryPanel {
        const inst = AresQueryPanel.ensurePanel(context);
        inst.postMessage({ type: 'update', data });
        return inst;
    }

    /** Show a user-friendly error inside the panel. */
    public static showError(context: vscode.ExtensionContext, error: AresError): AresQueryPanel {
        const inst = AresQueryPanel.ensurePanel(context);
        inst.postMessage({ type: 'error', error });
        return inst;
    }

    /** Access the underlying webview (for onDidReceiveMessage). */
    public get webview(): vscode.Webview {
        return this.panel.webview;
    }

    // -----------------------------------------------------------------------
    // Internals
    // -----------------------------------------------------------------------

    private static ensurePanel(context: vscode.ExtensionContext): AresQueryPanel {
        if (AresQueryPanel.instance) {
            AresQueryPanel.instance.panel.reveal(vscode.ViewColumn.Beside);
            return AresQueryPanel.instance;
        }

        const panel = vscode.window.createWebviewPanel(
            AresQueryPanel.viewType,
            'ARES Memory',
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
            }
        );

        panel.webview.html = AresQueryPanel.getHtml();
        AresQueryPanel.instance = new AresQueryPanel(panel);
        return AresQueryPanel.instance;
    }

    private postMessage(msg: PanelMessage): void {
        this.panel.webview.postMessage(msg);
    }

    // -----------------------------------------------------------------------
    // HTML / CSS / JS — everything embedded, zero external deps
    // -----------------------------------------------------------------------

    private static getHtml(): string {
        return /* html */ `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ARES Memory</title>
<style>
/* ===== Reset & Base ===== */
*{margin:0;padding:0;box-sizing:border-box}
body{
    background:var(--vscode-editor-background);
    color:var(--vscode-editor-foreground);
    font-family:var(--vscode-font-family,-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif);
    font-size:var(--vscode-font-size,13px);
    line-height:1.6;
    overflow-x:hidden;
}

/* ===== Layout shell ===== */
.container{max-width:820px;margin:0 auto;padding:24px 20px}

/* ===== Animations ===== */
@keyframes fadeIn{from{opacity:0}to{opacity:1}}
@keyframes slideUp{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}
@keyframes pulse{0%,100%{opacity:.4}50%{opacity:1}}
.animate-fade{animation:fadeIn .35s ease both}
.animate-slide{animation:slideUp .4s ease both}

/* ===== Header ===== */
.header{
    display:flex;align-items:center;gap:12px;
    padding-bottom:16px;
    border-bottom:1px solid var(--vscode-panel-border);
    margin-bottom:20px;
    animation:fadeIn .3s ease;
}
.header-icon{
    width:36px;height:36px;border-radius:10px;
    background:var(--vscode-button-background);
    color:var(--vscode-button-foreground);
    display:flex;align-items:center;justify-content:center;
    font-size:18px;flex-shrink:0;
}
.header-text h1{font-size:18px;font-weight:600;letter-spacing:-.3px}
.query-badge{
    display:inline-block;margin-top:4px;
    font-size:10px;font-weight:700;
    text-transform:uppercase;letter-spacing:1px;
    padding:2px 8px;border-radius:4px;
    background:var(--vscode-button-background);
    color:var(--vscode-button-foreground);
}

/* ===== File path ===== */
.file-path{
    background:color-mix(in srgb,var(--vscode-editor-foreground) 6%,transparent);
    border:1px solid var(--vscode-panel-border);
    border-radius:6px;padding:8px 12px;
    font-family:var(--vscode-editor-font-family,'Consolas','Courier New',monospace);
    font-size:12px;color:var(--vscode-descriptionForeground);
    margin-bottom:20px;word-break:break-all;
    animation:slideUp .35s ease both;
}
.file-path::before{content:'📄 '}

/* ===== Sections ===== */
.section{
    margin-bottom:20px;
    border:1px solid var(--vscode-panel-border);
    border-radius:8px;overflow:hidden;
}
.section-header{
    font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;
    color:var(--vscode-descriptionForeground);
    padding:10px 16px;
    background:color-mix(in srgb,var(--vscode-editor-foreground) 4%,transparent);
    border-bottom:1px solid var(--vscode-panel-border);
}
.section-body{padding:16px}

/* ===== Answer ===== */
.answer-text{line-height:1.75;white-space:pre-wrap;word-wrap:break-word}

/* ===== Confidence ===== */
.confidence-row{display:flex;align-items:center;gap:14px}
.confidence-label{
    font-size:12px;font-weight:600;min-width:60px;
}
.confidence-label.high{color:#2ea043}
.confidence-label.medium{color:#d29922}
.confidence-label.low{color:#da3633}
.confidence-bar-track{
    flex:1;height:8px;border-radius:4px;
    background:color-mix(in srgb,var(--vscode-editor-foreground) 10%,transparent);
    overflow:hidden;
}
.confidence-bar-fill{height:100%;border-radius:4px;transition:width .6s ease}
.confidence-bar-fill.high{background:#2ea043}
.confidence-bar-fill.medium{background:#d29922}
.confidence-bar-fill.low{background:#da3633}
.confidence-pct{
    font-size:14px;font-weight:700;min-width:48px;text-align:right;
}
.confidence-pct.high{color:#2ea043}
.confidence-pct.medium{color:#d29922}
.confidence-pct.low{color:#da3633}

/* ===== Evidence ===== */
.evidence-item{
    padding:10px 0;
    border-bottom:1px solid color-mix(in srgb,var(--vscode-panel-border) 50%,transparent);
}
.evidence-item:last-child{border-bottom:none}
.evidence-source{
    font-family:var(--vscode-editor-font-family,'Consolas','Courier New',monospace);
    font-size:12px;color:var(--vscode-textLink-foreground);
    cursor:pointer;text-decoration:none;display:inline-block;margin-bottom:4px;
}
.evidence-source:hover{text-decoration:underline}
.evidence-line-hint{
    font-size:11px;color:var(--vscode-descriptionForeground);
    margin-left:6px;
}
.evidence-detail{font-size:12px;color:var(--vscode-descriptionForeground);line-height:1.5}

/* ===== Decision cards ===== */
.decision-card{
    border:1px solid var(--vscode-panel-border);
    border-radius:8px;padding:14px 16px;
    margin-bottom:10px;
    background:color-mix(in srgb,var(--vscode-editor-foreground) 3%,transparent);
    transition:border-color .15s ease;
}
.decision-card:last-child{margin-bottom:0}
.decision-card:hover{border-color:var(--vscode-button-background)}
.decision-meta{display:flex;align-items:center;gap:10px;margin-bottom:6px}
.decision-date{
    font-size:11px;font-weight:500;
    color:var(--vscode-descriptionForeground);
    background:color-mix(in srgb,var(--vscode-editor-foreground) 8%,transparent);
    padding:2px 8px;border-radius:4px;
}
.decision-author{font-size:11px;color:var(--vscode-descriptionForeground)}
.decision-author::before{content:'👤 '}
.decision-summary{font-size:13px;line-height:1.5}

/* ===== Empty state ===== */
.empty-state{
    display:flex;flex-direction:column;align-items:center;justify-content:center;
    text-align:center;padding:60px 24px;
    color:var(--vscode-descriptionForeground);
    animation:fadeIn .4s ease;
}
.empty-icon{font-size:56px;margin-bottom:20px;opacity:.6}
.empty-title{font-size:16px;font-weight:600;color:var(--vscode-editor-foreground);margin-bottom:8px}
.empty-hint{font-size:13px;line-height:1.6;max-width:380px;margin-bottom:16px}
.empty-command{
    display:inline-block;
    font-family:var(--vscode-editor-font-family,'Consolas','Courier New',monospace);
    font-size:12px;
    background:var(--vscode-button-background);color:var(--vscode-button-foreground);
    padding:6px 14px;border-radius:6px;margin-bottom:20px;
}
.empty-examples{list-style:none;text-align:left;font-size:12px;line-height:2}
.empty-examples li::before{content:'• ';color:var(--vscode-button-background)}

/* ===== Loading ===== */
.loading-state{
    display:flex;flex-direction:column;align-items:center;justify-content:center;
    padding:100px 24px;gap:16px;animation:fadeIn .2s ease;
}
.loading-spinner{
    width:32px;height:32px;border-radius:50%;
    border:3px solid color-mix(in srgb,var(--vscode-editor-foreground) 15%,transparent);
    border-top-color:var(--vscode-button-background);
    animation:spin .8s linear infinite;
}
@keyframes spin{to{transform:rotate(360deg)}}
.loading-text{font-size:13px;color:var(--vscode-descriptionForeground);animation:pulse 1.5s ease infinite}

/* ===== Error state ===== */
.error-state{
    display:flex;flex-direction:column;align-items:center;justify-content:center;
    text-align:center;padding:60px 24px;animation:fadeIn .35s ease;
}
.error-icon{font-size:48px;margin-bottom:16px}
.error-title{font-size:16px;font-weight:600;color:var(--vscode-editor-foreground);margin-bottom:8px}
.error-message{font-size:13px;color:var(--vscode-descriptionForeground);max-width:400px;margin-bottom:20px;line-height:1.6}
.error-reasons{
    list-style:none;text-align:left;font-size:12px;line-height:2.2;
    background:color-mix(in srgb,var(--vscode-editor-foreground) 4%,transparent);
    border:1px solid var(--vscode-panel-border);
    border-radius:8px;padding:12px 20px;
}
.error-reasons li::before{content:'• ';color:#da3633}

/* ===== Utility ===== */
.hidden{display:none!important}
</style>
</head>
<body>

<!-- Loading -->
<div id="loadingState" class="loading-state">
    <div class="loading-spinner"></div>
    <div class="loading-text">Querying ARES Memory…</div>
</div>

<!-- Error -->
<div id="errorState" class="error-state hidden">
    <div class="error-icon">⚠️</div>
    <div id="errorTitle" class="error-title"></div>
    <div id="errorMessage" class="error-message"></div>
    <ul class="error-reasons">
        <li>Repository not ingested</li>
        <li>MCP server unavailable</li>
        <li>SQLite database missing</li>
        <li>Query failed</li>
    </ul>
</div>

<!-- Main content -->
<div id="content" class="container hidden">
    <!-- Header -->
    <div class="header">
        <div class="header-icon">⚡</div>
        <div class="header-text">
            <h1>ARES Memory</h1>
            <div id="queryBadge" class="query-badge hidden"></div>
        </div>
    </div>

    <!-- File Path -->
    <div id="filePath" class="file-path hidden"></div>

    <!-- Result content (shown when data exists) -->
    <div id="resultContent">
        <!-- Answer -->
        <div class="section animate-slide" style="animation-delay:.05s">
            <div class="section-header">Answer</div>
            <div class="section-body">
                <div id="answer" class="answer-text"></div>
            </div>
        </div>

        <!-- Confidence -->
        <div class="section animate-slide" style="animation-delay:.1s">
            <div class="section-header">Confidence</div>
            <div class="section-body">
                <div class="confidence-row">
                    <div id="confidenceLabel" class="confidence-label"></div>
                    <div class="confidence-bar-track">
                        <div id="confidenceBar" class="confidence-bar-fill"></div>
                    </div>
                    <div id="confidencePct" class="confidence-pct"></div>
                </div>
            </div>
        </div>

        <!-- Evidence -->
        <div id="evidenceSection" class="section animate-slide hidden" style="animation-delay:.15s">
            <div class="section-header">Evidence</div>
            <div id="evidenceList" class="section-body"></div>
        </div>

        <!-- Decisions -->
        <div id="decisionsSection" class="section animate-slide hidden" style="animation-delay:.2s">
            <div class="section-header">Related Decisions</div>
            <div id="decisionsList" class="section-body"></div>
        </div>

        <!-- Drift Analysis -->
        <div id="driftSection" class="section animate-slide hidden" style="animation-delay:.25s">
            <div class="section-header">Drift Analysis</div>
            <div id="driftList" class="section-body"></div>
        </div>

        <!-- Simulation Result -->
        <div id="simulationSection" class="section animate-slide hidden" style="animation-delay:.3s">
            <div class="section-header">Simulation Result</div>
            <div id="simulationList" class="section-body"></div>
        </div>

        <!-- Traceability -->
        <div id="traceabilitySection" class="section animate-slide hidden" style="animation-delay:.35s">
            <div class="section-header">Traceability</div>
            <div id="traceabilityList" class="section-body"></div>
        </div>

        <!-- Dashboard -->
        <div id="dashboardSection" class="section animate-slide hidden" style="animation-delay:.40s">
            <div class="section-header">Project Health Dashboard</div>
            <div id="dashboardList" class="section-body"></div>
        </div>
    </div>

    <!-- Empty state -->
    <div id="emptyState" class="empty-state hidden">
        <div class="empty-icon">🧠</div>
        <div class="empty-title">ARES has no memory for this file yet.</div>
        <div class="empty-hint">Run the ingest command to build repository memory.</div>
        <div class="empty-command">ARES: Ingest Repository</div>
        <ul class="empty-examples">
            <li>Why does this file exist?</li>
            <li>Who owns this component?</li>
            <li>What breaks if I change it?</li>
            <li>Which decision created it?</li>
        </ul>
    </div>
</div>

<script>
(function () {
    const vscode = acquireVsCodeApi();

    // --- DOM refs ---
    const dom = {
        loading:          document.getElementById('loadingState'),
        error:            document.getElementById('errorState'),
        errorTitle:       document.getElementById('errorTitle'),
        errorMessage:     document.getElementById('errorMessage'),
        content:          document.getElementById('content'),
        queryBadge:       document.getElementById('queryBadge'),
        filePath:         document.getElementById('filePath'),
        resultContent:    document.getElementById('resultContent'),
        answer:           document.getElementById('answer'),
        confidenceLabel:  document.getElementById('confidenceLabel'),
        confidenceBar:    document.getElementById('confidenceBar'),
        confidencePct:    document.getElementById('confidencePct'),
        evidenceSection:  document.getElementById('evidenceSection'),
        evidenceList:     document.getElementById('evidenceList'),
        decisionsSection: document.getElementById('decisionsSection'),
        decisionsList:    document.getElementById('decisionsList'),
        driftSection:     document.getElementById('driftSection'),
        driftList:        document.getElementById('driftList'),
        simulationSection: document.getElementById('simulationSection'),
        simulationList:   document.getElementById('simulationList'),
        traceabilitySection: document.getElementById('traceabilitySection'),
        traceabilityList: document.getElementById('traceabilityList'),
        dashboardSection: document.getElementById('dashboardSection'),
        dashboardList:    document.getElementById('dashboardList'),
        emptyState:       document.getElementById('emptyState'),
    };

    // --- Helpers ---
    function confidenceLevel(v) {
        if (v > 0.8)  return 'high';
        if (v > 0.5)  return 'medium';
        return 'low';
    }
    function confidenceText(v) {
        if (v > 0.8)  return 'High';
        if (v > 0.5)  return 'Medium';
        return 'Low';
    }
    function isEmpty(d) {
        return (!d.answer || d.answer.trim() === '') &&
               (!d.evidence || d.evidence.length === 0) &&
               (!d.dashboard);
    }
    function hideAll() {
        dom.loading.classList.add('hidden');
        dom.error.classList.add('hidden');
        dom.content.classList.add('hidden');
    }

    // --- Renderers (modular — add new renderers for future query types) ---

    function renderHeader(data) {
        if (data.query_type) {
            dom.queryBadge.textContent = data.query_type.replace(/_/g, ' ');
            dom.queryBadge.classList.remove('hidden');
        } else {
            dom.queryBadge.classList.add('hidden');
        }

        if (data.file_path) {
            dom.filePath.textContent = data.file_path;
            dom.filePath.classList.remove('hidden');
        } else {
            dom.filePath.classList.add('hidden');
        }
    }

    function renderAnswer(data) {
        dom.answer.textContent = data.answer;
    }

    function renderConfidence(data) {
        var pct = Math.round((data.confidence || 0) * 100);
        var level = confidenceLevel(data.confidence || 0);

        dom.confidenceLabel.textContent = confidenceText(data.confidence || 0);
        dom.confidenceLabel.className = 'confidence-label ' + level;

        dom.confidenceBar.className = 'confidence-bar-fill ' + level;
        dom.confidenceBar.style.width = pct + '%';

        dom.confidencePct.textContent = pct + '%';
        dom.confidencePct.className = 'confidence-pct ' + level;
    }

    function renderEvidence(data) {
        dom.evidenceList.innerHTML = '';
        if (!data.evidence || data.evidence.length === 0) {
            dom.evidenceSection.classList.add('hidden');
            return;
        }
        dom.evidenceSection.classList.remove('hidden');

        data.evidence.forEach(function (ev) {
            var item = document.createElement('div');
            item.className = 'evidence-item';

            var src = document.createElement('a');
            src.className = 'evidence-source';
            src.textContent = ev.source;
            src.title = 'Open ' + ev.source;
            src.addEventListener('click', function () {
                vscode.postMessage({
                    command: 'openFile',
                    path: ev.source,
                    line: ev.line,
                    column: ev.column,
                });
            });
            item.appendChild(src);

            if (typeof ev.line === 'number') {
                var hint = document.createElement('span');
                hint.className = 'evidence-line-hint';
                hint.textContent = ':' + ev.line + (typeof ev.column === 'number' ? ':' + ev.column : '');
                item.appendChild(hint);
            }

            var detail = document.createElement('div');
            detail.className = 'evidence-detail';
            detail.textContent = ev.detail;
            item.appendChild(detail);

            dom.evidenceList.appendChild(item);
        });
    }

    function renderDecisions(data) {
        dom.decisionsList.innerHTML = '';
        if (!data.related_decisions || data.related_decisions.length === 0) {
            dom.decisionsSection.classList.add('hidden');
            return;
        }
        dom.decisionsSection.classList.remove('hidden');

        data.related_decisions.forEach(function (dec) {
            var card = document.createElement('div');
            card.className = 'decision-card';

            var meta = document.createElement('div');
            meta.className = 'decision-meta';

            var date = document.createElement('span');
            date.className = 'decision-date';
            date.textContent = dec.date;

            var author = document.createElement('span');
            author.className = 'decision-author';
            author.textContent = dec.author;

            meta.appendChild(date);
            meta.appendChild(author);

            var summary = document.createElement('div');
            summary.className = 'decision-summary';
            summary.textContent = dec.summary;

            card.appendChild(meta);
            card.appendChild(summary);
            dom.decisionsList.appendChild(card);
        });
    }

    function renderDrift(data) {
        dom.driftList.innerHTML = '';
        if (data.has_drift === undefined && !data.summary) {
            dom.driftSection.classList.add('hidden');
            return;
        }
        dom.driftSection.classList.remove('hidden');

        var card = document.createElement('div');
        card.className = 'decision-card';
        
        var title = document.createElement('div');
        title.className = 'decision-meta';
        
        var status = document.createElement('span');
        status.className = 'decision-author';
        status.textContent = data.has_drift ? "Drift Detected" : "No Drift";
        status.style.color = data.has_drift ? "#f48771" : "#89d185";
        
        var score = document.createElement('span');
        score.className = 'decision-date';
        score.textContent = "Score: " + (data.drift_score !== undefined ? data.drift_score.toFixed(2) : "0.00");
        
        title.appendChild(status);
        title.appendChild(score);
        
        var summary = document.createElement('div');
        summary.className = 'decision-summary';
        summary.textContent = data.summary || "No summary available.";
        
        card.appendChild(title);
        card.appendChild(summary);
        
        if (data.decision_orphans && data.decision_orphans.length > 0) {
            var orphansTitle = document.createElement('div');
            orphansTitle.className = 'section-header';
            orphansTitle.style.marginTop = '10px';
            orphansTitle.style.fontSize = '12px';
            orphansTitle.textContent = "Orphaned Decisions:";
            card.appendChild(orphansTitle);
            
            data.decision_orphans.forEach(function(orphan) {
                var orphanItem = document.createElement('div');
                orphanItem.className = 'evidence-source';
                orphanItem.textContent = orphan;
                card.appendChild(orphanItem);
            });
        }
        
        dom.driftList.appendChild(card);
    }

    function renderSimulation(data) {
        dom.simulationList.innerHTML = '';
        if (data.action === undefined || data.impact_radius === undefined) {
            dom.simulationSection.classList.add('hidden');
            return;
        }
        dom.simulationSection.classList.remove('hidden');

        var card = document.createElement('div');
        card.className = 'decision-card';
        
        var title = document.createElement('div');
        title.className = 'decision-meta';
        
        var status = document.createElement('span');
        status.className = 'decision-author';
        status.textContent = data.reversible ? "Reversible" : "Irreversible";
        status.style.color = data.reversible ? "#89d185" : "#f48771";
        
        var score = document.createElement('span');
        score.className = 'decision-date';
        score.textContent = "Risk Score: " + (data.risk_score !== undefined ? data.risk_score.toFixed(2) : "0.00");
        
        title.appendChild(status);
        title.appendChild(score);
        
        var summary = document.createElement('div');
        summary.className = 'decision-summary';
        summary.textContent = data.summary || "No summary available.";
        
        card.appendChild(title);
        card.appendChild(summary);
        
        if (data.impact_radius && data.impact_radius.length > 0) {
            var impactTitle = document.createElement('div');
            impactTitle.className = 'section-header';
            impactTitle.style.marginTop = '10px';
            impactTitle.style.fontSize = '12px';
            impactTitle.textContent = "Impact Radius (" + data.impact_radius.length + "):";
            card.appendChild(impactTitle);
            
            data.impact_radius.forEach(function(item) {
                var it = document.createElement('div');
                it.className = 'evidence-source';
                it.textContent = item;
                card.appendChild(it);
            });
        }
        
        if (data.decision_conflicts && data.decision_conflicts.length > 0) {
            var conflictTitle = document.createElement('div');
            conflictTitle.className = 'section-header';
            conflictTitle.style.marginTop = '10px';
            conflictTitle.style.fontSize = '12px';
            conflictTitle.textContent = "Decision Conflicts (" + data.decision_conflicts.length + "):";
            card.appendChild(conflictTitle);
            
            data.decision_conflicts.forEach(function(item) {
                var it = document.createElement('div');
                it.className = 'evidence-source';
                it.textContent = item;
                card.appendChild(it);
            });
        }
        
        dom.simulationList.appendChild(card);
    }

    function renderTraceability(data) {
        dom.traceabilityList.innerHTML = '';
        if (data.traversal_depth === undefined || data.trace_paths === undefined) {
            dom.traceabilitySection.classList.add('hidden');
            return;
        }
        dom.traceabilitySection.classList.remove('hidden');

        var card = document.createElement('div');
        card.className = 'decision-card';
        
        var title = document.createElement('div');
        title.className = 'decision-meta';
        var depthSpan = document.createElement('span');
        depthSpan.className = 'decision-author';
        depthSpan.textContent = "Depth: " + data.traversal_depth + " | Nodes Visited: " + (data.nodes_visited || 0) + " | Edges Traversed: " + (data.edges_traversed || 0);
        title.appendChild(depthSpan);
        card.appendChild(title);

        var summary = document.createElement('div');
        summary.className = 'decision-summary';
        summary.textContent = data.summary || "No summary available.";
        card.appendChild(summary);

        const categories = [
            { name: "Requirements", items: data.requirements },
            { name: "Decisions", items: data.decisions },
            { name: "Files", items: data.files },
            { name: "Functions", items: data.functions },
            { name: "Tests", items: data.tests }
        ];

        categories.forEach(cat => {
            if (cat.items && cat.items.length > 0) {
                var hr = document.createElement('hr');
                hr.style.borderColor = 'var(--vscode-panel-border)';
                hr.style.borderStyle = 'solid';
                hr.style.borderWidth = '1px 0 0 0';
                hr.style.margin = '15px 0';
                card.appendChild(hr);

                var catTitle = document.createElement('div');
                catTitle.className = 'section-header';
                catTitle.style.marginTop = '10px';
                catTitle.style.fontSize = '12px';
                catTitle.textContent = cat.name + " (" + cat.items.length + ")";
                card.appendChild(catTitle);
                
                cat.items.forEach(function(item) {
                    var it = document.createElement('div');
                    it.className = 'evidence-source';
                    it.textContent = item;
                    card.appendChild(it);
                });
            }
        });

        if (data.trace_paths && data.trace_paths.length > 0) {
            var hr = document.createElement('hr');
            hr.style.borderColor = 'var(--vscode-panel-border)';
            hr.style.borderStyle = 'solid';
            hr.style.borderWidth = '1px 0 0 0';
            hr.style.margin = '15px 0';
            card.appendChild(hr);

            var pathTitle = document.createElement('div');
            pathTitle.className = 'section-header';
            pathTitle.style.marginTop = '10px';
            pathTitle.style.fontSize = '12px';
            pathTitle.textContent = "Traversal Paths";
            card.appendChild(pathTitle);

            data.trace_paths.forEach(function(path) {
                var pathDiv = document.createElement('div');
                pathDiv.className = 'evidence-source';
                pathDiv.style.fontFamily = 'var(--vscode-editor-font-family)';
                pathDiv.style.marginTop = '5px';
                pathDiv.style.color = 'var(--vscode-descriptionForeground)';
                pathDiv.innerHTML = path.nodes.join(' <br/>&darr;<br/> ');
                card.appendChild(pathDiv);
            });
        }

        var hrCycle = document.createElement('hr');
        hrCycle.style.borderColor = 'var(--vscode-panel-border)';
        hrCycle.style.borderStyle = 'solid';
        hrCycle.style.borderWidth = '1px 0 0 0';
        hrCycle.style.margin = '15px 0';
        card.appendChild(hrCycle);

        var cycleTitle = document.createElement('div');
        cycleTitle.className = 'section-header';
        cycleTitle.style.marginTop = '10px';
        cycleTitle.style.fontSize = '12px';
        cycleTitle.textContent = "Cycles";
        card.appendChild(cycleTitle);

        var cycleResult = document.createElement('div');
        cycleResult.className = 'evidence-source';
        if (data.cycles_detected) {
            cycleResult.textContent = "⚠ Cycle detected";
            cycleResult.style.color = "#d29922";
        } else {
            cycleResult.textContent = "✓ None";
            cycleResult.style.color = "#89d185";
        }
        card.appendChild(cycleResult);

        dom.traceabilityList.appendChild(card);
    }

    function renderDashboard(data) {
        dom.dashboardList.innerHTML = '';
        if (!data.dashboard) {
            dom.dashboardSection.classList.add('hidden');
            return;
        }
        dom.dashboardSection.classList.remove('hidden');

        var dash = data.dashboard;

        // Container
        var container = document.createElement('div');
        container.style.display = 'flex';
        container.style.flexDirection = 'column';
        container.style.gap = '20px';

        // 1. Quick Actions
        var quickActions = document.createElement('div');
        quickActions.style.display = 'flex';
        quickActions.style.gap = '8px';
        quickActions.style.flexWrap = 'wrap';
        quickActions.style.marginBottom = '4px';

        const actions = [
            { label: 'Ingest', cmd: 'ares.ingest' },
            { label: 'Doctor', cmd: 'ares.doctor' },
            { label: 'Benchmark', cmd: 'ares.benchmark' },
            { label: 'Why Exists', cmd: 'ares.whyExists' },
            { label: 'Impact', cmd: 'ares.impactAnalysis' },
            { label: 'Traceability', cmd: 'ares.traceabilityAnalysis' }
        ];

        actions.forEach(action => {
            var btn = document.createElement('button');
            btn.textContent = action.label;
            btn.className = 'empty-command';
            btn.style.margin = '0';
            btn.style.cursor = 'pointer';
            btn.addEventListener('click', function() {
                vscode.postMessage({ command: 'executeCommand', args: [action.cmd] });
            });
            quickActions.appendChild(btn);
        });
        container.appendChild(quickActions);

        // 1. Health Banner
        var healthScore = dash.health ? dash.health.score : 0;
        var healthColor = healthScore > 90 ? '#89d185' : (healthScore > 70 ? '#d29922' : '#f48771');
        var healthStatus = dash.health ? dash.health.status : 'Unknown';
        var banner = document.createElement('div');
        banner.className = 'decision-card';
        banner.style.display = 'flex';
        banner.style.alignItems = 'center';
        banner.style.justifyContent = 'space-between';
        banner.style.background = 'color-mix(in srgb, var(--vscode-editor-foreground) 4%, transparent)';
        banner.style.borderLeft = '4px solid ' + healthColor;
        
        var refreshingHtml = dash.refreshing ? '<span style="font-size: 11px; margin-left: 10px; color: var(--vscode-descriptionForeground); font-weight: normal; animation: pulse 1.5s infinite;">↻ Refreshing...</span>' : '';
        banner.innerHTML = '<div style="font-size: 16px; font-weight: 600;">Repository Health' + refreshingHtml + '</div><div style="font-size: 24px; font-weight: 700; color: ' + healthColor + '">' + healthScore + '/100 (' + healthStatus + ')</div>';
        container.appendChild(banner);

        // Helper to create a section card
        function createSectionCard(title, command) {
            var card = document.createElement('div');
            card.className = 'decision-card';
            card.style.cursor = 'pointer';
            card.addEventListener('click', function() {
                vscode.postMessage({ command: 'executeCommand', args: [command] });
            });
            card.addEventListener('mouseover', function() { card.style.borderColor = 'var(--vscode-button-background)'; });
            card.addEventListener('mouseout', function() { card.style.borderColor = 'var(--vscode-panel-border)'; });

            var header = document.createElement('div');
            header.className = 'section-header';
            header.style.marginBottom = '12px';
            header.style.border = 'none';
            header.style.padding = '0';
            header.style.background = 'transparent';
            header.textContent = title;
            card.appendChild(header);

            var grid = document.createElement('div');
            grid.style.display = 'grid';
            grid.style.gridTemplateColumns = '1fr 1fr';
            grid.style.gap = '12px';
            card.appendChild(grid);

            return { card, grid };
        }

        function addMetricRow(grid, label, value, color) {
            var row = document.createElement('div');
            row.style.display = 'flex';
            row.style.justifyContent = 'space-between';
            row.style.fontSize = '12px';
            
            var lbl = document.createElement('span');
            lbl.style.color = 'var(--vscode-descriptionForeground)';
            lbl.textContent = label;
            
            var val = document.createElement('span');
            val.style.fontWeight = '600';
            val.style.color = color || 'var(--vscode-editor-foreground)';
            val.textContent = value;

            row.appendChild(lbl);
            row.appendChild(val);
            grid.appendChild(row);
        }

        // 2. Repository
        if (dash.repository) {
            var repoCard = createSectionCard('Repository', 'ares.doctor');
            addMetricRow(repoCard.grid, 'Name', dash.repository.name);
            addMetricRow(repoCard.grid, 'Language', dash.repository.language);
            addMetricRow(repoCard.grid, 'Branch', dash.repository.branch);
            addMetricRow(repoCard.grid, 'Commit', dash.repository.commit);
            addMetricRow(repoCard.grid, 'Indexed', dash.repository.indexed ? '✓' : '✗', dash.repository.indexed ? '#89d185' : '#f48771');
            addMetricRow(repoCard.grid, 'Last Ingest', dash.repository.last_ingest);
            addMetricRow(repoCard.grid, 'Dirty', dash.repository.is_dirty ? 'Yes' : 'No', dash.repository.is_dirty ? '#f48771' : 'var(--vscode-editor-foreground)');
            addMetricRow(repoCard.grid, 'Files', dash.repository.files);
            addMetricRow(repoCard.grid, 'Functions', dash.repository.functions);
            addMetricRow(repoCard.grid, 'Modules', dash.repository.modules);
            container.appendChild(repoCard.card);
        }

        // 3. Knowledge Graph
        if (dash.graph) {
            var graphCard = createSectionCard('Knowledge Graph', 'ares.benchmark');
            addMetricRow(graphCard.grid, 'Nodes', dash.graph.nodes);
            addMetricRow(graphCard.grid, 'Edges', dash.graph.edges);
            addMetricRow(graphCard.grid, 'Depth', dash.graph.depth);
            addMetricRow(graphCard.grid, 'Average Degree', dash.graph.average_degree ? dash.graph.average_degree.toFixed(2) : '0');
            container.appendChild(graphCard.card);
        }

        // 4. Intelligence
        if (dash.intelligence) {
            var intelCard = createSectionCard('Intelligence', 'ares.whyExists');
            
            function getStatusColor(status) {
                if (!status) return '#d29922';
                let s = status.toUpperCase();
                if (s.includes('READY')) return '#89d185';
                if (s.includes('NOT AVAILABLE') || s.includes('NONE')) return '#f48771';
                return '#d29922';
            }
            
            addMetricRow(intelCard.grid, 'Why Exists', dash.intelligence.why_exists_status, getStatusColor(dash.intelligence.why_exists_status));
            addMetricRow(intelCard.grid, 'Graph', dash.intelligence.graph_status, getStatusColor(dash.intelligence.graph_status));
            addMetricRow(intelCard.grid, 'Git Memory', dash.intelligence.git_memory_status, getStatusColor(dash.intelligence.git_memory_status));
            addMetricRow(intelCard.grid, 'Ownership', dash.intelligence.ownership_status, getStatusColor(dash.intelligence.ownership_status));
            addMetricRow(intelCard.grid, 'Requirements', dash.intelligence.requirements_status, getStatusColor(dash.intelligence.requirements_status));
            addMetricRow(intelCard.grid, 'Governance', dash.intelligence.governance_status, getStatusColor(dash.intelligence.governance_status));
            addMetricRow(intelCard.grid, 'Impact', dash.intelligence.impact_status, getStatusColor(dash.intelligence.impact_status));
            addMetricRow(intelCard.grid, 'Traceability', dash.intelligence.traceability_status, getStatusColor(dash.intelligence.traceability_status));
            addMetricRow(intelCard.grid, 'Simulation', dash.intelligence.simulation_status, getStatusColor(dash.intelligence.simulation_status));
            addMetricRow(intelCard.grid, 'Drift', dash.intelligence.drift_status, getStatusColor(dash.intelligence.drift_status));
            container.appendChild(intelCard.card);
        }

        // 5. Integrity
        if (dash.integrity) {
            var intCard = createSectionCard('Graph Integrity', 'ares.doctor');
            var fkColor = dash.integrity.foreign_keys_passed ? '#89d185' : '#f48771';
            addMetricRow(intCard.grid, 'Foreign Keys', dash.integrity.foreign_keys_passed ? 'PASS' : 'FAIL', fkColor);
            addMetricRow(intCard.grid, 'Missing Targets', dash.integrity.missing_targets, dash.integrity.missing_targets > 0 ? '#f48771' : '#89d185');
            addMetricRow(intCard.grid, 'Missing Sources', dash.integrity.missing_sources, dash.integrity.missing_sources > 0 ? '#f48771' : '#89d185');
            addMetricRow(intCard.grid, 'Orphans', dash.integrity.orphans, dash.integrity.orphans > 0 ? '#d29922' : '#89d185');
            addMetricRow(intCard.grid, 'Cycles', dash.integrity.cycles, dash.integrity.cycles > 0 ? '#f48771' : '#89d185');
            container.appendChild(intCard.card);
        }

        // 6. Coverage
        if (dash.coverage) {
            var covCard = createSectionCard('Coverage', 'ares.coverageAnalysis');
            addMetricRow(covCard.grid, 'Git History', dash.coverage.git_history_enabled ? '✓' : '✗', dash.coverage.git_history_enabled ? '#89d185' : '#f48771');
            addMetricRow(covCard.grid, 'Ownership', dash.coverage.ownership_enabled ? 'Enabled' : 'Disabled', dash.coverage.ownership_enabled ? '#89d185' : '#d29922');
            addMetricRow(covCard.grid, 'Requirements', dash.coverage.requirements);
            addMetricRow(covCard.grid, 'ADRs', dash.coverage.adrs);
            addMetricRow(covCard.grid, 'Decisions', dash.coverage.decisions);
            addMetricRow(covCard.grid, 'Architecture Docs', dash.coverage.architecture_docs);
            container.appendChild(covCard.card);
        }

        // 7. Performance
        if (dash.performance) {
            var perfCard = createSectionCard('Performance', 'ares.benchmark');
            addMetricRow(perfCard.grid, 'Scanner', dash.performance.scanner_ms + ' ms');
            addMetricRow(perfCard.grid, 'AST Parsing', dash.performance.ast_parsing_ms + ' ms');
            addMetricRow(perfCard.grid, 'Git Memory', dash.performance.git_memory_ms + ' ms');
            addMetricRow(perfCard.grid, 'Knowledge Graph', dash.performance.knowledge_graph_ms + ' ms');
            addMetricRow(perfCard.grid, 'Persistence', dash.performance.persistence_ms + ' ms');
            addMetricRow(perfCard.grid, 'Total Ingest Time', dash.performance.total_time_ms + ' ms');
            container.appendChild(perfCard.card);
        }

        // 8. Activity
        if (dash.activity && dash.activity.length > 0) {
            var actCard = createSectionCard('Recent Activity', 'ares.ingest');
            dash.activity.forEach(function(evt) {
                addMetricRow(actCard.grid, '✔ ' + evt.message, evt.relative_time, '#89d185');
            });
            container.appendChild(actCard.card);
        }

        // 9. Recent Queries
        if (data.recent_queries && data.recent_queries.length > 0) {
            var qCard = document.createElement('div');
            qCard.className = 'decision-card';
            
            var qHeader = document.createElement('div');
            qHeader.className = 'section-header';
            qHeader.style.marginBottom = '12px';
            qHeader.style.border = 'none';
            qHeader.style.padding = '0';
            qHeader.style.background = 'transparent';
            qHeader.textContent = 'Recent Queries';
            qCard.appendChild(qHeader);

            data.recent_queries.forEach(function(q) {
                var row = document.createElement('div');
                row.style.display = 'flex';
                row.style.justifyContent = 'space-between';
                row.style.fontSize = '12px';
                row.style.marginBottom = '8px';
                row.style.cursor = 'pointer';
                
                var cmdSpan = document.createElement('span');
                cmdSpan.style.color = 'var(--vscode-textLink-foreground)';
                cmdSpan.style.fontWeight = '600';
                cmdSpan.textContent = q.command + (q.target ? ' (' + q.target + ')' : '');

                var timeSpan = document.createElement('span');
                timeSpan.style.color = 'var(--vscode-descriptionForeground)';
                var date = new Date(q.timestamp);
                var diff = Math.floor((new Date() - date) / 1000 / 60);
                timeSpan.textContent = diff < 1 ? 'Just now' : (diff < 60 ? diff + 'm ago' : Math.floor(diff/60) + 'h ago');

                row.appendChild(cmdSpan);
                row.appendChild(timeSpan);
                
                row.addEventListener('click', function() {
                    let cmdId = q.command.toLowerCase().replace(/ /g, '');
                    if (cmdId === 'whyexists') cmdId = 'whyExists';
                    if (cmdId === 'impactanalysis') cmdId = 'impactAnalysis';
                    if (cmdId === 'traceabilityanalysis') cmdId = 'traceabilityAnalysis';
                    if (cmdId === 'driftanalysis') cmdId = 'driftAnalysis';
                    
                    let vscodeCmd = 'ares.' + cmdId;
                    vscode.postMessage({ command: 'executeCommand', args: [vscodeCmd] });
                });
                
                row.addEventListener('mouseover', function() { cmdSpan.style.textDecoration = 'underline'; });
                row.addEventListener('mouseout', function() { cmdSpan.style.textDecoration = 'none'; });

                qCard.appendChild(row);
            });
            container.appendChild(qCard);
        }

        // 10. Cache Stats
        if (dash.cache_stats) {
            var cacheCard = createSectionCard('Overview Cache', 'ares.benchmark');
            addMetricRow(cacheCard.grid, 'Hit Rate', dash.cache_stats.hit_rate);
            addMetricRow(cacheCard.grid, 'Age', dash.cache_stats.age.toFixed(1) + ' seconds');
            addMetricRow(cacheCard.grid, 'TTL', dash.cache_stats.ttl.toFixed(1) + ' seconds');
            addMetricRow(cacheCard.grid, 'State', dash.refreshing ? 'Refreshing (tokio::spawn)' : 'Idle');
            container.appendChild(cacheCard.card);
        }

        // 11. Version Info
        if (dash.version) {
            var versionLabel = document.createElement('div');
            versionLabel.style.fontSize = '11px';
            versionLabel.style.color = 'var(--vscode-descriptionForeground)';
            versionLabel.style.textAlign = 'center';
            versionLabel.style.marginTop = '10px';
            versionLabel.textContent = 'ARES MemoryOS v' + dash.version.ares_version + ' | DB Schema v' + dash.version.schema_version;
            container.appendChild(versionLabel);
        }

        dom.dashboardList.appendChild(container);
    }

    function renderEmptyState() {
        dom.resultContent.classList.add('hidden');
        dom.emptyState.classList.remove('hidden');
    }

    // --- Top-level state transitions ---

    function showLoading() {
        hideAll();
        dom.loading.classList.remove('hidden');
    }

    function showError(err) {
        hideAll();
        dom.errorTitle.textContent = err.message || 'Unable to retrieve repository memory';
        dom.errorMessage.textContent = err.detail || 'The query could not be completed. See possible reasons below.';
        dom.error.classList.remove('hidden');
    }

    function showData(data) {
        hideAll();
        dom.content.classList.remove('hidden');

        renderHeader(data);

        if (isEmpty(data)) {
            renderEmptyState();
            return;
        }

        dom.resultContent.classList.remove('hidden');
        dom.emptyState.classList.add('hidden');

        renderAnswer(data);
        renderConfidence(data);
        renderEvidence(data);
        renderDecisions(data);
        renderDrift(data);
        renderSimulation(data);
        renderTraceability(data);
        renderDashboard(data);
    }

    // --- Message listener ---
    window.addEventListener('message', function (event) {
        var msg = event.data;
        switch (msg.type) {
            case 'loading': showLoading(); break;
            case 'update':  showData(msg.data); break;
            case 'error':   showError(msg.error); break;
        }
    });
    
    // Background polling for ARES Home
    setInterval(function() {
        if (!dom.dashboardSection.classList.contains('hidden')) {
            vscode.postMessage({ command: 'refreshDashboard' });
        }
    }, 5000);
})();
</script>
</body>
</html>`;
    }
}
