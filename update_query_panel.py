with open('extensions/ares-memory-vscode/src/queryPanel.ts', 'r', encoding='utf-8') as f:
    text = f.read()

# 1. Update AresResponse
text = text.replace(
    '    execution_time_ms?: number;\n}',
    '    execution_time_ms?: number;\n    gaps?: any[];\n    health_score?: number;\n}'
)

# 2. Update HTML skeleton
skeleton = """
<div id="gapsSection" class="section hidden">
  <div id="healthScore" style="margin-bottom: 20px;"></div>
  <div id="gapCounts" style="display: flex; gap: 10px; margin-bottom: 20px; flex-wrap: wrap;"></div>
  <div class="section-title">TOP GAPS</div>
  <div id="gapList" class="card-list"></div>
</div>
"""
idx = text.find('<div id="dashboardSection"')
text = text[:idx] + skeleton + '\n' + text[idx:]

# 3. Add JS render function & switch
render_js = """
    function renderGaps(data) {
        if (!data.gaps) {
            dom.gapsSection.classList.add('hidden');
            return;
        }

        dom.gapsSection.classList.remove('hidden');
        dom.healthScore.innerHTML = '';
        dom.gapCounts.innerHTML = '';
        dom.gapList.innerHTML = '';

        // Hide other generic sections
        document.getElementById('evidenceSection')?.classList.add('hidden');
        document.getElementById('relatedSection')?.classList.add('hidden');
        document.getElementById('traceabilitySection')?.classList.add('hidden');
        document.getElementById('dashboardSection')?.classList.add('hidden');

        // Render Health Score
        var scoreStr = '<div style="padding:16px;background:var(--vscode-editor-inactiveSelectionBackground);border-radius:8px;">';
        scoreStr += '<h2 style="margin-bottom:8px;">REPOSITORY HEALTH &nbsp; <span style="float:right">Score: ' + Math.round(data.health_score) + '</span></h2>';
        scoreStr += '<div style="width:100%;height:10px;background:var(--vscode-editor-background);border-radius:5px;overflow:hidden;">';
        var color = data.health_score > 80 ? 'var(--vscode-testing-iconPassed)' : data.health_score > 50 ? 'var(--vscode-testing-iconQueued)' : 'var(--vscode-testing-iconFailed)';
        scoreStr += '<div style="width:' + Math.round(data.health_score) + '%;height:100%;background:' + color + ';"></div>';
        scoreStr += '</div>';
        scoreStr += '</div>';
        dom.healthScore.innerHTML = scoreStr;

        // Render counts
        var counts = {};
        data.gaps.forEach(function(g) {
            counts[g.gap_type] = (counts[g.gap_type] || 0) + 1;
        });
        
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

        // Priorities
        var prio = {
            'unknown_ownership': 1,
            'code_without_decision': 2,
            'stale_decision': 3,
            'decision_without_code': 4,
            'orphaned_requirement': 5
        };

        var sorted = data.gaps.sort(function(a, b) {
            return prio[a.gap_type] - prio[b.gap_type];
        });

        var top10 = sorted.slice(0, 10);
        
        var icons = {
            'unknown_ownership': '🔴',
            'code_without_decision': '🟡',
            'stale_decision': '🟡',
            'decision_without_code': '⚪',
            'orphaned_requirement': '⚪'
        };

        top10.forEach(function(gap) {
            var card = document.createElement('div');
            card.className = 'card animate-slide';
            var html = '<div class="card-header">';
            html += '<div class="card-title">' + icons[gap.gap_type] + ' ' + labels[gap.gap_type] + '</div>';
            html += '</div>';
            html += '<div class="card-body">';
            html += '<code>' + gap.node_label + '</code>';
            html += '<p style="margin-top:8px;opacity:0.8">' + gap.details + '</p>';
            html += '</div>';
            
            var btn = document.createElement('button');
            btn.className = 'nav-button';
            btn.style.marginTop = '12px';
            btn.style.width = '100%';
            btn.innerText = 'Create Decision →';
            btn.onclick = function() {
                vscode.postMessage({ 
                    type: 'createDecision', 
                    file_path: gap.node_label, 
                    gap_description: gap.details 
                });
            };
            
            card.innerHTML = html;
            card.appendChild(btn);
            dom.gapList.appendChild(card);
        });
    }
"""

idx3 = text.find('function renderDashboard')
text = text[:idx3] + render_js + '\n    ' + text[idx3:]

# Add dom references
idx4 = text.find("dashboardSection: document.getElementById('dashboardSection')")
dom_refs = ",\n            gapsSection: document.getElementById('gapsSection'),\n            healthScore: document.getElementById('healthScore'),\n            gapCounts: document.getElementById('gapCounts'),\n            gapList: document.getElementById('gapList')"
text = text[:idx4+63] + dom_refs + text[idx4+63:]

# Add to event listener
idx5 = text.find('if (data.dashboard) {')
text = text[:idx5] + 'if (data.gaps) {\n                renderGaps(data);\n            }\n            ' + text[idx5:]

# 4. Handle createDecision in TypeScript
ts_handler = """
                case 'createDecision': {
                    const { file_path, gap_description } = message;
                    if (AresQueryPanel.mcpClient) {
                        AresQueryPanel.mcpClient.callTool('ares_record_decision', {
                            title: `Decision needed: ${gap_description}`,
                            description: `Identified by ARES health check: ${gap_description}`,
                            status: 'draft',
                            impacted_paths: [file_path]
                        }).then(() => {
                            vscode.window.showInformationMessage('Decision draft created.');
                        }).catch((err) => {
                            vscode.window.showErrorMessage('Failed to create decision: ' + err.message);
                        });
                    }
                    return;
                }
"""
idx6 = text.find("case 'navigate':")
text = text[:idx6] + ts_handler + '                ' + text[idx6:]

with open('extensions/ares-memory-vscode/src/queryPanel.ts', 'w', encoding='utf-8') as f:
    f.write(text)
print('Updated queryPanel.ts')
