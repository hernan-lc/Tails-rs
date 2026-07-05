const greek = [..."αβγ Ω π".matchAll(/\p{Script=Greek}/gu)];
console.log("greek letters: " + greek.map((m) => m[0]).join(""));
const re = /(?<year>\d{4})-(?<month>\d{2})/u;
"released 2026-07".match(re).groups.year;
