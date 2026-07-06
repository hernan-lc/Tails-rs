const math = require("./math_heavy.cjs");
const sum = math.add(2, 3);
const product = math.mul(4, 5);
export default { sum, product, pi: math.PI };
