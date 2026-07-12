//import * as z from './modules/zod-ts/zod.ts'
import * as z from './modules/zod-ts/zod.ts'

// Primitives with chained refinements (these call Object.getOwnPropertyDescriptors via mergeDefs)
const stringSchema = z.string().min(2).max(20).email()
const numberSchema = z.number().min(0).max(120).int()
const boolSchema = z.boolean()

// Object schema built from the above
const userSchema = z.object({
  id: z.number().int().positive(),
  name: z.string().min(1),
  email: z.string().email(),
  age: z.number().min(0).max(120),
  active: z.boolean(),
  tags: z.array(z.string()).min(1),
  role: z.enum(['admin', 'user', 'guest']),
  status: z.union([z.literal('on'), z.literal('off')]),
  nickname: z.string().optional(),
})

type User = z.infer<typeof userSchema>

const user: User = {
  id: 1,
  name: 'pepe',
  email: 'pepe@example.com',
  age: 17,
  active: true,
  tags: ['a', 'b'],
  role: 'admin',
  status: 'on',
  nickname: 'p',
}

const parsed = userSchema.parse(user)
console.log('parsed user:', parsed)

// Fix defaults / coercion
const coerced = z.coerce.number().parse('42')
console.log('coerced:', coerced)

// safeParse error path
const bad = userSchema.safeParse({ id: -1, name: '', email: 'x', age: 999, active: 'no', tags: [], role: 'god', status: 'maybe' })
console.log('safeParse success:', bad.success)

// .pick / .omit (clone + mergeDefs chains)
const light = userSchema.pick({ id: true, name: true })
console.log('pick keys:', Object.keys(light.shape).join(','))

const partial = userSchema.partial()
console.log('partial parse ok:', partial.safeParse({}).success)

// direct Object API used by zod internally
const descs = Object.getOwnPropertyDescriptors({ a: 1, b: 2 })
const rebuilt = Object.defineProperties({}, descs)
console.log('rebuilt:', rebuilt.a, rebuilt.b, descs.a.value, descs.b.enumerable)

console.log('ALL OK')
