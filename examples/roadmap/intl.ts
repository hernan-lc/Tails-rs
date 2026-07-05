const fmt = new Intl.ListFormat("en", { style: "long", type: "conjunction" });
console.log(fmt.format(["Rust", "WebAssembly", "JavaScript"]));
new Intl.NumberFormat("de-DE", { style: "currency", currency: "EUR" }).format(
  1234567.89,
);
