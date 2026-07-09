const express = require('express');
const app = express();
const port = 3000;

app.get('/', (req, res) => {
  res.send('Hello World!');
});
app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
console.log(app)
setTimeout(() => {
  console.log("exiting")
  process.exit();
},10000)
