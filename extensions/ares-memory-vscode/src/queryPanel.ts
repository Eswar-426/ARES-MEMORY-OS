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

export interface AresResponse {
    answer: string;
    confidence: number;
    evidence: AresEvidence[];
    related_decisions: AresDecision[];
    query_type: string;
    file_path?: string;
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
               (!d.evidence || d.evidence.length === 0);
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
})();
</script>
</body>
</html>`;
    }
}
