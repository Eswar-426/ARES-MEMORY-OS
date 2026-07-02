import * as vscode from 'vscode';
import { McpClient } from './mcp-client';

export class ChatPanel {
    public static currentPanel: ChatPanel | undefined;
    public static readonly viewType = 'aresChat';

    private readonly panel: vscode.WebviewPanel;
    private readonly context: vscode.ExtensionContext;
    private readonly mcpClient: McpClient;
    private disposables: vscode.Disposable[] = [];
    private messages: { role: 'user' | 'assistant', content: string, actions?: any[], citations?: any[], isLoading?: boolean }[] = [];

    private constructor(panel: vscode.WebviewPanel, context: vscode.ExtensionContext, mcpClient: McpClient) {
        this.panel = panel;
        this.context = context;
        this.mcpClient = mcpClient;

        this.panel.webview.html = this.getHtml();
        this.setupMessageHandler();

        this.panel.onDidDispose(() => this.dispose(), null, this.disposables);
    }

    public static show(context: vscode.ExtensionContext, mcpClient: McpClient): ChatPanel {
        if (ChatPanel.currentPanel) {
            ChatPanel.currentPanel.panel.reveal(vscode.ViewColumn.Beside);
            return ChatPanel.currentPanel;
        }

        const panel = vscode.window.createWebviewPanel(
            ChatPanel.viewType,
            'ARES Chat',
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
            }
        );

        ChatPanel.currentPanel = new ChatPanel(panel, context, mcpClient);
        return ChatPanel.currentPanel;
    }

    public async handleUserQuery(query: string) {
        this.messages.push({ role: 'user', content: query });
        this.messages.push({ role: 'assistant', content: '...', isLoading: true });
        this.updateWebview();
        
        try {
            const result = await this.mcpClient.callTool('ares_chat', { query });
            let answer = "No answer returned.";
            let actions: any[] = [];
            let citations: any[] = [];
            
            if (result && result.content && Array.isArray(result.content)) {
                for (const block of result.content) {
                    if (block.type === 'text') {
                        try {
                            const payload = JSON.parse(block.text);
                            answer = payload.answer || answer;
                            actions = payload.actions || [];
                            citations = payload.citations || [];
                        } catch {
                            answer = block.text;
                        }
                        break;
                    }
                }
            }
            
            this.messages = this.messages.filter(m => !m.isLoading);
            this.messages.push({ role: 'assistant', content: answer, actions, citations });
            this.updateWebview();
        } catch (e: any) {
            this.messages = this.messages.filter((m: any) => !m.isLoading);
            this.messages.push({ role: 'assistant', content: `**Error:** Failed to execute ARES Engine.\n\n${e.message}` });
            this.updateWebview();
        }
    }

    private setupMessageHandler() {
        this.panel.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.command) {
                    case 'sendMessage':
                        await this.handleUserQuery(message.text);
                        return;
                    
                    case 'executeAction':
                        if (message.actionCommand === 'OpenGraph') {
                            vscode.commands.executeCommand('ares.graphExplorer', message.payload);
                        } else if (message.actionCommand === 'OpenFile') {
                            const uri = vscode.Uri.file(message.payload);
                            vscode.window.showTextDocument(uri);
                        } else {
                            vscode.window.showInformationMessage(`Action ${message.actionCommand} executing...`);
                        }
                        return;
                }
            },
            null,
            this.disposables
        );
    }

    private updateWebview() {
        this.panel.webview.postMessage({ command: 'updateMessages', messages: this.messages });
    }

    public dispose() {
        ChatPanel.currentPanel = undefined;
        this.panel.dispose();
        while (this.disposables.length) {
            const x = this.disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }

    private getHtml(): string {
        return /* html */ `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        :root {
            --bg-color: var(--vscode-editor-background);
            --text-color: var(--vscode-editor-foreground);
            --border-color: var(--vscode-panel-border);
            --accent-bg: var(--vscode-button-background);
            --accent-fg: var(--vscode-button-foreground);
            --accent-hover: var(--vscode-button-hoverBackground);
        }
        body {
            background-color: var(--bg-color);
            color: var(--text-color);
            font-family: var(--vscode-font-family);
            margin: 0;
            padding: 20px;
            display: flex;
            flex-direction: column;
            height: 100vh;
            box-sizing: border-box;
        }
        #chat-container {
            flex: 1;
            overflow-y: auto;
            margin-bottom: 20px;
            display: flex;
            flex-direction: column;
            gap: 15px;
        }
        .message {
            padding: 12px 16px;
            border-radius: 8px;
            max-width: 85%;
            word-wrap: break-word;
            line-height: 1.5;
        }
        .user-message {
            background-color: var(--accent-bg);
            color: var(--accent-fg);
            align-self: flex-end;
            border-bottom-right-radius: 2px;
        }
        .assistant-message {
            background-color: var(--vscode-editor-inactiveSelectionBackground);
            align-self: flex-start;
            border-bottom-left-radius: 2px;
            white-space: pre-wrap;
        }
        .actions-container {
            margin-top: 10px;
            display: flex;
            flex-wrap: wrap;
            gap: 8px;
        }
        .action-btn {
            background-color: var(--vscode-button-secondaryBackground, #333);
            color: var(--vscode-button-secondaryForeground, #fff);
            border: 1px solid var(--border-color);
            padding: 4px 10px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
        }
        .action-btn:hover {
            background-color: var(--vscode-button-secondaryHoverBackground, #444);
        }
        #input-container {
            display: flex;
            gap: 10px;
        }
        #message-input {
            flex: 1;
            background-color: var(--vscode-input-background);
            color: var(--vscode-input-foreground);
            border: 1px solid var(--vscode-input-border);
            padding: 10px;
            border-radius: 4px;
            font-family: inherit;
            resize: none;
            height: 44px;
        }
        #message-input:focus {
            outline: 1px solid var(--vscode-focusBorder);
            border-color: transparent;
        }
        #send-btn {
            background-color: var(--accent-bg);
            color: var(--accent-fg);
            border: none;
            padding: 0 20px;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 600;
        }
        #send-btn:hover {
            background-color: var(--accent-hover);
        }
    </style>
</head>
<body>
    <div id="chat-container">
        <div class="message assistant-message">Hello! I am ARES. Ask me anything about the codebase.</div>
    </div>
    
    <div id="input-container">
        <textarea id="message-input" placeholder="Ask a question about the repository..." rows="1"></textarea>
        <button id="send-btn">Send</button>
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        const chatContainer = document.getElementById('chat-container');
        const messageInput = document.getElementById('message-input');
        const sendBtn = document.getElementById('send-btn');
        
        function sendMessage() {
            const text = messageInput.value.trim();
            if (text) {
                vscode.postMessage({ command: 'sendMessage', text });
                messageInput.value = '';
            }
        }
        
        sendBtn.addEventListener('click', sendMessage);
        messageInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
            }
        });
        
        window.addEventListener('message', event => {
            const message = event.data;
            if (message.command === 'updateMessages') {
                chatContainer.innerHTML = '';
                
                // Add welcome message back
                const welcomeDiv = document.createElement('div');
                welcomeDiv.className = 'message assistant-message';
                welcomeDiv.textContent = 'Hello! I am ARES. Ask me anything about the codebase.';
                chatContainer.appendChild(welcomeDiv);
                
                message.messages.forEach(msg => {
                    const msgDiv = document.createElement('div');
                    msgDiv.className = 'message ' + (msg.role === 'user' ? 'user-message' : 'assistant-message');
                    let htmlContent = msg.content
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
    
// Parse [label](id) into clickable spans
htmlContent = htmlContent.replace(/\[([^\]]+)\]\(([^\)]+)\)/g, (match, label, id) => {
    return <span class="chat-citation" style="color: var(--vscode-textLink-foreground); cursor: pointer; text-decoration: underline;" data-id=" + id + "> + label + </span>;
});

msgDiv.innerHTML = htmlContent;

// Add click listeners to citations
setTimeout(() => {
    const citations = msgDiv.querySelectorAll('.chat-citation');
    citations.forEach(cit => {
        cit.addEventListener('click', () => {
            vscode.postMessage({
                command: 'executeAction',
                actionCommand: 'OpenGraph',
                payload: cit.getAttribute('data-id')
            });
        });
    });
}, 0);
                    
                    if (msg.citations && msg.citations.length > 0) {
                        const citationsDiv = document.createElement('div');
                        citationsDiv.style.marginTop = "10px";
                        citationsDiv.style.fontSize = "12px";
                        citationsDiv.style.fontWeight = "600";
                        citationsDiv.textContent = "Citations & Evidence:";
                        
                        const citationsList = document.createElement('div');
                        citationsList.style.display = "flex";
                        citationsList.style.flexWrap = "wrap";
                        citationsList.style.gap = "8px";
                        citationsList.style.marginTop = "5px";
                        
                        msg.citations.forEach(cit => {
                            const btn = document.createElement('button');
                            btn.className = 'action-btn';
                            btn.textContent = "📄 " + (cit.label || cit.node_id || cit);
                            btn.title = "View in Graph Explorer";
                            btn.onclick = () => {
                                vscode.postMessage({
                                    command: 'executeAction',
                                    actionCommand: 'OpenGraph',
                                    payload: cit.node_id || cit
                                });
                            };
                            citationsList.appendChild(btn);
                        });
                        citationsDiv.appendChild(citationsList);
                        msgDiv.appendChild(citationsDiv);
                    }

                    if (msg.actions && msg.actions.length > 0) {
                        const actionsDiv = document.createElement('div');
                        actionsDiv.className = 'actions-container';
                        
                        msg.actions.forEach(action => {
                            const btn = document.createElement('button');
                            btn.className = 'action-btn';
                            btn.textContent = action.label;
                            btn.onclick = () => {
                                vscode.postMessage({
                                    command: 'executeAction',
                                    actionCommand: action.command,
                                    payload: action.payload
                                });
                            };
                            actionsDiv.appendChild(btn);
                        });
                        msgDiv.appendChild(actionsDiv);
                    });
                            };
                            actionsDiv.appendChild(btn);
                        });
                        
                        msgDiv.appendChild(actionsDiv);
                    }
                    
                    chatContainer.appendChild(msgDiv);
                });
                
                chatContainer.scrollTop = chatContainer.scrollHeight;
            }
        });
    </script>
</body>
</html>`;
    }
}




