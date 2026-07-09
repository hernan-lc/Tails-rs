import * as z from 'zod'
const schema = z.object({
  name: z.string().min(1),
  age: z.number().min(1),
  email: z.string().email(),
})
type User = z.infer<typeof schema>
const user: User = {
  name: 'pepe',
  age: 17,
  email: 'e@gmail.com',
}
const result = schema.parse(user)
console.log(result)
