export function formatValue(val: any): string {
    if (val === null) return 'null';
    if (val === undefined) return 'undefined';
    if (typeof val === 'string') return JSON.stringify(val);
    if (typeof val === 'number' || typeof val === 'boolean') return String(val);
    if (typeof val === 'function') return '[Function]';
    if (val instanceof Error) return val.name + ': ' + val.message;
    try {
        if (Array.isArray(val)) return '[' + val.map(formatValue).join(', ') + ']';
        if (typeof val === 'object') {
            const entries = Object.entries(val).map(([k, v]) => k + ': ' + formatValue(v));
            return '{' + entries.join(', ') + '}';
        }
        return String(val);
    } catch {
        return String(val);
    }
}
