import re

with open('crates/ares-store/src/repositories/gaps.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = re.sub(r'\.map_err\(\|e\| AresError::sqlite\(\"[^\"]+\", e\)\)', '.map_err(AresError::db)', text)
text = text.replace('self.store.get_connection()', 'self.store.get_conn()')

with open('crates/ares-store/src/repositories/gaps.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('Fixed gaps.rs')
