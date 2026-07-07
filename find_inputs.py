with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()
import re
matches = re.findall(r'\.handler\(move \|input:\s*(\w+)', text)
print('Input types used:', set(matches))
for i, line in enumerate(text.split('\n')):
    if 'MemoryQueryInput' in line and ('struct' in line or 'pub' in line or '{' in line):
        start = max(0, i)
        end = min(len(text.split('\n')), i+10)
        for j in range(start, end):
            print(f'{j+1:4d}: {text.split(chr(10))[j]}')
        print('---')
        break
