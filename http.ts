const express = require('express');
const app = express();
const port = 3000;

app.get('/', (req, res) => {
  res.send('Hello World!');
});

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
setInterval(() => {
  console.log("tick");
fetch("http://localhost:3000/")
    .then((res) => res.text())
    .then((body) => console.log(body))
    .catch((err) => console.error(err));
}, 5000);
// setTimeout(() => {
//   console.log("exiting");
//   process.exit();
// }, 15000);
