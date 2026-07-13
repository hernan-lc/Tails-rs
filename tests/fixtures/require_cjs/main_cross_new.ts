const LRU = require("./lru_export.cjs");
const c = new LRU();
c.set("a", 1);
export default c.get("a");
