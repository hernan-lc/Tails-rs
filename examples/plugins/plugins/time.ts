export default {
  metadata: {
    name: "time",
    version: "1.0.0"
  },
  now() {
    return new Date().toISOString();
  }
};
