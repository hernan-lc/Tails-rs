let [a, b] = [0n, 1n];
const fib = [];
while (fib.length < 40) {
  fib.push(a);
  [a, b] = [b, a + b];
}
console.log("fib(39) = " + fib.at(-1));
fib.slice(0, 12).join(", ");
