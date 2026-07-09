import * as z from 'zod'
const schema = z.object({
  name: z.string(),
  age: z.number(),
  email: z.string(),
})
type User = z.infer<typeof schema>
const user: User = {
  name: '',
  age: 0,
  email: '',
}
const result = schema.parse(user)
console.log(result)
