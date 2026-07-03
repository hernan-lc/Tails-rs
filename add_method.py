with open('src/vm/interpreter/heap_types.rs', 'r') as f:
    lines = f.readlines()

# Find the line "    }" that ends the JsRegExp impl
for i in range(len(lines) - 1, -1, -1):
    if lines[i].strip() == '}' and i > 280:
        # Insert before this closing brace
        insert_lines = [
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
            '            && self.source.find("\\\\").is_none()\n',
            '            && !self.source.contains("|")\n',
            '    }\n',
        ]
        lines = lines[:i] + insert_lines + lines[i:]
        break

with open('src/vm/interpreter/heap_types.rs', 'w') as f:
    f.writelines(lines)
print('Method added successfully')
