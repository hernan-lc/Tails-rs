import sys

with open('src/vm/interpreter/heap_types.rs', 'r') as f:
    lines = f.readlines()

# Find the split function and insert after it
insert_idx = None
for i, line in enumerate(lines):
    if 'pub fn split(&self, input: &str) -> Vec<String>' in line:
        # Find end of this function (closing brace)
        j = i
        while j < len(lines) and lines[j].strip() != '}':
            j += 1
        insert_idx = j + 1  # After the closing }
        break

if insert_idx:
    new_lines = [
        '\n',
        '    /// Phase 3.4 - Fast-path: detect simple literal patterns (no regex\n',
        '    /// metacharacters) and indicate they can use str::find directly,\n',
        '    /// bypassing the regex crate entirely.\n',
        '    pub fn is_literal_pattern(&self) -> bool {\n',
        '        if self.compiled.is_some() {\n',
        '            return false;\n',
        '        }\n',
        '        self.source.find(".").is_none()\n',
        '            && self.source.find("^").is_none()\n',
        '            && self.source.find("$").is_none()\n',
        '            && !self.source.contains("*")\n',
        '            && !self.source.contains("+")\n',
        '            && !self.source.contains("?")\n',
        '            && !self.source.contains("(")\n',
        '            && !self.source.contains(")")\n',
        '            && !self.source.contains("[")\n',
        '            && !self.source.contains("]")\n',
        '            && !self.source.contains("{")\n',
        '            && !self.source.contains("}")\n',
        '            && !self.source.contains("\\\\")\n',
        '            && !self.source.contains("|")\n',
        '    }\n',
        '}\n',
    ]
    lines = lines[:insert_idx] + new_lines + lines[insert_idx:]
    
    with open('src/vm/interpreter/heap_types.rs', 'w') as f:
        f.writelines(lines)
    print(f'Inserted at line {insert_idx}')
else:
    print('Could not find insertion point')
    sys.exit(1)
