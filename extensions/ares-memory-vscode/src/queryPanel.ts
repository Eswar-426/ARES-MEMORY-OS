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
    execution_time_ms?: number;
    gaps?: any[];
    health_score?: number;
    score_breakdown?: any;
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

        // Handle messages from the webview
        this.panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'executeCommand':
                        if (message.args && message.args.length > 0) {
                            vscode.commands.executeCommand(message.args[0], ...message.args.slice(1));
                        }
                        return;
                    case 'openFile':
                        if (message.path) {
                            vscode.workspace.openTextDocument(message.path).then(doc => {
                                vscode.window.showTextDocument(doc).then(editor => {
                                    if (message.line !== undefined) {
                                        const line = Math.max(0, message.line - 1);
                                        const char = message.column !== undefined ? Math.max(0, message.column - 1) : 0;
                                        const pos = new vscode.Position(line, char);
                                        editor.selection = new vscode.Selection(pos, pos);
                                        editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
                                    }
                                });
                            }, err => {
                                vscode.window.showErrorMessage(`Could not open file: ${err.message}`);
                            });
                        }
                        return;
                }
            },
            null,
            this.disposables
        );
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
        inst.panel.webview.html = AresQueryPanel.getHtml();
        setTimeout(() => {
            inst.postMessage({ type: 'loading' });
        }, 150);
        return inst;
    }

    /** Show a successful response. */
    public static show(context: vscode.ExtensionContext, data: AresResponse): AresQueryPanel {
        const inst = AresQueryPanel.ensurePanel(context);
        // Force fresh HTML to break VS Code's webview restore cache,
        // then delay postMessage until the new JS has loaded its listener.
        inst.panel.webview.html = AresQueryPanel.getHtml();
        setTimeout(() => {
            inst.postMessage({ type: 'update', data });
        }, 150);
        return inst;
    }

    /** Show a user-friendly error inside the panel. */
    public static showError(context: vscode.ExtensionContext, error: AresError): AresQueryPanel {
        const inst = AresQueryPanel.ensurePanel(context);
        inst.panel.webview.html = AresQueryPanel.getHtml();
        setTimeout(() => {
            inst.postMessage({ type: 'error', error });
        }, 150);
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

.confidence-reasons {
    margin-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.confidence-reason {
    font-size: 11px;
    color: var(--vscode-descriptionForeground);
    padding: 1px 0;
    line-height: 1.4;
}

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

/* ─── Insight Cards 2×2 ─── */
.dash-cards-grid{
    display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-bottom:24px;
}
.dash-insight-card{
    padding:16px;border-radius:10px;
    background:color-mix(in srgb,var(--vscode-editor-foreground) 3%,transparent);
    border:1px solid var(--vscode-panel-border);
    transition:border-color .15s;
}
.dash-insight-card:hover{border-color:color-mix(in srgb,var(--vscode-button-background) 50%,transparent)}
.dash-card-title{
    font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:.8px;
    color:var(--vscode-descriptionForeground);margin-bottom:12px;
    display:flex;align-items:center;gap:6px;
}
.dash-card-title-icon{font-size:14px}
.dash-card-rows{display:flex;flex-direction:column;gap:6px}
.dash-card-row{
    display:flex;justify-content:space-between;align-items:baseline;
    font-size:12px;
}
.dash-card-label{color:var(--vscode-descriptionForeground)}
.dash-card-value{font-weight:600;font-size:13px;color:var(--vscode-editor-foreground)}
.dash-card-value-accent{font-weight:700;font-size:16px;color:var(--vscode-button-background)}
.dash-card-value-warn{font-weight:600;color:#d29922}
.dash-card-value-ok{font-weight:600;color:#2ea043}
.dash-card-value-muted{color:var(--vscode-descriptionForeground)}

/* ─── Section Title ─── */
.dash-section-title{
    font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:.8px;
    color:var(--vscode-descriptionForeground);margin-bottom:12px;
}

/* ─── Descriptive Action Cards ─── */
.dash-actions-grid{
    display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-bottom:24px;
}
.dash-action-card{
    display:flex;align-items:flex-start;gap:12px;
    padding:14px 16px;border-radius:10px;cursor:pointer;text-align:left;
    background:color-mix(in srgb,var(--vscode-button-background) 6%,transparent);
    border:1px solid color-mix(in srgb,var(--vscode-button-background) 15%,transparent);
    color:var(--vscode-editor-foreground);
    transition:all .2s ease;
}
.dash-action-card:hover{
    background:color-mix(in srgb,var(--vscode-button-background) 15%,transparent);
    border-color:var(--vscode-button-background);
    transform:translateY(-1px);
    box-shadow:0 2px 8px rgba(0,0,0,.15);
}
.dash-action-icon{font-size:24px;flex-shrink:0;margin-top:1px}
.dash-action-text{display:flex;flex-direction:column;gap:2px;min-width:0}
.dash-action-label{font-size:13px;font-weight:600;line-height:1.3}
.dash-action-desc{font-size:11px;color:var(--vscode-descriptionForeground);line-height:1.4}

.dash-header{
    display:flex;align-items:center;gap:16px;
    padding:20px 0;margin-bottom:20px;
    border-bottom:1px solid var(--vscode-panel-border);
}
.dash-header-icon{font-size:36px}
.dash-repo-name{font-size:20px;font-weight:700;letter-spacing:-.4px}
.dash-repo-status{display:flex;gap:8px;margin-top:4px}
.dash-badge{
    font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:.5px;
    padding:2px 8px;border-radius:4px;
    background:color-mix(in srgb,var(--vscode-editor-foreground) 8%,transparent);
    color:var(--vscode-descriptionForeground);
}
.dash-badge-ok{background:color-mix(in srgb,#2ea043 15%,transparent);color:#2ea043}
.dash-badge-warn{background:color-mix(in srgb,#d29922 15%,transparent);color:#d29922}
</style>

</head>
<body>

<!-- Loading -->
<div id="loadingState" class="loading-state">
    <div class="loading-spinner"></div>
    <div class="loading-text">Querying ARES Memory…</div>
</div>

<!-- Error -->
<div id="dashboardSection" class="section hidden">
    <div class="section-title">ARES REPOSITORY HEALTH</div>
    <div id="dashboardList" class="card-list"></div>
</div>

<div id="gapsSection" class="section hidden">
    <div id="healthScore" style="margin-bottom: 20px;"></div>
    <div id="gapCounts" style="display: flex; gap: 10px; margin-bottom: 20px; flex-wrap: wrap;"></div>
    <div class="section-title">TOP GAPS</div>
    <div id="gapList" class="card-list"></div>
</div>

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
            <div id="executionTime" class="query-badge hidden" style="background:var(--vscode-editorInfo-background);color:var(--vscode-editorInfo-foreground);margin-left:8px;"></div>
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
            <div id="confidenceSection" class="section-body">
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
            <div class="section-header" id="evidenceHeader">Evidence</div>
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
        confidenceSection: document.getElementById('confidenceSection'),
        evidenceSection:  document.getElementById('evidenceSection'),
        evidenceHeader:   document.getElementById('evidenceHeader'),
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
        gapsSection: document.getElementById('gapsSection'),
        healthScore: document.getElementById('healthScore'),
        gapCounts: document.getElementById('gapCounts'),
        gapList: document.getElementById('gapList'),
        dashboardList:    document.getElementById('dashboardList'),
        emptyState:       document.getElementById('emptyState'),
        executionTime:    document.getElementById('executionTime'),
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

        if (data.execution_time_ms) {
            dom.executionTime.textContent = data.execution_time_ms + ' ms';
            dom.executionTime.classList.remove('hidden');
        } else {
            dom.executionTime.classList.add('hidden');
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

        // Confidence bar fix — set directly since renderConfidence may have stale DOM refs
        var raw = data.confidence;
        var score = 0;
        if (typeof raw === 'number') { score = raw; }
        else if (typeof raw === 'string') { score = parseFloat(raw) || 0; }
        else if (raw && typeof raw === 'object') { score = parseFloat(raw.score) || 0; }
        var pct = Math.min(100, Math.max(0, Math.round(score)));

        var level = pct <= 33 ? 'low' : pct <= 66 ? 'medium' : 'high';
        var levelText = pct <= 33 ? 'Low' : pct <= 66 ? 'Medium' : 'High';

        dom.confidenceLabel.textContent = levelText;
        dom.confidenceLabel.className = 'confidence-label ' + level;
        dom.confidenceBar.className = 'confidence-bar-fill ' + level;
        dom.confidenceBar.style.width = pct + '%';
        dom.confidencePct.textContent = pct + '%';
        dom.confidencePct.className = 'confidence-pct ' + level;

        // Reasons checklist
        var reasons = (raw && typeof raw === 'object' && Array.isArray(raw.reasons)) ? raw.reasons : [];
        var rc = document.getElementById('confidenceSection');
        if (rc) {
            var existing = rc.querySelector('.confidence-reasons');
            if (existing) existing.remove();
            if (reasons.length > 0) {
                var div = document.createElement('div');
                div.className = 'confidence-reasons';
                for (var i = 0; i < reasons.length; i++) {
                    var item = document.createElement('div');
                    item.className = 'confidence-reason';
                    item.textContent = '✓ ' + reasons[i];
                    div.appendChild(item);
                }
                rc.appendChild(div);
            }
        }
    }


    function renderEvidence(data) {
        dom.evidenceList.innerHTML = '';
        if (!data.evidence || data.evidence.length === 0) {
            dom.evidenceSection.classList.add('hidden');
            return;
        }
        dom.evidenceSection.classList.remove('hidden');
        dom.evidenceHeader.textContent = 'Evidence (' + data.evidence.length + ')';

        data.evidence.forEach(function (ev) {
            var item = document.createElement('div');
            item.className = 'evidence-item';

            // New schema: ev.category + ev.value
            // Fallback: ev.source + ev.detail (backward compat)
            var category = ev.category || ev.source || 'unknown';
            var value = ev.value || ev.detail || '';

            var src = document.createElement('a');
            src.className = 'evidence-source';
            src.textContent = category;
            src.title = category + ': ' + value;
            // Only make clickable if it looks like a file path
            if (category.indexOf('/') !== -1 || category.indexOf('.') !== -1) {
                src.addEventListener('click', function () {
                    vscode.postMessage({
                        command: 'openFile',
                        path: category,
                    });
                });
            }
            item.appendChild(src);

            if (value && value !== category) {
                var detail = document.createElement('div');
                detail.className = 'evidence-detail';
                detail.textContent = value;
                item.appendChild(detail);
            }

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
        // Only render drift section for actual drift queries
        if (!data.metadata || data.metadata.generator !== "DriftGenerator") {
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

    function renderGaps(data) {
        if (!data.gaps) {
            dom.gapsSection.classList.add('hidden');
            return;
        }

        dom.gapsSection.classList.remove('hidden');
        dom.healthScore.innerHTML = '';
        dom.gapCounts.innerHTML = '';
        dom.gapList.innerHTML = '';

        // Hide other sections
        document.getElementById('evidenceSection')?.classList.add('hidden');
        document.getElementById('relatedSection')?.classList.add('hidden');
        document.getElementById('traceabilitySection')?.classList.add('hidden');
        document.getElementById('dashboardSection')?.classList.add('hidden');

        var scoreStr = '<div style="padding:16px;background:var(--vscode-editor-inactiveSelectionBackground);border-radius:8px;">';
        scoreStr += '<h2 style="margin-bottom:8px;">REPOSITORY HEALTH &nbsp; <span style="float:right">Score: ' + Math.round(data.health_score || 0) + '</span></h2>';
        scoreStr += '<div style="width:100%;height:10px;background:var(--vscode-editor-background);border-radius:5px;overflow:hidden;margin-bottom:12px;">';
        var color = data.health_score > 70 ? 'var(--vscode-testing-iconPassed)' : data.health_score > 40 ? 'var(--vscode-testing-iconQueued)' : 'var(--vscode-testing-iconFailed)';
        scoreStr += '<div style="width:' + Math.round(data.health_score || 0) + '%;height:100%;background:' + color + ';"></div>';
        scoreStr += '</div>';

        if (data.score_breakdown && data.score_breakdown.overall !== undefined) {
            var bd = data.score_breakdown;
            scoreStr += '<div style="display:flex; justify-content:space-between; font-size:11px; opacity:0.8;">';
            scoreStr += '<span>Files w/ Decisions (40%): <b>' + Math.round((bd.files_with_decisions_term || 0) * 100) + '%</b></span>';
            scoreStr += '<span>Decisions w/ Reqs (30%): <b>' + Math.round((bd.decisions_with_requirements_term || 0) * 100) + '%</b></span>';
            scoreStr += '<span>Files w/ Owners (20%): <b>' + Math.round((bd.files_with_owners_term || 0) * 100) + '%</b></span>';
            scoreStr += '<span>Fresh Decisions (10%): <b>' + Math.round((bd.fresh_decisions_term || 0) * 100) + '%</b></span>';
            scoreStr += '</div>';
        }

        scoreStr += '</div>';
        dom.healthScore.innerHTML = scoreStr;

        var counts = {};
        data.gaps.forEach(function(g) { counts[g.gap_type] = (counts[g.gap_type] || 0) + 1; });
        
        var labels = {
            'unknown_ownership': 'Unknown Ownership',
            'code_without_decision': 'Code w/o Decision',
            'stale_decision': 'Stale Decision',
            'decision_without_code': 'Decision w/o Code',
            'orphaned_requirement': 'Orphaned Req'
        };

        Object.keys(labels).forEach(function(type) {
            var c = counts[type] || 0;
            var badge = document.createElement('div');
            badge.style.padding = '8px 12px';
            badge.style.background = 'var(--vscode-button-secondaryBackground)';
            badge.style.borderRadius = '6px';
            badge.style.textAlign = 'center';
            badge.style.minWidth = '120px';
            badge.innerHTML = '<div style="font-size:11px;opacity:0.8">' + labels[type] + '</div><div style="font-size:18px;font-weight:bold">' + c + '</div>';
            dom.gapCounts.appendChild(badge);
        });

        var prio = { 'unknown_ownership': 1, 'code_without_decision': 2, 'stale_decision': 3, 'decision_without_code': 4, 'orphaned_requirement': 5 };
        var sorted = data.gaps.sort(function(a, b) { return prio[a.gap_type] - prio[b.gap_type]; });
        var top10 = sorted.slice(0, 10);
        
        var icons = { 'unknown_ownership': '🔴', 'code_without_decision': '🟡', 'stale_decision': '🟡', 'decision_without_code': '⚪', 'orphaned_requirement': '⚪' };

        top10.forEach(function(gap) {
            var card = document.createElement('div');
            card.className = 'card animate-slide';
            card.innerHTML = '<div class="card-header"><div class="card-title">' + (icons[gap.gap_type] || '') + ' ' + (labels[gap.gap_type] || gap.gap_type) + '</div></div>' +
                             '<div class="card-body"><code>' + gap.node_label + '</code><p style="margin-top:8px;opacity:0.8">' + gap.details + '</p></div>';
            
            var btn = document.createElement('button');
            btn.className = 'nav-button';
            btn.style.marginTop = '12px';
            btn.style.width = '100%';
            btn.innerText = 'Create Decision →';
            btn.onclick = function() {
                vscode.postMessage({ command: 'executeCommand', args: ['ares.createDecisionFromGap', gap.node_label, gap.details] });
            };
            
            card.appendChild(btn);
            dom.gapList.appendChild(card);
        });
    }

    function renderDashboard(data) {
        if (data.query_type === 'healthCheck' || data.gaps) {
            renderGaps(data);
            return;
        }

        if (!data.dashboard) {
            dom.dashboardSection.classList.add('hidden');
            return;
        }

        var dash = data.dashboard;
        var repo = dash.repository || {};
        var graph = dash.graph || dash.knowledge_graph || {};
        var integrity = dash.integrity || {};

        document.getElementById('dashboardSection')?.classList.remove('hidden');
        dom.dashboardList.innerHTML = '';

        // Hide other generic sections
        document.querySelectorAll('#resultContent > .section').forEach(function(el) {
            if (el !== dom.dashboardSection) {
                el.classList.add('hidden');
            }
        });

        var html = '';

        // ─── Repository Header ───────────────────────────────
        html += '<div class="dash-header">';
        html += '<div class="dash-header-icon">📦</div>';
        html += '<div class="dash-header-info">';
        html += '<div class="dash-repo-name">' + (repo.name || 'Repository') + '</div>';
        html += '<div class="dash-repo-status">';
        if (repo.indexed) {
            html += '<span class="dash-badge dash-badge-ok">Indexed ✓</span>';
        } else {
            html += '<span class="dash-badge dash-badge-warn">Not Indexed</span>';
        }
        if (repo.commit) {
            html += '<span class="dash-badge">' + repo.commit.substring(0, 7) + '</span>';
        }
        html += '</div></div></div>';

        // ─── 4 Insight Cards (2×2) ──────────────────────────
        html += '<div class="dash-cards-grid">';

        html += '<div class="dash-insight-card">';
        html += '<div class="dash-card-title"><span class="dash-card-title-icon">📂</span> Repository</div>';
        html += '<div class="dash-card-rows">';
        html += '<div class="dash-card-row"><span class="dash-card-label">Files</span><span class="dash-card-value">' + (repo.files || 0) + '</span></div>';
        html += '<div class="dash-card-row"><span class="dash-card-label">Modules</span><span class="dash-card-value">' + (graph.modules || repo.modules || 0) + '</span></div>';
        html += '<div class="dash-card-row"><span class="dash-card-label">Nodes</span><span class="dash-card-value-accent">' + (graph.nodes || 0) + '</span></div>';
        html += '</div></div>';

        html += '<div class="dash-insight-card">';
        html += '<div class="dash-card-title"><span class="dash-card-title-icon">🔗</span> Graph</div>';
        html += '<div class="dash-card-rows">';
        html += '<div class="dash-card-row"><span class="dash-card-label">Edges</span><span class="dash-card-value">' + (graph.edges || 0) + '</span></div>';
        html += '<div class="dash-card-row"><span class="dash-card-label">Orphans</span><span class="' + ((integrity.orphan_nodes || 0) > 10 ? 'dash-card-value-warn' : 'dash-card-value') + '">' + (integrity.orphan_nodes || 0) + '</span></div>';
        var connectivity = (graph.nodes > 0 && graph.edges > 0) ? Math.min(100, Math.round((1 - (integrity.orphan_nodes || 0) / graph.nodes) * 100)) : 0;
        html += '<div class="dash-card-row"><span class="dash-card-label">Connected</span><span class="' + (connectivity > 90 ? 'dash-card-value-ok' : 'dash-card-value') + '">' + connectivity + '%</span></div>';
        html += '</div></div>';

        html += '<div class="dash-insight-card">';
        html += '<div class="dash-card-title"><span class="dash-card-title-icon">⏱️</span> Activity</div>';
        html += '<div class="dash-card-rows">';
        var lastIngest = repo.last_ingest || repo.updated_at;
        html += '<div class="dash-card-row"><span class="dash-card-label">Last ingest</span><span class="dash-card-value-muted">' + (lastIngest ? 'Just now' : '—') + '</span></div>';
        var lastQuery = (data.recent_queries && data.recent_queries.length > 0) ? data.recent_queries[0] : null;
        html += '<div class="dash-card-row"><span class="dash-card-label">Last query</span><span class="dash-card-value-muted">' + (lastQuery ? 'Recently' : '—') + '</span></div>';
        html += '<div class="dash-card-row"><span class="dash-card-label">Queries</span><span class="dash-card-value">' + (data.recent_queries ? data.recent_queries.length : 0) + '</span></div>';
        html += '</div></div>';

        html += '<div class="dash-insight-card">';
        html += '<div class="dash-card-title"><span class="dash-card-title-icon">💡</span> Insights</div>';
        html += '<div class="dash-card-rows">';
        var typeCount = Object.keys(graph.types || {}).length;
        html += '<div class="dash-card-row"><span class="dash-card-label">Node types</span><span class="dash-card-value">' + typeCount + '</span></div>';
        html += '<div class="dash-card-row"><span class="dash-card-label">Avg degree</span><span class="dash-card-value">' + (graph.nodes > 0 ? ((graph.edges || 0) * 2 / graph.nodes).toFixed(1) : '0') + '</span></div>';
        html += '<div class="dash-card-row"><span class="dash-card-label">Density</span><span class="dash-card-value-muted">' + (graph.nodes > 1 ? ((graph.edges || 0) / (graph.nodes * (graph.nodes - 1) / 2) * 100).toFixed(2) + '%' : '—') + '</span></div>';
        html += '</div></div>';

        html += '</div>';

        // ─── Quick Actions (descriptive cards) ──────────────
        html += '<div class="dash-section-title">Quick Actions</div>';
        html += '<div class="dash-actions-grid">';
        var actions = [
            { icon: '🌐', label: 'Graph Explorer', desc: 'Visualize repository architecture.', cmd: 'ares.graphExplorer' },
            { icon: '🧠', label: 'Why Exists', desc: 'Ask why any file or module exists.', cmd: 'ares.whyExists' },
            { icon: '🧬', label: 'Impact Analysis', desc: 'See what breaks if something changes.', cmd: 'ares.impactAnalysis' },
            { icon: '📐', label: 'Traceability', desc: 'Trace connections across the codebase.', cmd: 'ares.traceabilityAnalysis' },
            { icon: '📊', label: 'Drift Analysis', desc: 'Detect code that drifted from intent.', cmd: 'ares.driftAnalysis' },
            { icon: '🔄', label: 'Ingest / Refresh', desc: 'Re-index repository into memory.', cmd: 'ares.ingest' },
        ];
        for (var j = 0; j < actions.length; j++) {
            var a = actions[j];
            html += '<button class="dash-action-card" data-cmd="' + a.cmd + '">';
            html += '<span class="dash-action-icon">' + a.icon + '</span>';
            html += '<span class="dash-action-text">';
            html += '<span class="dash-action-label">' + a.label + '</span>';
            html += '<span class="dash-action-desc">' + a.desc + '</span>';
            html += '</span>';
            html += '</button>';
        }
        html += '</div>';

        dom.dashboardList.innerHTML = html;

        // Attach event listeners after rendering
        document.querySelectorAll('.dash-action-card').forEach(function(btn) {
            btn.addEventListener('click', function() {
                vscode.postMessage({ command: 'executeCommand', args: [btn.getAttribute('data-cmd')] });
            });
        });
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
        try {
            hideAll();
            dom.content.classList.remove('hidden');

            renderHeader(data);

            if (isEmpty(data)) {
                renderEmptyState();
                return;
            }

            dom.resultContent.classList.remove('hidden');
            dom.emptyState.classList.add('hidden');

            // Hide other generic sections
            document.getElementById('evidenceSection')?.classList.add('hidden');
            document.getElementById('relatedSection')?.classList.add('hidden');
            document.getElementById('traceabilitySection')?.classList.add('hidden');
            document.getElementById('gapsSection')?.classList.add('hidden');

            if (data.query_type !== 'ARES Home') {
                dom.answer.parentElement.parentElement.classList.remove('hidden');
                dom.confidenceLabel.parentElement.parentElement.parentElement.classList.remove('hidden');
                renderAnswer(data);
                if (data.evidence && data.evidence.length > 0) {
                    renderEvidence(data);
                } else {
                    dom.evidenceSection.classList.add('hidden');
                }
                renderDecisions(data);
                // Drift verdict is now rendered in the narrative answer — legacy widget disabled
                dom.driftSection.classList.add('hidden');
                renderSimulation(data);
                renderTraceability(data);
            } else {
                dom.answer.parentElement.parentElement.classList.add('hidden');
                dom.confidenceLabel.parentElement.parentElement.parentElement.classList.add('hidden');
                dom.evidenceSection.classList.add('hidden');
                dom.decisionsSection.classList.add('hidden');
                dom.driftSection.classList.add('hidden');
                dom.simulationSection.classList.add('hidden');
                dom.traceabilitySection.classList.add('hidden');
            }
            renderDashboard(data);
        } catch (e) {
            showError({message: "UI Render Error", detail: e.toString()});
        }
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
