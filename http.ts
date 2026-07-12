// CommonJs
/**
 * @type {import('fastify').FastifyInstance} Instance of Fastify
 */
const fastify = require('fastify')({
  logger: true
})


fastify.listen({ port: 3000 }, function (err, address) {
  if (err) {
    fastify.log.error(err)
    process.exit(1)
  }
  // Server is now listening on ${address}
})

fetch("http://localhost:3000/")
    .then((res) => res.text())
    .then((body) => console.log(body))
    .catch((err) => console.error(err));
setInterval(() => {
  console.log("tick");
}, 5000);
// setTimeout(() => {
//   console.log("exiting");
//   process.exit();
// }, 15000);
