with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()
for i, line in enumerate(lines):
    if 'app_state' in line and ('struct' in line.lower() or 'store' in line.lower() or 'Arc' in line or 'clone' in line.lower()):
        start = max(0, i-1)
        end = min(len(lines), i+3)
        for j in range(start, end):
            print(f'{j+1:4d}: {lines[j]}', end='')
        print('---')
