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
        inst.panel.title = 'ARES Memory';
        inst.panel.webview.html = AresQueryPanel.getHtml();
        setTimeout(() => {
            inst.postMessage({ type: 'loading' });
        }, 150);
        return inst;
    }

    /** Show a successful response. */
    public static show(context: vscode.ExtensionContext, data: AresResponse): AresQueryPanel {
        const inst = AresQueryPanel.ensurePanel(context);
        inst.updateTitle(data);
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
        inst.panel.title = 'ARES Memory · Error';
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
            AresQueryPanel.instance.panel.title = 'ARES Memory';
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

    private updateTitle(response: AresResponse): void {
        const titles: Record<string, string> = {
            'briefing': 'ARES · Briefing',
            'healthCheck': 'ARES · Health Check',
            'dead_code': 'ARES · Dead Code',
            'who_owns': 'ARES · Who Owns',
            'ARES Home': 'ARES · Home',
            'context_file': 'ARES · Context File',
            'why_exists': 'ARES · Why Exists',
            'impact': 'ARES · Impact Analysis',
            'drift': 'ARES · Drift Analysis',
            'decisions': 'ARES · Decisions',
        };
        const qt = response.query_type || '';
        this.panel.title = titles[qt] || 'ARES Memory';
    }

    private postMessage(msg: PanelMessage): void {
        this.panel.webview.postMessage(msg);
    }

    // -----------------------------------------------------------------------
    // HTML / CSS / JS — everything embedded, zero external deps
    // -----------------------------------------------------------------------

    private static getHtml(): string {
        return /* html */ `
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ARES Memory</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{background:var(--vscode-editor-background);color:var(--vscode-editor-foreground);font-family:var(--vscode-font-family,-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif);font-size:var(--vscode-font-size,13px);line-height:1.6;overflow-x:hidden}
.container{max-width:820px;margin:0 auto;padding:24px 20px}
.hidden{display:none!important}
@keyframes fadeIn{from{opacity:0}to{opacity:1}}
@keyframes slideUp{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}
@keyframes pulse{0%,100%{opacity:.4}50%{opacity:1}}
@keyframes spin{to{transform:rotate(360deg)}}

.header{display:flex;align-items:center;gap:12px;padding-bottom:16px;border-bottom:1px solid var(--vscode-panel-border);margin-bottom:20px;animation:fadeIn .3s ease}
.header-icon{width:36px;height:36px;border-radius:10px;background:var(--vscode-button-background);color:var(--vscode-button-foreground);display:flex;align-items:center;justify-content:center;font-size:18px;flex-shrink:0}
.header-text h1{font-size:18px;font-weight:600;letter-spacing:-.3px}
.query-badge{display:inline-block;margin-top:4px;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:1px;padding:2px 8px;border-radius:4px;background:var(--vscode-button-background);color:var(--vscode-button-foreground)}

.file-path{background:color-mix(in srgb,var(--vscode-editor-foreground) 6%,transparent);border:1px solid var(--vscode-panel-border);border-radius:6px;padding:8px 12px;font-family:var(--vscode-editor-font-family,'Consolas','Courier New',monospace);font-size:12px;color:var(--vscode-descriptionForeground);margin-bottom:20px;word-break:break-all;animation:slideUp .35s ease both}
.file-path::before{content:'\u0001F4C4 '}

.loading-state{display:flex;flex-direction:column;align-items:center;justify-content:center;padding:100px 24px;gap:16px;animation:fadeIn .2s ease}
.loading-spinner{width:32px;height:32px;border-radius:50%;border:3px solid color-mix(in srgb,var(--vscode-editor-foreground) 15%,transparent);border-top-color:var(--vscode-button-background);animation:spin .8s linear infinite}
.loading-text{font-size:13px;color:var(--vscode-descriptionForeground);animation:pulse 1.5s ease infinite}

.error-state{display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;padding:60px 24px;animation:fadeIn .35s ease}
.error-icon{font-size:48px;margin-bottom:16px}
.error-title{font-size:16px;font-weight:600;color:var(--vscode-foreground);margin-bottom:8px}
.error-message{font-size:13px;color:var(--vscode-descriptionForeground);max-width:400px;margin-bottom:20px;line-height:1.6}
.error-reasons{list-style:none;text-align:left;font-size:12px;line-height:2.2;background:color-mix(in srgb,var(--vscode-editor-foreground) 4%,transparent);border:1px solid var(--vscode-panel-border);border-radius:8px;padding:12px 20px}
.error-reasons li::before{content:'\u2022 ';color:#da3633}

.empty-state{display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;padding:60px 24px;color:var(--vscode-descriptionForeground);animation:fadeIn .4s ease}
.empty-icon{font-size:56px;margin-bottom:20px;opacity:.6}
.empty-title{font-size:16px;font-weight:600;color:var(--vscode-foreground);margin-bottom:8px}
.empty-hint{font-size:13px;line-height:1.6;max-width:380px;margin-bottom:16px}
.empty-command{display:inline-block;font-family:var(--vscode-editor-font-family,'Consolas','Courier New',monospace);font-size:12px;background:var(--vscode-button-background);color:var(--vscode-button-foreground);padding:6px 14px;border-radius:6px;margin-bottom:20px}
.empty-examples{list-style:none;text-align:left;font-size:12px;line-height:2}
.empty-examples li::before{content:'\u2022 ';color:var(--vscode-button-background)}

.query-type-badge{display:inline-flex;align-items:center;gap:6px;padding:4px 14px;border-radius:20px;font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.5px;margin-bottom:16px;animation:fadeIn .3s ease}
.query-type-badge.why_exists{background:color-mix(in srgb,var(--vscode-charts-blue) 15%,transparent);color:var(--vscode-charts-blue);border:1px solid color-mix(in srgb,var(--vscode-charts-blue) 30%,transparent)}
.query-type-badge.impact{background:color-mix(in srgb,var(--vscode-charts-red) 15%,transparent);color:var(--vscode-charts-red);border:1px solid color-mix(in srgb,var(--vscode-charts-red) 30%,transparent)}
.query-type-badge.drift{background:color-mix(in srgb,var(--vscode-charts-orange) 15%,transparent);color:var(--vscode-charts-orange);border:1px solid color-mix(in srgb,var(--vscode-charts-orange) 30%,transparent)}
.query-type-badge.dead_code{background:color-mix(in srgb,var(--vscode-charts-yellow) 15%,transparent);color:var(--vscode-charts-yellow);border:1px solid color-mix(in srgb,var(--vscode-charts-yellow) 30%,transparent)}
.query-type-badge.briefing{background:color-mix(in srgb,var(--vscode-charts-green) 15%,transparent);color:var(--vscode-charts-green);border:1px solid color-mix(in srgb,var(--vscode-charts-green) 30%,transparent)}
.query-type-badge.context_file{background:color-mix(in srgb,var(--vscode-charts-purple) 15%,transparent);color:var(--vscode-charts-purple);border:1px solid color-mix(in srgb,var(--vscode-charts-purple) 30%,transparent)}
.query-type-badge.healthCheck{background:color-mix(in srgb,var(--vscode-charts-blue) 15%,transparent);color:var(--vscode-charts-blue);border:1px solid color-mix(in srgb,var(--vscode-charts-blue) 30%,transparent)}
.query-type-badge.who_owns{background:color-mix(in srgb,var(--vscode-charts-cyan) 15%,transparent);color:var(--vscode-charts-cyan);border:1px solid color-mix(in srgb,var(--vscode-charts-cyan) 30%,transparent)}
.query-type-badge.decisions{background:color-mix(in srgb,var(--vscode-charts-green) 15%,transparent);color:var(--vscode-charts-green);border:1px solid color-mix(in srgb,var(--vscode-charts-green) 30%,transparent)}
.query-type-badge.default{background:color-mix(in srgb,var(--vscode-descriptionForeground) 10%,transparent);color:var(--vscode-descriptionForeground);border:1px solid color-mix(in srgb,var(--vscode-descriptionForeground) 20%,transparent)}

.answer-content{line-height:1.65;font-size:13px}
.answer-content p{margin:0 0 10px 0}
.answer-content p:last-child{margin-bottom:0}
.answer-content strong{font-weight:700;color:var(--vscode-foreground)}
.answer-content em{font-style:italic;color:var(--vscode-foreground)}
.answer-content code{background:color-mix(in srgb,var(--vscode-textBlockQuote-background) 60%,transparent);padding:1px 6px;border-radius:3px;font-family:var(--vscode-editor-font-family,'Consolas',monospace);font-size:12px;color:var(--vscode-textPreformat-foreground)}
.answer-content pre{background:var(--vscode-textBlockQuote-background);border:1px solid var(--vscode-panel-border);border-radius:6px;padding:12px 16px;margin:10px 0;overflow-x:auto}
.answer-content pre code{background:transparent;padding:0;font-size:12px}
.answer-content h3{font-size:14px;font-weight:700;margin:18px 0 8px 0;padding-bottom:6px;border-bottom:1px solid var(--vscode-panel-border)}
.answer-content h4{font-size:13px;font-weight:600;margin:14px 0 6px 0}
.answer-content ul{margin:6px 0;padding-left:20px}
.answer-content li{margin:3px 0;line-height:1.5}
.answer-content li.indent{margin-left:16px}
.answer-content .warning-line{color:var(--vscode-problemsWarningIcon-foreground);margin:6px 0;font-weight:600}
.answer-content .info-line{color:var(--vscode-charts-blue);margin:6px 0}
.answer-content .risk-high{color:var(--vscode-problemsErrorIcon-foreground);font-weight:700}
.answer-content .risk-medium{color:var(--vscode-problemsWarningIcon-foreground);font-weight:600}

.provenance-badge{display:inline-block;padding:1px 8px;border-radius:3px;font-size:10px;font-weight:600;text-transform:uppercase;margin-left:8px;vertical-align:middle}
.provenance-human{background:var(--vscode-badge-background);color:var(--vscode-badge-foreground)}
.provenance-agent{background:rgba(100,180,255,0.15);color:rgba(100,180,255,0.9)}
.staleness-fresh{color:var(--vscode-terminal-ansiGreen)}
.staleness-aging{color:var(--vscode-terminal-ansiYellow)}
.staleness-stale{color:var(--vscode-terminal-ansiBrightRed);font-weight:600}
.staleness-expired{color:#f44;font-weight:700}

.dash-header{display:flex;align-items:center;gap:14px;padding:20px;background:linear-gradient(135deg,color-mix(in srgb,var(--vscode-charts-blue) 8%,transparent),color-mix(in srgb,var(--vscode-charts-purple) 5%,transparent));border:1px solid color-mix(in srgb,var(--vscode-charts-blue) 20%,transparent);border-radius:10px;margin-bottom:20px;animation:fadeIn .4s ease}
.dash-header-icon{font-size:28px;filter:grayscale(20%)}
.dash-header-info{flex:1}
.dash-repo-name{font-size:20px;font-weight:700;color:var(--vscode-foreground);letter-spacing:-.3px}
.dash-repo-meta{font-size:12px;color:var(--vscode-descriptionForeground);margin-top:4px}
.dash-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:12px;margin-bottom:20px;animation:slideUp .4s ease}
.dash-card{background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:8px;padding:16px 18px;transition:border-color .2s ease}
.dash-card:hover{border-color:color-mix(in srgb,var(--vscode-charts-blue) 40%,var(--vscode-panel-border))}
.dash-card-icon{font-size:18px;margin-bottom:8px;opacity:.8}
.dash-card-label{font-size:11px;text-transform:uppercase;letter-spacing:.5px;color:var(--vscode-descriptionForeground);margin-bottom:4px}
.dash-card-value{font-size:22px;font-weight:700;color:var(--vscode-foreground);line-height:1.2}
.dash-card-sub{font-size:11px;color:var(--vscode-descriptionForeground);margin-top:4px}
.dash-section{margin-bottom:20px;animation:slideUp .45s ease}
.dash-section-title{font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;color:var(--vscode-descriptionForeground);margin-bottom:10px;padding-bottom:6px;border-bottom:1px solid var(--vscode-panel-border)}
.dash-activity-item{display:flex;align-items:center;gap:10px;padding:8px 0;border-bottom:1px solid color-mix(in srgb,var(--vscode-panel-border) 50%,transparent);font-size:13px;color:var(--vscode-foreground)}
.dash-activity-item:last-child{border-bottom:none}
.dash-activity-dot{width:6px;height:6px;border-radius:50%;background:var(--vscode-charts-blue);flex-shrink:0}
.dash-activity-time{margin-left:auto;font-size:11px;color:var(--vscode-descriptionForeground);white-space:nowrap}
.dash-actions{display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:8px;margin-top:16px;animation:slideUp .5s ease}
.dash-action{display:flex;align-items:center;gap:10px;padding:12px 14px;background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:8px;cursor:pointer;font-size:13px;color:var(--vscode-foreground);transition:all .2s ease}
.dash-action:hover{border-color:var(--vscode-charts-blue);background:color-mix(in srgb,var(--vscode-charts-blue) 6%,var(--vscode-editor-background))}
.dash-action-icon{font-size:16px}
.dash-status{display:inline-flex;align-items:center;gap:6px;padding:3px 10px;border-radius:12px;font-size:11px;font-weight:600}
.dash-status.ok{background:color-mix(in srgb,var(--vscode-terminal-ansiGreen) 15%,transparent);color:var(--vscode-terminal-ansiGreen)}
.dash-status.warn{background:color-mix(in srgb,var(--vscode-problemsWarningIcon-foreground) 15%,transparent);color:var(--vscode-problemsWarningIcon-foreground)}
.dash-status.err{background:color-mix(in srgb,var(--vscode-problemsErrorIcon-foreground) 15%,transparent);color:var(--vscode-problemsErrorIcon-foreground)}
.a-tag{background:var(--vscode-badge-background);color:var(--vscode-badge-foreground);padding:2px 8px;border-radius:10px;font-size:11px;display:inline-block}
.a-card{background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:6px;margin:8px 0}
.a-card-head{padding:8px 12px;font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.5px;color:var(--vscode-descriptionForeground);border-bottom:1px solid var(--vscode-panel-border)}
.a-card-body{padding:12px}
.a-file-card{display:flex;gap:8px;padding:8px;align-items:flex-start}
.a-file-icon{font-size:16px;flex-shrink:0}
.a-file-info{flex:1;min-width:0}
.a-file-path{font-family:var(--vscode-editor-font-family,'Consolas','Courier New',monospace);font-size:13px;word-break:break-all;color:var(--vscode-foreground)}
.a-file-meta{font-size:11px;color:var(--vscode-descriptionForeground);margin-top:2px}
.a-gap-item{display:flex;gap:8px;align-items:flex-start;padding:4px 0}
.a-gap-icon{color:var(--vscode-terminal-ansiYellow);flex-shrink:0}
.a-recommendation{background:color-mix(in srgb,var(--vscode-terminal-ansiYellow) 10%,transparent);border:1px solid color-mix(in srgb,var(--vscode-terminal-ansiYellow) 30%,transparent);border-radius:6px;padding:12px;margin:8px 0}
.a-decision-summary{margin:4px 0;font-size:13px;line-height:1.5}
.a-briefing{display:flex;flex-direction:column;gap:0}
.a-briefing-header{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;padding:16px 0;border-bottom:1px solid var(--vscode-panel-border);margin-bottom:16px}
.a-briefing-title{font-size:18px;font-weight:700;color:var(--vscode-foreground)}
.a-briefing-score{font-size:36px;font-weight:700;font-variant-numeric:tabular-nums;line-height:1}
.a-briefing-stack{display:flex;flex-wrap:wrap;gap:6px;margin-top:8px}
.a-animate-in{animation:slideUp .3s ease both}
.a-delay-1{animation-delay:100ms}
.a-delay-2{animation-delay:200ms}
.a-delay-3{animation-delay:300ms}
.a-delay-4{animation-delay:400ms}
.provenance-badge{font-size:10px;padding:2px 6px;border-radius:10px;text-transform:uppercase;letter-spacing:0.5px;font-weight:600;display:inline-block;margin-left:8px}
.provenance-human{background:color-mix(in srgb,var(--vscode-terminal-ansiGreen) 15%,transparent);color:var(--vscode-terminal-ansiGreen)}
.provenance-agent{background:color-mix(in srgb,var(--vscode-charts-purple) 15%,transparent);color:var(--vscode-charts-purple)}
.staleness-badge{font-size:10px;padding:2px 6px;border-radius:10px;text-transform:uppercase;letter-spacing:0.5px;font-weight:600;display:inline-block;margin-left:4px}
.staleness-fresh{background:color-mix(in srgb,var(--vscode-descriptionForeground) 15%,transparent);color:var(--vscode-descriptionForeground)}
.staleness-stale{background:color-mix(in srgb,var(--vscode-terminal-ansiYellow) 15%,transparent);color:var(--vscode-terminal-ansiYellow)}
.staleness-expired{background:color-mix(in srgb,#f44 15%,transparent);color:#f44}
.a-decision-card{background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:6px;margin:8px 0;padding:12px}
.a-decision-meta{display:flex;align-items:center;gap:8px;margin-bottom:6px;flex-wrap:wrap}
.a-decision-date{font-size:11px;color:var(--vscode-descriptionForeground)}
.a-decision-author{font-size:11px;font-weight:600;color:var(--vscode-foreground)}
</style>
</head>
<body>

<div id="loadingState" class="loading-state">
    <div class="loading-spinner"></div>
    <div class="loading-text">Querying ARES Memory\u2026</div>
</div>

<div id="errorState" class="error-state hidden">
    <div class="error-icon">\u26A0\uFE0F</div>
    <div id="errorTitle" class="error-title"></div>
    <div id="errorMessage" class="error-message"></div>
    <ul class="error-reasons">
        <li>Repository not ingested</li>
        <li>MCP server unavailable</li>
        <li>SQLite database missing</li>
        <li>Query failed</li>
    </ul>
</div>

<div id="content" class="container hidden">
    <div class="header">
        <div class="header-icon">\u26A1</div>
        <div class="header-text">
            <h1>ARES Memory</h1>
            <div id="queryBadge" class="query-badge hidden"></div>
            <div id="executionTime" class="query-badge hidden" style="background:var(--vscode-editorInfo-background);color:var(--vscode-editorInfo-foreground);margin-left:8px"></div>
        </div>
    </div>
    <div id="filePath" class="file-path hidden"></div>
    <div id="resultContent" class="hidden"></div>
    <div id="gapsSection" class="hidden">
        <div id="healthScore"></div>
        <div id="gapCounts"></div>
        <div id="gapList"></div>
    </div>
    <div id="dashboardSection" class="hidden">
        <div id="dashboardList"></div>
    </div>
    <div id="emptyState" class="empty-state hidden">
        <div class="empty-icon">\uD83E\uDDE0</div>
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
        loading: document.getElementById('loadingState'),
        error: document.getElementById('errorState'),
        errorTitle: document.getElementById('errorTitle'),
        errorMessage: document.getElementById('errorMessage'),
        content: document.getElementById('content'),
        queryBadge: document.getElementById('queryBadge'),
        filePath: document.getElementById('filePath'),
        resultContent: document.getElementById('resultContent'),
        gapsSection: document.getElementById('gapsSection'),
        healthScore: document.getElementById('healthScore'),
        gapCounts: document.getElementById('gapCounts'),
        gapList: document.getElementById('gapList'),
        dashboardSection: document.getElementById('dashboardSection'),
        dashboardList: document.getElementById('dashboardList'),
        emptyState: document.getElementById('emptyState'),
        executionTime: document.getElementById('executionTime'),
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

    function hideAllSections() {
        dom.queryBadge.classList.add('hidden');
        dom.resultContent.classList.add('hidden');
        dom.emptyState.classList.add('hidden');
        dom.gapsSection.classList.add('hidden');
        dom.dashboardSection.classList.add('hidden');
        var h = document.querySelector('.header');
        if (h) h.classList.add('hidden');
        var fp = document.getElementById('filePath');
        if (fp) fp.classList.add('hidden');
    }

    // --- Renderers (modular — add new renderers for future query types) ---

    function renderBriefing(data) {
        hideAll();
        dom.content.classList.remove('hidden');

        const p = data.project || {};
        const act = data.recent_activity || {};
        const handoff = data.agent_handoff || {};
        const gaps = data.critical_gaps || [];
        const freshness = data.context_freshness_hours || 999;

        // Freshness badge
        let freshnessHtml = '';
        if (freshness < 1) {
            freshnessHtml = '<span style="color:var(--vscode-terminal-ansiGreen)">● Fresh — ingested less than 1 hour ago</span>';
        } else if (freshness < 24) {
            freshnessHtml = '<span style="color:var(--vscode-terminal-ansiYellow)">● Aging — ingested ' + Math.round(freshness) + ' hours ago</span>';
        } else {
            freshnessHtml = '<span style="color:var(--vscode-terminal-ansiBrightRed)">● Very stale — ingested over 1 day ago. Re-ingest recommended.</span>';
        }

        // Health score color
        const hs = p.health_score || 0;
        let hsColor = 'var(--vscode-terminal-ansiGreen)';
        if (hs < 40) hsColor = 'var(--vscode-terminal-ansiBrightRed)';
        else if (hs < 70) hsColor = 'var(--vscode-terminal-ansiYellow)';

        // Tech stack badges
        const stack = (p.technology_stack || []).map(function(t) {
            return '<span class="a-tag">' + t + '</span>';
        }).join(' ');

        // Key modules
        let modulesHtml = '';
        (p.key_modules || []).forEach(function(m) {
            modulesHtml += '<div class="a-file-card"><span class="a-file-icon">📁</span><div class="a-file-info"><div class="a-file-path">' + m.path + '</div><div class="a-file-meta">' + (m.owner || 'Unknown') + ' · ' + (m.inbound_edges || 0) + ' dependents</div></div></div>';
        });

        // Recent activity
        let activityHtml = '<div class="a-card-body">' + (act.summary || 'No recent activity.') + '</div>';
        if (act.most_active_module) {
            activityHtml += '<div class="a-file-meta" style="margin-top:8px">Most active: ' + act.most_active_module + '</div>';
        }

        // Gaps
        let gapsHtml = '';
        gaps.forEach(function(g) {
            gapsHtml += '<div class="a-gap-item"><span class="a-gap-icon">⚠</span><span>' + g + '</span></div>';
        });

        // Recommended action
        let recHtml = '';
        if (data.recommended_first_action) {
            recHtml = '<div class="a-card a-recommendation"><div class="a-card-head">Recommended First Action</div><div class="a-card-body" style="font-weight:500">' + data.recommended_first_action + '</div></div>';
        }

        // Agent handoff
        let handoffHtml = '';
        if (handoff.last_session) {
            const ls = handoff.last_session;
            handoffHtml = '<div class="a-card"><div class="a-card-head">Agent Handoff</div><div class="a-card-body">';
            handoffHtml += '<div class="a-decision-summary">' + (ls.summary || 'No summary') + '</div>';
            if (ls.left_incomplete) {
                handoffHtml += '<div class="a-file-meta" style="margin-top:4px;color:var(--vscode-terminal-ansiYellow)">Left incomplete: ' + ls.left_incomplete + '</div>';
            }
            handoffHtml += '</div></div>';
        }

        dom.resultContent.innerHTML = '' +
            '<div class="a-briefing">' +
            '  <div class="a-briefing-header">' +
            '    <div>' +
            '      <div class="a-briefing-title">' + (p.name || 'Unknown Project') + '</div>' +
            '      <div class="a-briefing-score" style="color:' + hsColor + '">' + (Math.round(hs) || 0) + '</div>' +
            '    </div>' +
            '    <div class="a-briefing-stack">' + stack + '</div>' +
            '    <div style="margin-top:8px">' + freshnessHtml + '</div>' +
            '  </div>' +
            '  <div class="a-card a-animate-in a-delay-1"><div class="a-card-head">Architecture</div><div class="a-card-body">' + (p.architecture_summary || '') + '</div></div>' +
            '  <div class="a-card a-animate-in a-delay-2"><div class="a-card-head">Key Modules</div><div class="a-card-body">' + modulesHtml + '</div></div>' +
            '  <div class="a-card a-animate-in a-delay-3"><div class="a-card-head">Recent Activity (' + (act.since_days || 7) + ' days)</div>' + activityHtml + '</div>' +
            (gapsHtml ? '  <div class="a-card a-animate-in a-delay-4"><div class="a-card-head">Critical Gaps</div><div class="a-card-body">' + gapsHtml + '</div></div>' : '') +
            recHtml +
            handoffHtml +
            '</div>';
        dom.resultContent.classList.remove('hidden');
    }

    function renderDeadCode(data) {
        hideAll();
        dom.content.classList.remove('hidden');

        const result = data.result || {};
        const deadFiles = result.dead_files || [];
        const deadFunctions = result.dead_functions || [];
        const totalFiles = result.total_dead_files || 0;
        const totalFunctions = result.total_dead_functions || 0;
        const removableLines = result.estimated_removable_lines || 0;
        const warning = result.warning || '';

        let html = '';

        // Header
        html += '<div style="margin-bottom:20px">';
        html += '<h2 style="font-size:22px;font-weight:700;margin:0 0 8px 0">Dead Code Analysis</h2>';
        html += '<p style="color:var(--vscode-descriptionForeground);margin:0">Files and functions with no callers detected</p>';
        html += '</div>';

        // Summary cards
        html += '<div style="display:flex;gap:12px;margin-bottom:24px;flex-wrap:wrap">';
        html += '<div style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:6px;padding:16px 20px;min-width:140px">';
        html += '<div style="font-size:28px;font-weight:700;color:var(--vscode-problemsWarningIcon-foreground)">' + totalFiles + '</div>';
        html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-top:4px">Dead Files</div>';
        html += '</div>';
        html += '<div style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:6px;padding:16px 20px;min-width:140px">';
        html += '<div style="font-size:28px;font-weight:700;color:var(--vscode-problemsWarningIcon-foreground)">' + totalFunctions + '</div>';
        html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-top:4px">Dead Functions</div>';
        html += '</div>';
        html += '<div style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:6px;padding:16px 20px;min-width:140px">';
        html += '<div style="font-size:28px;font-weight:700;color:var(--vscode-terminal-ansiCyan)">' + removableLines.toLocaleString() + '</div>';
        html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-top:4px">Est. Removable Lines</div>';
        html += '</div>';
        html += '</div>';

        // Empty state
        if (totalFiles === 0 && totalFunctions === 0) {
            html += '<div style="text-align:center;padding:48px 20px;color:var(--vscode-descriptionForeground)">';
            html += '<div style="font-size:40px;margin-bottom:12px">✓</div>';
            html += '<div style="font-size:16px;font-weight:600;margin-bottom:8px">No Dead Code Detected</div>';
            html += '<div style="font-size:13px">All files and functions have callers. Codebase appears healthy.</div>';
            html += '</div>';
            dom.resultContent.innerHTML = html;
            dom.resultContent.classList.remove('hidden');
            return;
        }

        // Dead Files Section
        if (deadFiles.length > 0) {
            html += '<div style="margin-bottom:24px">';
            html += '<h3 style="font-size:14px;font-weight:600;margin:0 0 12px 0;padding-bottom:8px;border-bottom:1px solid var(--vscode-panel-border)">Dead Files (' + deadFiles.length + ')</h3>';
            html += '<div style="max-height:400px;overflow-y:auto">';
            for (const file of deadFiles) {
                html += '<div style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:4px;padding:12px 16px;margin-bottom:8px">';
                html += '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px">';
                html += '<code style="font-size:13px;color:var(--vscode-textLink-foreground);word-break:break-all">' + file.path + '</code>';
                html += '<span style="font-size:11px;color:var(--vscode-descriptionForeground);white-space:nowrap;margin-left:12px">' + file.language + '</span>';
                html += '</div>';
                html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-bottom:4px">Age: ' + file.age_days + ' days</div>';
                html += '<div style="font-size:12px;color:var(--vscode-problemsWarningIcon-foreground)">' + file.recommendation + '</div>';
                html += '</div>';
            }
            html += '</div>';
            html += '</div>';
        }

        // Dead Functions Section
        if (deadFunctions.length > 0) {
            html += '<div style="margin-bottom:24px">';
            html += '<h3 style="font-size:14px;font-weight:600;margin:0 0 12px 0;padding-bottom:8px;border-bottom:1px solid var(--vscode-panel-border)">Dead Functions (' + deadFunctions.length + ')</h3>';
            html += '<div style="max-height:400px;overflow-y:auto">';
            for (const fn of deadFunctions) {
                html += '<div style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:4px;padding:12px 16px;margin-bottom:8px">';
                html += '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px">';
                html += '<code style="font-size:13px;color:var(--vscode-terminal-ansiYellow)">' + fn.function_name + '</code>';
                html += '<span style="font-size:11px;color:var(--vscode-descriptionForeground);white-space:nowrap;margin-left:12px">' + fn.age_days + ' days</span>';
                html += '</div>';
                html += '<div style="font-size:12px;color:var(--vscode-textLink-foreground);margin-bottom:4px">' + fn.path + '</div>';
                html += '<div style="font-size:12px;color:var(--vscode-problemsWarningIcon-foreground)">' + fn.recommendation + '</div>';
                html += '</div>';
            }
            html += '</div>';
            html += '</div>';
        }

        // Warning box
        if (warning) {
            html += '<div style="background:rgba(255,170,0,0.1);border:1px solid rgba(255,170,0,0.4);border-radius:6px;padding:12px 16px;margin-top:8px">';
            html += '<div style="font-size:12px;color:var(--vscode-problemsWarningIcon-foreground);font-weight:600;margin-bottom:4px">⚠ Warning</div>';
            html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground)">' + warning + '</div>';
            html += '</div>';
        }

        dom.resultContent.innerHTML = html;
        dom.resultContent.classList.remove('hidden');
    }

    function renderQueryTypeBadgeHtml(queryType) {
        if (!queryType) return '';
        const labels = {
            'why_exists': 'Why Exists',
            'impact': 'Impact Analysis',
            'drift': 'Drift Analysis',
            'dead_code': 'Dead Code',
            'briefing': 'Briefing',
            'context_file': 'Context File',
            'healthCheck': 'Health Check',
            'who_owns': 'Who Owns',
            'decisions': 'Decisions',
            'ARES Home': 'ARES Home',
        };
        const label = labels[queryType] || queryType.replace(/_/g, ' ').replace(/\\b\\w/g, function(c) { return c.toUpperCase(); });
        const cls = labels[queryType] ? queryType : 'default';
        return '<div class="query-type-badge ' + cls + '">' + label + '</div>';
    }

    function renderMarkdown(text) {
        if (!text) return '';
        var h = text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/\\x60\\x60\\x60(\\w*)\\n([\\s\\S]*?)\\x60\\x60\\x60/g, '<pre><code>$2</code></pre>')
            .replace(/\\x60([^\\x60]+)\\x60/g, '<code>$1</code>')
            .replace(/\\*\\*([^\\*]+)\\*\\*/g, '<strong>$1</strong>')
            .replace(/\\*([^\\*]+)\\*/g, '<em>$1</em>')
            .replace(/^### (.+)$/gm, '<h4>$1</h4>')
            .replace(/^## (.+)$/gm, '<h3>$1</h3>')
            .replace(/^# (.+)$/gm, '<h2>$1</h2>')
            .replace(/^  [•\\-] (.+)$/gm, '<li class="indent">$1</li>')
            .replace(/^[•\\-] (.+)$/gm, '<li>$1</li>')
            .replace(/\\n\\n/g, '</p><p>')
            .replace(/\\n/g, '<br>');
        return '<div class="answer-content"><p>' + h + '</p></div>';
    }

    function renderWhoOwns(data) {
        hideAll();
        dom.content.classList.remove('hidden');

        var result = data.result || {};
        var contribs = result.contributors || [];
        var owner = result.owner || 'Unassigned';
        var filePath = data.file_path || data.entity || '';

        var html = '';
        html += renderQueryTypeBadgeHtml('who_owns');

        html += '<div style="margin-bottom:20px">';
        if (filePath) {
            html += '<div style="font-size:13px;color:var(--vscode-descriptionForeground);margin-bottom:12px;word-break:break-all"><code>' + filePath + '</code></div>';
        }
        html += '<div style="display:flex;align-items:center;gap:12px;margin-bottom:16px">';
        html += '<div style="width:40px;height:40px;border-radius:50%;background:color-mix(in srgb, var(--vscode-charts-cyan) 15%, transparent);border:2px solid color-mix(in srgb, var(--vscode-charts-cyan) 40%, transparent);display:flex;align-items:center;justify-content:center;font-size:18px">👤</div>';
        html += '<div>';
        html += '<div style="font-size:11px;text-transform:uppercase;letter-spacing:.5px;color:var(--vscode-descriptionForeground)">Primary Owner</div>';
        html += '<div style="font-size:18px;font-weight:700;color:var(--vscode-foreground)">' + (owner || 'Unassigned') + '</div>';
        html += '</div></div></div>';

        if (contribs.length > 0) {
            html += '<div style="margin-bottom:20px">';
            html += '<div style="font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;color:var(--vscode-descriptionForeground);margin-bottom:12px;padding-bottom:6px;border-bottom:1px solid var(--vscode-panel-border)">Contributors</div>';
            var maxPct = 0;
            for (var i = 0; i < contribs.length; i++) {
                if (contribs[i].percentage > maxPct) maxPct = contribs[i].percentage;
            }
            maxPct = Math.max(maxPct, 1);
            for (var i = 0; i < contribs.length; i++) {
                var c = contribs[i];
                var barWidth = Math.max(4, (c.percentage / maxPct) * 100);
                var isTop = i === 0;
                html += '<div style="margin-bottom:12px">';
                html += '<div style="display:flex;justify-content:space-between;align-items:baseline;margin-bottom:4px">';
                html += '<span style="font-size:13px;font-weight:' + (isTop ? '600' : '400') + ';color:var(--vscode-foreground)">' + c.name + '</span>';
                html += '<span style="font-size:13px;font-weight:700;color:' + (isTop ? 'var(--vscode-charts-cyan)' : 'var(--vscode-descriptionForeground)') + '">' + c.percentage + '%</span>';
                html += '</div>';
                html += '<div style="height:6px;background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:3px;overflow:hidden">';
                html += '<div style="height:100%;width:' + barWidth + '%;background:' + (isTop ? 'var(--vscode-charts-cyan)' : 'color-mix(in srgb, var(--vscode-charts-cyan) 40%, var(--vscode-panel-border))') + ';border-radius:2px;transition:width .3s ease"></div>';
                html += '</div></div>';
            }
            html += '</div>';
        }

        if (result.bus_factor !== undefined) {
            var bfColor = result.bus_factor <= 1 ? 'var(--vscode-problemsErrorIcon-foreground)' : result.bus_factor <= 3 ? 'var(--vscode-problemsWarningIcon-foreground)' : 'var(--vscode-terminal-ansiGreen)';
            html += '<div style="background:color-mix(in srgb, ' + bfColor + ' 8%, transparent);border:1px solid color-mix(in srgb, ' + bfColor + ' 25%, transparent);border-radius:8px;padding:12px 16px;display:flex;align-items:center;gap:10px">';
            html += '<span style="font-size:20px">👥</span>';
            html += '<div><div style="font-size:12px;color:var(--vscode-descriptionForeground)">Bus Factor</div>';
            html += '<div style="font-size:20px;font-weight:700;color:' + bfColor + '">' + result.bus_factor + '</div></div></div>';
        }

        dom.resultContent.innerHTML = html;
        dom.resultContent.classList.remove('hidden');
    }

    function escHtml(s) {
        if (!s) return '';
        return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
    }

    function renderDecisions(data) {
        hideAll();
        dom.content.classList.remove('hidden');
        
        var html = '';
        html += renderQueryTypeBadgeHtml('decisions');

        var decisions = data.related_decisions || [];
        if (decisions.length === 0) {
            html += '<div style="margin-top:20px;color:var(--vscode-descriptionForeground)">No decisions found.</div>';
        } else {
            html += '<div style="margin-top:20px">';
            decisions.forEach(function (dec) {
                var author = escHtml(dec.author || 'Unknown');
                var date = escHtml(dec.date || 'Unknown Date');
                var summary = escHtml(dec.summary || '');
                var provenanceBadge = '<span class="provenance-badge provenance-agent">agent</span>';
                if (!dec.source || dec.source !== 'agent') {
                    provenanceBadge = '<span class="provenance-badge provenance-human">human</span>';
                }
                var stalenessBadge = '';
                if (dec.staleness) {
                    var stalenessClass = 'staleness-' + (dec.staleness || 'fresh');
                    stalenessBadge = '<span class="' + stalenessClass + '" style="font-size:10px;font-weight:600;text-transform:uppercase;margin-left:8px">' + escHtml(dec.staleness) + '</span>';
                }
                
                html += '<div class="a-decision-card">';
                html += '  <div class="a-decision-meta">';
                html += '    <span class="a-decision-date">' + date + '</span>';
                html += '    <span class="a-decision-author">' + author + '</span>';
                html += '    ' + provenanceBadge;
                html += '    ' + stalenessBadge;
                html += '  </div>';
                html += '  <div class="a-decision-summary">' + summary + '</div>';
                html += '</div>';
            });
            html += '</div>';
        }

        dom.resultContent.innerHTML = html;
        dom.resultContent.classList.remove('hidden');
    }

    function renderContextFile(data) {
        hideAll();
        dom.content.classList.remove('hidden');

        var html = '';
        html += renderQueryTypeBadgeHtml('context_file');
        
        var filePath = data.file_path || data.output_path || '';
        
        html += '<div class="a-card" style="margin-top:20px">';
        html += '  <div class="a-card-head">Context File Generated</div>';
        html += '  <div class="a-card-body">';
        html += '    <p style="margin:0 0 12px 0;font-size:13px">A new context file has been generated with all relevant project data.</p>';
        if (filePath) {
            html += '    <div class="a-file-card" style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:4px;margin-bottom:12px">';
            html += '      <span class="a-file-icon">📄</span>';
            html += '      <div class="a-file-info"><div class="a-file-path">' + escHtml(filePath) + '</div></div>';
            html += '    </div>';
            html += '    <button class="open-file-btn" data-path="' + escHtml(filePath) + '" style="background:var(--vscode-button-background);color:var(--vscode-button-foreground);border:none;padding:6px 14px;border-radius:4px;cursor:pointer;font-size:13px;font-weight:600">Open File</button>';
        }
        html += '  </div>';
        html += '</div>';

        dom.resultContent.innerHTML = html;
        dom.resultContent.classList.remove('hidden');

        // Attach event listener
        setTimeout(function() {
            var btn = document.querySelector('.open-file-btn');
            if (btn) {
                btn.addEventListener('click', function(e) {
                    e.preventDefault();
                    vscode.postMessage({ command: 'openFile', path: this.getAttribute('data-path') });
                });
            }
        }, 50);
    }

    function renderGaps(data) {
        hideAllSections();
        dom.gapsSection.classList.remove('hidden');
        dom.healthScore.innerHTML = '';
        dom.gapCounts.innerHTML = '';
        dom.gapList.innerHTML = '';

        var score = Math.round(data.health_score || 0);
        var color = score > 70 ? 'var(--vscode-terminal-ansiGreen)' : score > 40 ? 'var(--vscode-problemsWarningIcon-foreground)' : 'var(--vscode-problemsErrorIcon-foreground)';

        var html = '';
        html += '<div class="dash-header" style="margin-bottom:24px">';
        html += '<div class="dash-header-icon">🏥</div>';
        html += '<div class="dash-header-info">';
        html += '<div style="display:flex;align-items:baseline;gap:12px">';
        html += '<div class="dash-repo-name">Repository Health</div>';
        html += '<div class="dash-status ' + (score > 70 ? 'ok' : score > 40 ? 'warn' : 'err') + '">Score: ' + score + '</div>';
        html += '</div>';
        html += '<div style="margin-top:10px;height:8px;background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:4px;overflow:hidden">';
        html += '<div style="height:100%;width:' + score + '%;background:' + color + ';border-radius:3px;transition:width .5s ease"></div>';
        html += '</div>';

        if (data.score_breakdown && data.score_breakdown.overall !== undefined) {
            var bd = data.score_breakdown;
            html += '<div style="display:flex;gap:16px;margin-top:12px;flex-wrap:wrap">';
            html += '<div style="font-size:11px;color:var(--vscode-descriptionForeground)">Files w/ Decisions <b style="color:var(--vscode-foreground)">' + Math.round(bd.files_with_decisions_term || 0) + '%</b></div>';
            html += '<div style="font-size:11px;color:var(--vscode-descriptionForeground)">Decisions w/ Reqs <b style="color:var(--vscode-foreground)">' + Math.round(bd.decisions_with_requirements_term || 0) + '%</b></div>';
            html += '<div style="font-size:11px;color:var(--vscode-descriptionForeground)">Files w/ Owners <b style="color:var(--vscode-foreground)">' + Math.round(bd.files_with_owners_term || 0) + '%</b></div>';
            html += '<div style="font-size:11px;color:var(--vscode-descriptionForeground)">Fresh Decisions <b style="color:var(--vscode-foreground)">' + Math.round(bd.fresh_decisions_term || 0) + '%</b></div>';
            html += '</div>';
        }
        html += '</div>';
        dom.healthScore.innerHTML = html;

        var counts = {};
        var gaps = data.gaps || [];
        gaps.forEach(function(g) { counts[g.gap_type] = (counts[g.gap_type] || 0) + 1; });

        var labels = { 'unknown_ownership': 'Unknown Ownership', 'code_without_decision': 'Code w/o Decision', 'stale_decision': 'Stale Decision', 'decision_without_code': 'Decision w/o Code', 'orphaned_requirement': 'Orphaned Req' };
        var icons = { 'unknown_ownership': '🔴', 'code_without_decision': '🟡', 'stale_decision': '🟡', 'decision_without_code': '⚪', 'orphaned_requirement': '⚪' };
        var gapColors = { 'unknown_ownership': 'var(--vscode-problemsErrorIcon-foreground)', 'code_without_decision': 'var(--vscode-problemsWarningIcon-foreground)', 'stale_decision': 'var(--vscode-problemsWarningIcon-foreground)', 'decision_without_code': 'var(--vscode-descriptionForeground)', 'orphaned_requirement': 'var(--vscode-descriptionForeground)' };

        var countsHtml = '<div style="display:flex;gap:8px;flex-wrap:wrap;margin-bottom:20px;animation:slideUp .35s ease">';
        Object.keys(labels).forEach(function(type) {
            var c = counts[type] || 0;
            var gc = gapColors[type] || 'var(--vscode-descriptionForeground)';
            countsHtml += '<div style="background:color-mix(in srgb, ' + gc + ' 8%, transparent);border:1px solid color-mix(in srgb, ' + gc + ' 20%, transparent);border-radius:8px;padding:10px 16px;min-width:100px;text-align:center">';
            countsHtml += '<div style="font-size:10px;text-transform:uppercase;letter-spacing:.5px;color:var(--vscode-descriptionForeground);margin-bottom:4px">' + labels[type] + '</div>';
            countsHtml += '<div style="font-size:20px;font-weight:700;color:' + gc + '">' + c + '</div>';
            countsHtml += '</div>';
        });
        countsHtml += '</div>';
        dom.gapCounts.innerHTML = countsHtml;

        // Hotspots section
        var hotspots = data.hotspots || [];
        console.log('[Webview] hotspots count:', hotspots.length);
        if (hotspots.length > 0) {
            var hotHtml = '<div style="margin-bottom:24px;animation:slideUp .45s ease">';
            hotHtml += '<div style="font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;color:var(--vscode-descriptionForeground);margin-bottom:12px;padding-bottom:6px;border-bottom:1px solid var(--vscode-panel-border)">Hotspots (High Churn \u00D7 Complexity)</div>';
            hotspots.forEach(function(h, idx) {
                var scorePct = Math.round((h.hotspot_score || 0) * 100);
                var scoreColor = scorePct > 80 ? 'var(--vscode-problemsErrorIcon-foreground)' : scorePct > 60 ? 'var(--vscode-problemsWarningIcon-foreground)' : 'var(--vscode-terminal-ansiGreen)';
                var recColor = scorePct > 80 ? 'var(--vscode-problemsErrorIcon-foreground)' : scorePct > 60 ? 'var(--vscode-problemsWarningIcon-foreground)' : 'var(--vscode-descriptionForeground)';
                hotHtml += '<div style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:8px;padding:12px 16px;margin-bottom:8px;animation:slideUp ' + (0.4 + idx * 0.04) + 's ease">';
                hotHtml += '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px">';
                hotHtml += '<code style="font-size:12px;color:var(--vscode-textLink-foreground);word-break:break-all;flex:1;min-width:0">' + (h.path || '') + '</code>';
                hotHtml += '<span style="font-size:14px;font-weight:700;color:' + scoreColor + ';margin-left:12px;white-space:nowrap">' + scorePct + '%</span>';
                hotHtml += '</div>';
                hotHtml += '<div style="height:4px;background:color-mix(in srgb, var(--vscode-panel-border) 50%, transparent);border-radius:2px;overflow:hidden;margin-bottom:8px">';
                hotHtml += '<div style="height:100%;width:' + Math.max(2, scorePct) + '%;background:' + scoreColor + ';border-radius:2px"></div>';
                hotHtml += '</div>';
                hotHtml += '<div style="display:flex;gap:16px;font-size:11px;color:var(--vscode-descriptionForeground);margin-bottom:6px">';
                hotHtml += '<span>\uD83D\uDD50 ' + (h.commits_30_days || 0) + ' commits (30d)</span>';
                hotHtml += '<span>\u2699\uFE0F Complexity: ' + (h.complexity_proxy || 0) + '</span>';
                hotHtml += '<span>\uD83D\uDC64 ' + (h.owner || 'Unknown') + '</span>';
                hotHtml += '</div>';
                if (h.recommendation) {
                    hotHtml += '<div style="font-size:11px;color:' + recColor + '">' + h.recommendation + '</div>';
                }
                hotHtml += '</div>';
            });
            hotHtml += '</div>';
            dom.gapList.innerHTML = hotHtml + dom.gapList.innerHTML;
        }

        var prio = { 'unknown_ownership': 1, 'code_without_decision': 2, 'stale_decision': 3, 'decision_without_code': 4, 'orphaned_requirement': 5 };
        var sorted = gaps.slice().sort(function(a, b) { return (prio[a.gap_type] || 9) - (prio[b.gap_type] || 9); });
        var top10 = sorted.slice(0, 10);

        if (top10.length > 0) {
            var listHtml = '<div style="font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;color:var(--vscode-descriptionForeground);margin-bottom:12px;padding-bottom:6px;border-bottom:1px solid var(--vscode-panel-border);animation:slideUp .4s ease">Top Gaps</div>';
            top10.forEach(function(gap, idx) {
                var gc = gapColors[gap.gap_type] || 'var(--vscode-descriptionForeground)';
                listHtml += '<div style="background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:8px;padding:14px 16px;margin-bottom:8px;animation:slideUp ' + (0.35 + idx * 0.05) + 's ease">';
                listHtml += '<div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">';
                listHtml += '<span style="color:' + gc + ';font-size:13px">' + (icons[gap.gap_type] || '⚪') + '</span>';
                listHtml += '<span style="font-size:13px;font-weight:600;color:' + gc + '">' + (labels[gap.gap_type] || gap.gap_type) + '</span>';
                listHtml += '</div>';
                listHtml += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-bottom:10px;word-break:break-all">' + gap.details + '</div>';
                listHtml += '<button class="nav-button gap-action" style="width:100%;padding:6px 12px;font-size:12px" data-node="' + (gap.node_label || gap.node_id || '').replace(/"/g, '&quot;') + '" data-details="' + (gap.details || '').replace(/"/g, '&quot;') + '">Create Decision →</button>';
                listHtml += '</div>';
            });
            dom.gapList.innerHTML += listHtml;
            
            document.querySelectorAll('.gap-action').forEach(function(btn) {
                btn.addEventListener('click', function() {
                    vscode.postMessage({ command: 'executeCommand', args: ['ares.createDecisionFromGap', btn.getAttribute('data-node'), btn.getAttribute('data-details')] });
                });
            });
        }
    }

    function renderDashboard(data) {
        if (data.query_type === 'healthCheck' || data.gaps) {
            renderGaps(data);
            return;
        }

        hideAllSections();

        if (!data.dashboard) {
            return;
        }

        var dash = data.dashboard;
        var repo = dash.repository || {};
        var graph = dash.graph || {};
        var integrity = dash.integrity || {};
        var coverage = dash.coverage || {};
        var intel = dash.intelligence || {};
        var perf = dash.performance || {};
        var health = dash.health || {};
        var activity = dash.activity || [];

        dom.dashboardSection.classList.remove('hidden');
        dom.dashboardList.innerHTML = '';

        var html = '';

        // Badge
        html += renderQueryTypeBadgeHtml('ARES Home');

        // Repository Header
        html += '<div class="dash-header">';
        html += '<div class="dash-header-icon">📦</div>';
        html += '<div class="dash-header-info">';
        html += '<div class="dash-repo-name">' + (repo.name || 'Repository') + '</div>';
        html += '<div class="dash-repo-meta">';
        if (repo.indexed) {
            html += '<span class="dash-status ok">Indexed ✓</span>';
        } else {
            html += '<span class="dash-status warn">Not Indexed</span>';
        }
        if (repo.branch) { html += '<span style="font-size:11px;color:var(--vscode-descriptionForeground)">Branch: ' + repo.branch + '</span>'; }
        if (repo.commit) { html += '<span style="font-size:11px;color:var(--vscode-descriptionForeground)">' + repo.commit.substring(0, 7) + '</span>'; }
        html += '</div></div></div>';

        // Stats Grid
        html += '<div class="dash-grid">';
        html += '<div class="dash-card"><div class="dash-card-icon">📂</div><div class="dash-card-label">Files</div><div class="dash-card-value">' + (repo.files || 0) + '</div><div class="dash-card-sub">' + (repo.functions || 0) + ' functions</div></div>';
        html += '<div class="dash-card"><div class="dash-card-icon">📦</div><div class="dash-card-label">Modules</div><div class="dash-card-value">' + (repo.modules || 0) + '</div><div class="dash-card-sub">' + (repo.directories || 0) + ' directories</div></div>';
        html += '<div class="dash-card"><div class="dash-card-icon">🔗</div><div class="dash-card-label">Edges</div><div class="dash-card-value">' + (graph.edges || 0).toLocaleString() + '</div><div class="dash-card-sub">' + (graph.nodes || 0).toLocaleString() + ' nodes</div></div>';
        var orphans = integrity.orphan_nodes || integrity.orphans || 0;
        var orphanCls = orphans > 100 ? 'err' : orphans > 10 ? 'warn' : 'ok';
        html += '<div class="dash-card"><div class="dash-card-icon">⛔</div><div class="dash-card-label">Orphans</div><div class="dash-card-value">' + orphans + '</div><div class="dash-card-sub"><span class="dash-status ' + orphanCls + '">' + (orphans === 0 ? 'Clean' : orphans + ' isolated') + '</span></div></div>';
        html += '<div class="dash-card"><div class="dash-card-icon">👥</div><div class="dash-card-label">Authors</div><div class="dash-card-value">' + (graph.authors || 0) + '</div><div class="dash-card-sub">' + (graph.commits || 0) + ' commits</div></div>';
        var density = graph.nodes > 1 ? ((graph.edges || 0) / (graph.nodes * (graph.nodes - 1) / 2) * 100).toFixed(3) : '0';
        html += '<div class="dash-card"><div class="dash-card-icon">📊</div><div class="dash-card-label">Density</div><div class="dash-card-value">' + density + '%</div><div class="dash-card-sub">Avg degree ' + (graph.average_degree ? graph.average_degree.toFixed(1) : '0') + '</div></div>';
        html += '</div>';

        // Intelligence Status
        var engines = [
            { name: 'Why Exists', status: intel.why_exists_status },
            { name: 'Impact', status: intel.impact_status },
            { name: 'Drift', status: intel.drift_status },
            { name: 'Traceability', status: intel.traceability_status },
            { name: 'Simulation', status: intel.simulation_status },
            { name: 'Git Memory', status: intel.git_memory_status },
            { name: 'Ownership', status: intel.ownership_status },
        ];
        html += '<div class="dash-section">';
        html += '<div class="dash-section-title">Intelligence Engines</div>';
        html += '<div style="display:flex;gap:6px;flex-wrap:wrap">';
        for (var ei = 0; ei < engines.length; ei++) {
            var eng = engines[ei];
            var st = eng.status || 'UNKNOWN';
            var stCls = st === 'READY' ? 'ok' : st === 'NOT AVAILABLE' ? 'warn' : 'err';
            var stLabel = st === 'READY' ? 'Ready' : st === 'NOT AVAILABLE' ? 'N/A' : st;
            html += '<span class="dash-status ' + stCls + '">' + eng.name + ' · ' + stLabel + '</span>';
        }
        html += '</div></div>';

        // Performance
        if (perf.total_time_ms) {
            html += '<div class="dash-section">';
            html += '<div class="dash-section-title">Ingest Performance</div>';
            html += '<div class="dash-grid">';
            html += '<div class="dash-card"><div class="dash-card-label">Scanner</div><div class="dash-card-value">' + (perf.scanner_ms || 0) + 'ms</div></div>';
            html += '<div class="dash-card"><div class="dash-card-label">AST Parsing</div><div class="dash-card-value">' + (perf.ast_parsing_ms || 0) + 'ms</div></div>';
            html += '<div class="dash-card"><div class="dash-card-label">Git Memory</div><div class="dash-card-value">' + (perf.git_memory_ms || 0) + 'ms</div></div>';
            html += '<div class="dash-card"><div class="dash-card-label">Total</div><div class="dash-card-value" style="color:var(--vscode-charts-blue)">' + perf.total_time_ms + 'ms</div></div>';
            html += '</div></div>';
        }

        // Activity Feed
        if (activity.length > 0) {
            html += '<div class="dash-section">';
            html += '<div class="dash-section-title">Recent Activity</div>';
            for (var ai = 0; ai < Math.min(activity.length, 6); ai++) {
                var act = activity[ai];
                html += '<div class="dash-activity-item">';
                html += '<div class="dash-activity-dot"></div>';
                html += '<span>' + act.message + '</span>';
                html += '<span class="dash-activity-time">' + (act.relative_time || '') + '</span>';
                html += '</div>';
            }
            html += '</div>';
        }

        // Quick Actions
        html += '<div class="dash-section">';
        html += '<div class="dash-section-title">Quick Actions</div>';
        html += '<div class="dash-actions">';
        var actions = [
            { icon: '🧠', label: 'Why Exists', cmd: 'ares.whyExists' },
            { icon: '🧬', label: 'Impact Analysis', cmd: 'ares.impactAnalysis' },
            { icon: '📊', label: 'Drift Analysis', cmd: 'ares.driftAnalysis' },
            { icon: '🏥', label: 'Health Check', cmd: 'ares.healthCheck' },
            { icon: '📋', label: 'Briefing', cmd: 'ares.briefing' },
            { icon: '🔍', label: 'Find Dead Code', cmd: 'ares.findDeadCode' },
            { icon: '🌐', label: 'Graph Explorer', cmd: 'ares.graphExplorer' },
            { icon: '\uD83C\uDF10', label: 'Architecture Map', cmd: 'ares.architecture' },
            { icon: '🔄', label: 'Ingest / Refresh', cmd: 'ares.ingest' },
        ];
        for (var aj = 0; aj < actions.length; aj++) {
            var ac = actions[aj];
            html += '<button class="dash-action" data-cmd="' + ac.cmd + '">';
            html += '<span class="dash-action-icon">' + ac.icon + '</span>';
            html += '<span>' + ac.label + '</span>';
            html += '</button>';
        }
        html += '</div></div>';

        dom.dashboardList.innerHTML = html;

        document.querySelectorAll('.dash-action').forEach(function(btn) {
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

    function renderGenericQuery(data) {
        var html = '';

        // Badge & Header
        html += renderQueryTypeBadgeHtml(data.query_type);
        if (data.file_path) {
            html += '<div style="font-size:13px;color:var(--vscode-descriptionForeground);margin-bottom:12px;word-break:break-all"><code>' + escHtml(data.file_path) + '</code></div>';
        }

        // Answer
        if (data.answer) {
            html += '<div class="a-card"><div class="a-card-head">Answer</div><div class="a-card-body">' + renderMarkdown(data.answer) + '</div></div>';
        }

        // Confidence Section — only show for evidence-based queries
        var raw = data.confidence;
        var score = 0;
        if (typeof raw === 'number') { score = raw; }
        else if (typeof raw === 'string') { score = parseFloat(raw) || 0; }
        else if (raw && typeof raw === 'object') { score = parseFloat(raw.score) || 0; }
        var pct = Math.min(100, Math.max(0, Math.round(score)));
        var reasons = (raw && typeof raw === 'object' && Array.isArray(raw.reasons)) ? raw.reasons : [];

        if (pct > 0 || reasons.length > 0) {
            var level = pct <= 33 ? 'low' : pct <= 66 ? 'medium' : 'high';
            var levelText = pct <= 33 ? 'Low' : pct <= 66 ? 'Medium' : 'High';
            var barColor = pct <= 33 ? 'var(--vscode-problemsErrorIcon-foreground)' : pct <= 66 ? 'var(--vscode-problemsWarningIcon-foreground)' : 'var(--vscode-terminal-ansiGreen)';

            html += '<div class="a-card"><div class="a-card-head">Confidence</div><div class="a-card-body" style="display:flex;align-items:center;gap:12px">';
            html += '<div style="font-size:13px;font-weight:600;min-width:60px;color:' + barColor + '">' + levelText + '</div>';
            html += '<div style="flex:1;height:6px;background:var(--vscode-editor-background);border:1px solid var(--vscode-panel-border);border-radius:3px;overflow:hidden">';
            html += '<div style="height:100%;width:' + pct + '%;background:' + barColor + ';border-radius:2px"></div></div>';
            html += '<div style="font-size:13px;font-weight:700;color:' + barColor + '">' + pct + '%</div>';
            html += '</div>';
            if (reasons.length > 0) {
                html += '<div class="a-card-body" style="padding-top:0">';
                for (var i = 0; i < reasons.length; i++) {
                    html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-bottom:4px">\u2713 ' + escHtml(reasons[i]) + '</div>';
                }
                html += '</div>';
            }
            html += '</div>';
        }

        // Evidence Section
        if (data.evidence && data.evidence.length > 0) {
            html += '<div class="a-card"><div class="a-card-head">Evidence (' + data.evidence.length + ')</div><div class="a-card-body">';
            data.evidence.forEach(function (ev) {
                var category = ev.category || ev.source || 'unknown';
                var value = ev.value || ev.detail || '';
                var isClickable = category.indexOf('/') !== -1 || category.indexOf('.') !== -1;
                
                html += '<div style="margin-bottom:12px">';
                if (isClickable) {
                    html += '<a href="#" class="evidence-link" data-path="' + escHtml(category) + '" style="font-family:var(--vscode-editor-font-family);font-size:13px;color:var(--vscode-textLink-foreground);word-break:break-all">' + escHtml(category) + '</a>';
                } else {
                    html += '<div style="font-size:13px;font-weight:600;color:var(--vscode-foreground)">' + escHtml(category) + '</div>';
                }
                
                if (value && value !== category) {
                    html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-top:4px">' + escHtml(value) + '</div>';
                }
                html += '</div>';
            });
            html += '</div></div>';
        }

        // Decisions Section
        if (data.related_decisions && data.related_decisions.length > 0) {
            html += '<div class="a-card"><div class="a-card-head">Related Decisions</div><div class="a-card-body">';
            data.related_decisions.forEach(function (dec) {
                var author = escHtml(dec.author || 'Unknown');
                var date = escHtml(dec.date || 'Unknown Date');
                var summary = escHtml(dec.summary || '');
                var provenanceBadge = '<span class="provenance-badge provenance-agent">agent</span>';
                if (!dec.source || dec.source !== 'agent') {
                    provenanceBadge = '<span class="provenance-badge provenance-human">human</span>';
                }
                var stalenessBadge = '<span class="staleness-badge staleness-fresh">fresh</span>';
                
                html += '<div class="a-decision-card" style="margin:0 0 8px 0">';
                html += '  <div class="a-decision-meta">';
                html += '    <span class="a-decision-date">' + date + '</span>';
                html += '    <span class="a-decision-author">' + author + '</span>';
                html += '    ' + provenanceBadge;
                html += '    ' + stalenessBadge;
                html += '  </div>';
                html += '  <div class="a-decision-summary">' + summary + '</div>';
                html += '</div>';
            });
            html += '</div></div>';
        }

        // Drift Section
        if (data.has_drift !== undefined && data.metadata && data.metadata.generator === "DriftGenerator") {
            html += '<div class="a-card"><div class="a-card-head">Drift Analysis</div><div class="a-card-body">';
            var driftColor = data.has_drift ? "#f48771" : "#89d185";
            html += '<div class="a-decision-meta">';
            html += '<span class="a-decision-author" style="color:' + driftColor + '">' + (data.has_drift ? "Drift Detected" : "No Drift") + '</span>';
            html += '<span class="a-decision-date">Score: ' + (data.drift_score !== undefined ? data.drift_score.toFixed(2) : "0.00") + '</span>';
            html += '</div>';
            html += '<div class="a-decision-summary">' + escHtml(data.summary || "No summary available.") + '</div>';
            
            if (data.decision_orphans && data.decision_orphans.length > 0) {
                html += '<div style="margin-top:16px;font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;color:var(--vscode-descriptionForeground)">Orphaned Decisions</div>';
                data.decision_orphans.forEach(function(orphan) {
                    html += '<div style="font-size:12px;color:var(--vscode-foreground);margin-top:4px;border-left:2px solid var(--vscode-panel-border);padding-left:8px">' + escHtml(orphan) + '</div>';
                });
            }
            html += '</div></div>';
        }

        // Hidden Coupling Section (co-change detection from ares_architecture)
        if (data.hidden_coupling && data.hidden_coupling.length > 0) {
            html += '<div class="a-card a-animate-in"><div class="a-card-head">Hidden Coupling Detected</div><div class="a-card-body">';
            html += '<div style="font-size:12px;color:var(--vscode-descriptionForeground);margin-bottom:12px">Files that change together frequently but have no declared dependency.</div>';
            data.hidden_coupling.forEach(function(pair, idx) {
                html += '<div style="border:1px solid var(--vscode-panel-border);border-radius:6px;padding:12px;margin-bottom:8px;animation:slideUp ' + (0.3 + idx * 0.05) + 's ease">';
                html += '<div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">';
                html += '<span style="font-size:14px;color:var(--vscode-terminal-ansiYellow)">\u26A0\uFE0F</span>';
                html += '<span style="font-size:12px;font-weight:700;color:var(--vscode-terminal-ansiYellow)">' + pair.co_change_count + ' co-changes in ' + pair.period_days + ' days</span>';
                html += '</div>';
                html += '<div style="display:flex;flex-direction:column;gap:6px;margin-bottom:8px">';
                html += '<code style="font-size:12px;color:var(--vscode-textLink-foreground);word-break:break-all">' + escHtml(pair.file_a) + '</code>';
                html += '<div style="text-align:center;font-size:11px;color:var(--vscode-descriptionForeground)">\u2195</div>';
                html += '<code style="font-size:12px;color:var(--vscode-textLink-foreground);word-break:break-all">' + escHtml(pair.file_b) + '</code>';
                html += '</div>';
                if (pair.risk) {
                    html += '<div style="font-size:11px;color:var(--vscode-problemsWarningIcon-foreground);line-height:1.5">' + escHtml(pair.risk) + '</div>';
                }
                html += '</div>';
            });
            html += '</div></div>';
        }

        // Write directly to DOM once
        dom.resultContent.innerHTML = html;
        dom.resultContent.classList.remove('hidden');

        // Attach event listeners for evidence links
        setTimeout(function() {
            document.querySelectorAll('.evidence-link').forEach(function(link) {
                link.addEventListener('click', function(e) {
                    e.preventDefault();
                    vscode.postMessage({ command: 'openFile', path: this.getAttribute('data-path') });
                });
            });
        }, 50);
    }

    function showData(data) {
        try {
            hideAll();
            dom.content.classList.remove('hidden');
            hideAllSections();

            if (isEmpty(data)) {
                renderEmptyState();
                return;
            }

            var isFullPage = data.query_type === 'briefing'
                || data.query_type === 'dead_code'
                || data.query_type === 'healthCheck'
                || data.query_type === 'who_owns'
                || data.query_type === 'decisions'
                || data.query_type === 'ARES Home'
                || data.query_type === 'context_file';

            var headerEl = document.querySelector('.header');
            if (isFullPage) {
                if (headerEl) headerEl.classList.add('hidden');
                if (dom.filePath) dom.filePath.classList.add('hidden');
            } else {
                if (headerEl) headerEl.classList.remove('hidden');
                if (dom.filePath) {
                    if (data.file_path) {
                        dom.filePath.textContent = data.file_path;
                        dom.filePath.classList.remove('hidden');
                    } else {
                        dom.filePath.classList.add('hidden');
                    }
                }
                if (dom.executionTime) {
                    if (data.execution_time_ms) {
                        dom.executionTime.textContent = data.execution_time_ms + ' ms';
                        dom.executionTime.classList.remove('hidden');
                    } else {
                        dom.executionTime.classList.add('hidden');
                    }
                }
                if (dom.queryBadge) dom.queryBadge.classList.add('hidden');
            }

            if (data.query_type === 'briefing') { renderBriefing(data); return; }
            if (data.query_type === 'dead_code') { renderDeadCode(data); return; }
            if (data.query_type === 'healthCheck') { renderGaps(data); return; }
            if (data.query_type === 'who_owns') { renderWhoOwns(data); return; }
            if (data.query_type === 'decisions') { renderDecisions(data); return; }
            if (data.query_type === 'ARES Home') { renderDashboard(data); return; }
            if (data.query_type === 'context_file') { renderContextFile(data); return; }

            // Everything else is rendered generically
            renderGenericQuery(data);

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
